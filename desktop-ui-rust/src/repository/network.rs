use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;

use iced::futures::channel::mpsc;
use player_core::{Command, QuicSession, SseCallback, SseCancelToken, TrackData};

#[derive(Clone, Debug)]
pub enum NetworkStatus {
    Disconnected,
    Connecting,
    Connected,
}

#[derive(Clone, Debug)]
pub enum NetworkNotice {
    Status {
        generation: u64,
        address: String,
        status: NetworkStatus,
        error: Option<String>,
    },
    Playlist {
        generation: u64,
        address: String,
        tracks: Vec<TrackData>,
    },
    SseRefresh {
        generation: u64,
        address: String,
    },
}

pub struct NetworkHandle {
    sender: mpsc::UnboundedSender<NetworkNotice>,
    notices: Arc<Mutex<mpsc::UnboundedReceiver<NetworkNotice>>>,
    generation: Arc<AtomicU64>,
    state: Arc<Mutex<NetworkState>>,
}

struct NetworkState {
    address: String,
    enabled: bool,
    cancel: Option<Arc<SseCancelToken>>,
    worker: Option<thread::JoinHandle<()>>,
    sessions: HashMap<String, Arc<QuicSession>>,
}

impl NetworkHandle {
    pub fn start() -> Self {
        let (sender, receiver) = mpsc::unbounded();
        Self {
            sender,
            notices: Arc::new(Mutex::new(receiver)),
            generation: Arc::new(AtomicU64::new(0)),
            state: Arc::new(Mutex::new(NetworkState {
                address: String::new(),
                enabled: false,
                cancel: None,
                worker: None,
                sessions: HashMap::new(),
            })),
        }
    }

    pub fn notices(&self) -> crate::app::ChannelStream<NetworkNotice> {
        crate::app::ChannelStream::new(self.notices.clone())
    }

    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::SeqCst)
    }

    pub fn apply(&self, enabled: bool, address: String) {
        let generation = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
        let address = address.trim().to_owned();

        let mut state = lock(&self.state);
        state.enabled = enabled;
        state.address = address.clone();
        if let Some(token) = state.cancel.take() {
            token.cancel();
        }
        state.worker.take();

        if !enabled || address.is_empty() {
            send_notice(
                &self.sender,
                NetworkNotice::Status {
                    generation,
                    address,
                    status: NetworkStatus::Disconnected,
                    error: if enabled {
                        Some("адрес сервера не задан".to_owned())
                    } else {
                        None
                    },
                },
            );
            return;
        }

        send_notice(
            &self.sender,
            NetworkNotice::Status {
                generation,
                address: address.clone(),
                status: NetworkStatus::Connecting,
                error: None,
            },
        );

        let sender = self.sender.clone();
        let sessions = state.sessions.clone();
        let state_arc = self.state.clone();
        let generation_arc = self.generation.clone();
        let spawned = thread::Builder::new()
            .name("network-connect".to_owned())
            .spawn(move || {
                run_connect(generation, address, sender, sessions, state_arc, generation_arc)
            });
        match spawned {
            Ok(handle) => state.worker = Some(handle),
            Err(err) => send_notice(
                &self.sender,
                NetworkNotice::Status {
                    generation,
                    address: lock(&self.state).address.clone(),
                    status: NetworkStatus::Disconnected,
                    error: Some(format!("поток сети не запустился: {err}")),
                },
            ),
        }
    }

    pub fn refresh_playlist(&self) {
        let generation = self.generation.load(Ordering::SeqCst);
        let (address, enabled, session) = {
            let state = lock(&self.state);
            (
                state.address.clone(),
                state.enabled,
                state.sessions.get(&state.address).cloned(),
            )
        };
        if !enabled || address.is_empty() {
            return;
        }
        let Some(session) = session else {
            self.apply(true, address);
            return;
        };

        let sender = self.sender.clone();
        let state_arc = self.state.clone();
        let generation_arc = self.generation.clone();
        let _ = thread::Builder::new()
            .name("network-refresh".to_owned())
            .spawn(move || {
                fetch_and_publish(
                    generation,
                    &address,
                    &session,
                    &sender,
                    &state_arc,
                    &generation_arc,
                );
            });
    }

    pub fn session_for(&self, address: &str) -> Option<Arc<QuicSession>> {
        let address = address.trim();
        if address.is_empty() {
            return None;
        }
        lock(&self.state).sessions.get(address).cloned()
    }

    pub fn stop(&self) {
        let _ = self.generation.fetch_add(1, Ordering::SeqCst);
        let mut state = lock(&self.state);
        state.enabled = false;
        if let Some(token) = state.cancel.take() {
            token.cancel();
        }
        state.worker.take();
    }
}

impl Drop for NetworkHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

fn run_connect(
    generation: u64,
    address: String,
    sender: mpsc::UnboundedSender<NetworkNotice>,
    sessions: HashMap<String, Arc<QuicSession>>,
    state_arc: Arc<Mutex<NetworkState>>,
    generation_arc: Arc<AtomicU64>,
) {
    if is_stale(generation, &generation_arc) {
        return;
    }

    let cached = sessions.get(&address).cloned();
    let session = match cached {
        Some(session) => Some(session),
        None => match QuicSession::connect(address.clone()) {
            Ok(session) => Some(session),
            Err(err) => {
                send_notice(
                    &sender,
                    NetworkNotice::Status {
                        generation,
                        address,
                        status: NetworkStatus::Disconnected,
                        error: Some(err.to_string()),
                    },
                );
                return;
            }
        },
    };

    let Some(session) = session else {
        return;
    };
    if is_stale(generation, &generation_arc) {
        return;
    }

    {
        let mut state = lock(&state_arc);
        if state.enabled && state.address == address && generation_arc.load(Ordering::SeqCst) == generation {
            state.sessions.insert(address.clone(), session.clone());
        } else {
            return;
        }
    }

    fetch_and_publish(
        generation,
        &address,
        &session,
        &sender,
        &state_arc,
        &generation_arc,
    );
    if is_stale(generation, &generation_arc) {
        return;
    }

    let token = SseCancelToken::new();
    {
        let mut state = lock(&state_arc);
        if !state.enabled || state.address != address || generation_arc.load(Ordering::SeqCst) != generation {
            token.cancel();
            return;
        }
        if let Some(old) = state.cancel.replace(token.clone()) {
            old.cancel();
        }
    }

    struct Sink {
        generation: u64,
        address: String,
        sender: mpsc::UnboundedSender<NetworkNotice>,
        generation_arc: Arc<AtomicU64>,
    }
    impl SseCallback for Sink {
        fn on_event(&self, _message: String) {
            if is_stale(self.generation, &self.generation_arc) {
                return;
            }
            send_notice(
                &self.sender,
                NetworkNotice::SseRefresh {
                    generation: self.generation,
                    address: self.address.clone(),
                },
            );
        }
    }

    session.subscribe_sse_with_cancel(
        Box::new(Sink {
            generation,
            address: address.clone(),
            sender: sender.clone(),
            generation_arc: generation_arc.clone(),
        }),
        token,
    );

    if !is_stale(generation, &generation_arc) {
        let mut state = lock(&state_arc);
        if state.address == address && generation_arc.load(Ordering::SeqCst) != generation {
            // superseded by newer generation; keep cache
        } else if state.address == address {
            state.sessions.remove(&address);
            state.cancel.take();
            send_notice(
                &sender,
                NetworkNotice::Status {
                    generation,
                    address,
                    status: NetworkStatus::Disconnected,
                    error: Some("подписка на обновления завершилась".to_owned()),
                },
            );
        }
    }
}

fn fetch_and_publish(
    generation: u64,
    address: &str,
    session: &Arc<QuicSession>,
    sender: &mpsc::UnboundedSender<NetworkNotice>,
    state_arc: &Arc<Mutex<NetworkState>>,
    generation_arc: &Arc<AtomicU64>,
) {
    if is_stale(generation, generation_arc) {
        return;
    }
    match session.send_command_result(Command::GetPlaylist, Vec::new()) {
        Ok(bytes) => {
            if is_stale(generation, generation_arc) {
                return;
            }
            let text = String::from_utf8_lossy(&bytes);
            let tracks = parse_playlist(&text);
            send_notice(
                sender,
                NetworkNotice::Status {
                    generation,
                    address: address.to_owned(),
                    status: NetworkStatus::Connected,
                    error: None,
                },
            );
            send_notice(
                sender,
                NetworkNotice::Playlist {
                    generation,
                    address: address.to_owned(),
                    tracks,
                },
            );
        }
        Err(err) => {
            if is_stale(generation, generation_arc) {
                return;
            }
            let mut state = lock(state_arc);
            if state.address == address {
                state.sessions.remove(address);
            }
            send_notice(
                sender,
                NetworkNotice::Status {
                    generation,
                    address: address.to_owned(),
                    status: NetworkStatus::Disconnected,
                    error: Some(err.to_string()),
                },
            );
        }
    }
}

fn parse_playlist(text: &str) -> Vec<TrackData> {
    let mut tracks = Vec::new();
    for line in text.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let song_path = line.trim_start_matches('/');
        let file_name = song_path.rsplit('/').next().unwrap_or(song_path);
        let track_name = file_name
            .rsplit_once('.')
            .map(|(name, _)| name)
            .unwrap_or(file_name)
            .to_owned();
        let track_index = tracks.len() as i32;
        tracks.push(TrackData {
            track_url: song_path.to_owned(),
            track_name,
            track_index,
            is_local: false,
        });
    }
    tracks
}

fn is_stale(generation: u64, current: &Arc<AtomicU64>) -> bool {
    current.load(Ordering::SeqCst) != generation
}

fn send_notice(sender: &mpsc::UnboundedSender<NetworkNotice>, notice: NetworkNotice) {
    let _ = sender.unbounded_send(notice);
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|error| error.into_inner())
}
