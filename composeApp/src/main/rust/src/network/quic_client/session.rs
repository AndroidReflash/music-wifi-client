use anyhow::{Context, Result};
use log::info;
use quinn::{ClientConfig, Connection, Endpoint, RecvStream};
use std::net::{Ipv6Addr, SocketAddr};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Once};
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::watch;

use super::error::NetworkError;
use super::verifier::DummyCertVerifier;

static ACTIVE_SSE_SUBSCRIPTIONS: AtomicUsize = AtomicUsize::new(0);

#[derive(uniffi::Object)]
pub struct QuicSession {
    connection: Connection,
    _endpoint: Endpoint,
    rt: Runtime,
}

#[derive(uniffi::Object)]
pub struct SseCancelToken {
    cancelled: AtomicBool,
    tx: watch::Sender<bool>,
}

#[uniffi::export]
impl SseCancelToken {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        let (tx, _rx) = watch::channel(false);
        Arc::new(Self {
            cancelled: AtomicBool::new(false),
            tx,
        })
    }

    pub fn cancel(&self) {
        let was = self.cancelled.swap(true, Ordering::SeqCst);
        let _ = self.tx.send(true);
        if !was {
            info!(
                "sse: cancel requested, active={}",
                ACTIVE_SSE_SUBSCRIPTIONS.load(Ordering::SeqCst)
            );
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

impl SseCancelToken {
    fn subscribe(&self) -> watch::Receiver<bool> {
        self.tx.subscribe()
    }
}

#[derive(uniffi::Enum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    GetMusic,
    GetPlaylist,
    SubscribeSse,
}

impl Command {
    fn id(self) -> u8 {
        match self {
            Command::GetMusic => 1,
            Command::GetPlaylist => 2,
            Command::SubscribeSse => 3,
        }
    }
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
        match self.send_command_result(command, data) {
            Ok(response) => response,
            Err(err) => {
                info!("network: send_command failed: {err}");
                Vec::new()
            }
        }
    }

    pub fn send_command_result(
        &self,
        command: Command,
        data: Vec<u8>,
    ) -> Result<Vec<u8>, NetworkError> {
        self.rt.block_on(self.request(command, data))
    }

    pub fn subscribe_sse(&self, callback: Box<dyn SseCallback>) {
        let token = SseCancelToken::new();
        self.subscribe_sse_with_cancel(callback, token);
    }

    pub fn subscribe_sse_with_cancel(
        &self,
        callback: Box<dyn SseCallback>,
        token: Arc<SseCancelToken>,
    ) {
        self.rt
            .block_on(self.run_sse(callback, token.subscribe()));
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

    async fn request(&self, command: Command, data: Vec<u8>) -> Result<Vec<u8>, NetworkError> {
        let (mut send, mut recv) = self
            .connection
            .open_bi()
            .await
            .map_err(|err| NetworkError::Request {
                reason: format!("поток не открылся: {err}"),
            })?;

        let mut packet = vec![command.id()];
        packet.extend(data);
        send.write_all(&packet)
            .await
            .map_err(|err| NetworkError::Request {
                reason: format!("запрос не ушёл: {err}"),
            })?;
        send.finish().map_err(|err| NetworkError::Request {
            reason: format!("запрос не закрылся: {err}"),
        })?;

        let mut response = Vec::new();
        let mut buf = vec![0; 4096];
        loop {
            match recv.read(&mut buf).await {
                Ok(Some(n)) if n > 0 => response.extend_from_slice(&buf[..n]),
                Ok(Some(_)) | Ok(None) => return Ok(response),
                Err(err) => {
                    return Err(NetworkError::Request {
                        reason: format!("ответ не дочитан: {err}"),
                    })
                }
            }
        }
    }

    async fn run_sse(&self, callback: Box<dyn SseCallback>, mut cancel: watch::Receiver<bool>) {
        let active = ACTIVE_SSE_SUBSCRIPTIONS.fetch_add(1, Ordering::SeqCst) + 1;
        info!("sse: subscribe start, active={active}");

        loop {
            if *cancel.borrow() {
                break;
            }

            match self.connection.open_bi().await {
                Ok((mut send, mut recv)) => {
                    if send.write_all(&[Command::SubscribeSse.id()]).await.is_ok() {
                        let _ = send.finish();
                        let mut buf = vec![0; 1024];
                        loop {
                            if *cancel.borrow() {
                                let remaining =
                                    ACTIVE_SSE_SUBSCRIPTIONS.fetch_sub(1, Ordering::SeqCst) - 1;
                                info!("sse: subscribe stop, active={remaining}");
                                return;
                            }
                            tokio::select! {
                                biased;
                                changed = cancel.changed() => {
                                    if changed.is_ok() || *cancel.borrow() {
                                        let remaining = ACTIVE_SSE_SUBSCRIPTIONS
                                            .fetch_sub(1, Ordering::SeqCst)
                                            - 1;
                                        info!("sse: subscribe stop, active={remaining}");
                                        return;
                                    }
                                }
                                result = recv.read(&mut buf) => {
                                    match result {
                                        Ok(Some(n)) if n > 0 => {
                                            let msg = String::from_utf8_lossy(&buf[..n]).into_owned();
                                            callback.on_event(msg);
                                        }
                                        _ => break,
                                    }
                                }
                            }
                        }
                    }
                }
                Err(_) => {}
            }

            if cancelled_sleep(&mut cancel, Duration::from_secs(5)).await {
                break;
            }
        }

        let remaining = ACTIVE_SSE_SUBSCRIPTIONS.fetch_sub(1, Ordering::SeqCst) - 1;
        info!("sse: subscribe stop, active={remaining}");
    }
}

async fn cancelled_sleep(cancel: &mut watch::Receiver<bool>, duration: Duration) -> bool {
    if *cancel.borrow() {
        return true;
    }
    tokio::select! {
        biased;
        changed = cancel.changed() => {
            let _ = changed;
            true
        }
        _ = tokio::time::sleep(duration) => *cancel.borrow(),
    }
}
