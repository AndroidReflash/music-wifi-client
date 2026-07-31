use anyhow::{Context, Result};

use super::verifier::DummyCertVerifier;
use quinn::{ClientConfig, Connection, Endpoint, RecvStream};
use std::net::{Ipv6Addr, SocketAddr};
use std::sync::Arc;
use std::sync::Once;
use std::time::Duration;
use tokio::runtime::Runtime;

use super::error::NetworkError;

#[derive(uniffi::Object)]
pub struct QuicSession {
    connection: Connection,
    _endpoint: Endpoint,
    rt: Runtime,
}

#[derive(uniffi::Enum)]
pub enum Command {
    GetMusic,
    GetPlaylist,
    SubscribeSse,
}

#[uniffi::export(callback_interface)]
pub trait SseCallback: Send + Sync {
    fn on_event(&self, message: String);
}

#[uniffi::export]
impl QuicSession {
    /// Подключается к серверу. Ни один отказ здесь не роняет процесс: адрес набирает
    /// человек, а паника через границу FFI — это `abort` всего приложения.
    #[uniffi::constructor]
    pub fn connect(addr_str: String) -> Result<Arc<Self>, NetworkError> {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            // Провайдер шифрования ставится один раз на процесс. Отказ означает, что
            // его уже поставил кто-то другой, — а нам нужно именно это.
            drop(rustls::crypto::ring::default_provider().install_default());
        });

        let addr: SocketAddr = addr_str.parse().map_err(|err| NetworkError::Address {
            reason: format!("адрес «{addr_str}» не разобран: {err}"),
        })?;

        let rt = Runtime::new().map_err(|err| NetworkError::Connect {
            reason: format!("не поднялся рантайм: {err}"),
        })?;

        let (connection, endpoint) = rt.block_on(async {
            let crypto = rustls::ClientConfig::builder()
                .dangerous()
                .with_custom_certificate_verifier(Arc::new(DummyCertVerifier))
                .with_no_client_auth();

            let idle = Duration::from_secs(90)
                .try_into()
                .map_err(|err| NetworkError::Connect {
                    reason: format!("срок молчания не принят: {err}"),
                })?;
            let mut transport = quinn::TransportConfig::default();
            transport.keep_alive_interval(Some(Duration::from_secs(30)));
            transport.max_idle_timeout(Some(idle));

            let quic_crypto = quinn::crypto::rustls::QuicClientConfig::try_from(crypto)
                .map_err(|err| NetworkError::Connect {
                    reason: format!("настройки шифрования не годятся: {err}"),
                })?;
            let mut client_config = ClientConfig::new(Arc::new(quic_crypto));
            client_config.transport_config(Arc::new(transport));

            // Свой сокет — любой свободный порт; шестая версия протокола принимает и
            // четвёртую, поэтому адрес сервера может быть каким угодно.
            let bind = SocketAddr::from((Ipv6Addr::UNSPECIFIED, 0));
            let mut endpoint = Endpoint::client(bind).map_err(|err| NetworkError::Connect {
                reason: format!("свой сокет не открылся: {err}"),
            })?;
            endpoint.set_default_client_config(client_config);

            let conn = endpoint
                .connect(addr, &addr.ip().to_string())
                .map_err(|err| NetworkError::Connect {
                    reason: format!("подключение не начато: {err}"),
                })?
                .await
                .map_err(|err| NetworkError::Connect {
                    reason: format!("рукопожатие не состоялось: {err}"),
                })?;

            Ok((conn, endpoint))
        })?;

        Ok(Arc::new(QuicSession {
            connection,
            _endpoint: endpoint,
            rt,
        }))
    }

    pub fn send_command(&self, command: Command, data: Vec<u8>) -> Vec<u8> {
        let command_id = match command {
            Command::GetMusic => 1,
            Command::GetPlaylist => 2,
            Command::SubscribeSse => 3,
        };

        self.rt.block_on(async {
            match self.connection.open_bi().await {
                Ok((mut send, mut recv)) => {
                    let mut packet = vec![command_id];
                    packet.extend(data);
                    
                    let _ = send.write_all(&packet).await;
                    let _ = send.finish();

                    let mut response = Vec::new();
                    let mut buf = vec![0; 4096];
                    loop {
                        match recv.read(&mut buf).await {
                            Ok(Some(n)) if n > 0 => {
                                response.extend_from_slice(&buf[..n]);
                            }
                            _ => break,
                        }
                    }
                    return response;
                }
                Err(_) => {}
            }
            Vec::new()
        })
    }

    pub fn subscribe_sse(&self, callback: Box<dyn SseCallback>) {
        self.rt.block_on(async {
            loop {
                match self.connection.open_bi().await {
                    Ok((mut send, mut recv)) => {
                        if send.write_all(&[3]).await.is_ok() {
                            let _ = send.finish();
                            let mut buf = vec![0; 1024];
                            let mut active = true;
                            while active {
                                match recv.read(&mut buf).await {
                                    Ok(Some(n)) if n > 0 => {
                                        let msg = String::from_utf8_lossy(&buf[..n]).into_owned();
                                        callback.on_event(msg);
                                    }
                                    _ => {
                                        active = false;
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => {}
                }
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        });
    }
}

impl QuicSession {
    pub fn get_runtime_handle(&self) -> &Runtime {
        &self.rt
    }

    /// Открывает поток на файл: отправляет команду, читает объявленную длину и отдаёт
    /// приёмную сторону, из которой байты пойдут с начала файла.
    ///
    /// Диапазона в протоколе нет — попросить кусок с середины нельзя, поэтому тот, кто
    /// читает, обязан копить полученное сам.
    pub async fn open_track(&self, path: &str) -> Result<(u64, RecvStream)> {
        let (mut send, mut recv) = self
            .connection
            .open_bi()
            .await
            .context("поток к серверу не открылся")?;

        let mut packet = vec![1u8];
        packet.extend_from_slice(path.as_bytes());
        send.write_all(&packet)
            .await
            .context("запрос файла не ушёл")?;
        send.finish().context("запрос файла не закрылся")?;

        let mut size_buf = [0u8; 8];
        recv.read_exact(&mut size_buf)
            .await
            .context("сервер не сказал длину файла")?;

        Ok((u64::from_be_bytes(size_buf), recv))
    }

}
