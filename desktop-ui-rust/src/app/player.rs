use std::path::Path;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::task::{Context, Poll};
use std::thread;

use iced::futures::channel::mpsc;
use iced::futures::Stream;
use player_core::{
    BandsListener, FolderWatcher, FolderWatcherCallback, Player, PlayerError, PlayerEvent,
    PlayerListener, TrackData,
};

pub enum PlayerNotice {
    Ready(Arc<Player>),
    Failed(PlayerError),
    Event(PlayerEvent),
    Bands(Vec<f64>),
    Playlist(Vec<TrackData>),
    PlaylistFailed(String),
}

pub struct PlayerHandle {
    notices: Arc<Mutex<mpsc::UnboundedReceiver<PlayerNotice>>>,
    sender: mpsc::UnboundedSender<PlayerNotice>,
    watcher: Arc<Mutex<Option<Arc<FolderWatcher>>>>,
    watcher_generation: Arc<AtomicU64>,
}

impl PlayerHandle {
    pub fn start(folder_path: String) -> Self {
        let (sender, receiver) = mpsc::unbounded();
        let watcher = Arc::new(Mutex::new(None));
        let watcher_generation = Arc::new(AtomicU64::new(0));

        start_player(sender.clone());
        start_watcher(
            folder_path,
            sender.clone(),
            watcher.clone(),
            watcher_generation.clone(),
        );

        Self {
            notices: Arc::new(Mutex::new(receiver)),
            sender,
            watcher,
            watcher_generation,
        }
    }

    pub fn notices(&self) -> ChannelStream<PlayerNotice> {
        ChannelStream::new(self.notices.clone())
    }

    pub fn set_folder(&self, folder_path: String) {
        self.watcher_generation.fetch_add(1, Ordering::SeqCst);
        if let Some(watcher) = lock(&self.watcher).take() {
            watcher.stop();
        }
        start_watcher(
            folder_path,
            self.sender.clone(),
            self.watcher.clone(),
            self.watcher_generation.clone(),
        );
    }

    pub fn take_watcher(&self) -> Option<Arc<FolderWatcher>> {
        self.watcher_generation.fetch_add(1, Ordering::SeqCst);
        lock(&self.watcher).take()
    }

    pub fn stop(&self) {
        if let Some(watcher) = self.take_watcher() {
            watcher.stop();
        }
    }
}

impl Drop for PlayerHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

#[derive(Clone)]
pub struct ChannelStream<T> {
    receiver: Arc<Mutex<mpsc::UnboundedReceiver<T>>>,
}

impl<T> ChannelStream<T> {
    pub fn new(receiver: Arc<Mutex<mpsc::UnboundedReceiver<T>>>) -> Self {
        Self { receiver }
    }
}

impl<T> Stream for ChannelStream<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<T>> {
        let this = self.get_mut();
        let mut receiver = lock(&this.receiver);

        Pin::new(&mut *receiver).poll_next(cx)
    }
}

struct PlayerListenerSink {
    sender: mpsc::UnboundedSender<PlayerNotice>,
}

impl PlayerListener for PlayerListenerSink {
    fn on_event(&self, event: PlayerEvent) {
        send(&self.sender, PlayerNotice::Event(event));
    }
}

struct BandsListenerSink {
    sender: mpsc::UnboundedSender<PlayerNotice>,
}

impl BandsListener for BandsListenerSink {
    fn on_bands(&self, bands: Vec<f64>) {
        send(&self.sender, PlayerNotice::Bands(bands));
    }
}

struct FolderWatcherSink {
    sender: mpsc::UnboundedSender<PlayerNotice>,
}

impl FolderWatcherCallback for FolderWatcherSink {
    fn on_changed(&self, tracks: Vec<TrackData>) {
        send(&self.sender, PlayerNotice::Playlist(tracks));
    }
}

fn start_player(sender: mpsc::UnboundedSender<PlayerNotice>) {
    let thread_sender = sender.clone();

    let spawned = thread::Builder::new()
        .name("player-startup".to_owned())
        .spawn(move || {
            let result = Player::new(
                Box::new(PlayerListenerSink {
                    sender: thread_sender.clone(),
                }),
                Box::new(BandsListenerSink {
                    sender: thread_sender.clone(),
                }),
                None,
            );

            match result {
                Ok(player) => send(&thread_sender, PlayerNotice::Ready(player)),
                Err(error) => send(&thread_sender, PlayerNotice::Failed(error)),
            }
        });

    if let Err(error) = spawned {
        send(
            &sender,
            PlayerNotice::Failed(PlayerError::Gone {
                reason: format!("поток плеера не запустился: {error}"),
            }),
        );
    }
}

fn start_watcher(
    folder_path: String,
    sender: mpsc::UnboundedSender<PlayerNotice>,
    watcher: Arc<Mutex<Option<Arc<FolderWatcher>>>>,
    generation: Arc<AtomicU64>,
) {
    if folder_path.trim().is_empty() {
        return;
    }

    let current_generation = generation.load(Ordering::SeqCst);
    let thread_sender = sender.clone();

    let spawned = thread::Builder::new()
        .name("folder-watcher".to_owned())
        .spawn(move || {
            if !Path::new(&folder_path).is_dir() {
                send(
                    &thread_sender,
                    PlayerNotice::PlaylistFailed(format!("папка не найдена: {folder_path}")),
                );
                return;
            }

            let instance = FolderWatcher::new(
                folder_path,
                Box::new(FolderWatcherSink {
                    sender: thread_sender,
                }),
            );

            let mut slot = lock(&watcher);
            if generation.load(Ordering::SeqCst) == current_generation {
                *slot = Some(instance);
            } else {
                drop(slot);
                instance.stop();
            }
        });

    if let Err(error) = spawned {
        send(
            &sender,
            PlayerNotice::PlaylistFailed(format!(
                "поток наблюдения за папкой не запустился: {error}"
            )),
        );
    }
}

fn send(sender: &mpsc::UnboundedSender<PlayerNotice>, notice: PlayerNotice) {
    if sender.unbounded_send(notice).is_err() {
        return;
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|error| error.into_inner())
}
