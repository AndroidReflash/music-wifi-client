use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio::time::timeout;
use crate::player::{scan_music_folder, TrackData};

#[derive(uniffi::Object)]
pub struct FolderWatcher {
    /// Рантайм tokio лежит здесь только ради времени жизни: пока он жив, живёт и задача
    /// наблюдателя, а с его уходом она гаснет. Читать поле не надо — отсюда имя с
    /// подчёркиванием, иначе компилятор считает его неиспользуемым.
    _rt: Runtime,
    tx: mpsc::Sender<WatcherCommand>,
    handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

enum WatcherCommand {
    Scan,
    Stop,
}

#[uniffi::export(callback_interface)]
pub trait FolderWatcherCallback: Send + Sync {
    fn on_changed(&self, tracks: Vec<TrackData>);
}

#[uniffi::export]
impl FolderWatcher {
    #[uniffi::constructor]
    pub fn new(folder_path: String, callback: Box<dyn FolderWatcherCallback>) -> Arc<Self> {
        let rt = Runtime::new().expect("Failed to create Tokio runtime");
        let (tx, mut rx) = mpsc::channel::<WatcherCommand>(32);
        let path = PathBuf::from(&folder_path);
        let callback = Arc::new(callback);
        let folder_path_clone = folder_path.clone();

        let initial_tracks = scan_music_folder(folder_path_clone.clone());
        callback.on_changed(initial_tracks);

        let tx_for_watcher = tx.clone();
        let handle = rt.spawn(async move {
            let mut watcher = RecommendedWatcher::new(
                move |res: Result<Event, notify::Error>| {
                    if let Ok(event) = res {
                        if matches!(event.kind, EventKind::Create(_) | EventKind::Remove(_) | EventKind::Modify(_)) {
                            let _ = tx_for_watcher.try_send(WatcherCommand::Scan);
                        }
                    }
                },
                Config::default().with_poll_interval(Duration::from_millis(500)),
            ).expect("Failed to create watcher");

            watcher.watch(&path, RecursiveMode::NonRecursive).expect("Failed to watch folder");

            let mut debounce_timer = None::<tokio::time::Instant>;

            loop {
                tokio::select! {
                    biased;
                    cmd = rx.recv() => {
                        match cmd {
                            Some(WatcherCommand::Scan) => {
                                debounce_timer = Some(tokio::time::Instant::now() + Duration::from_millis(300));
                            }
                            Some(WatcherCommand::Stop) | None => {
                                break;
                            }
                        }
                    }
                    _ = async {
                        if let Some(deadline) = debounce_timer {
                            let now = tokio::time::Instant::now();
                            if now >= deadline {
                                debounce_timer = None;
                                let tracks = scan_music_folder(folder_path_clone.clone());
                                callback.on_changed(tracks);
                            } else {
                                let wait_duration = deadline - now;
                                let _ = timeout(wait_duration, tokio::time::sleep(wait_duration)).await;
                            }
                        } else {
                            tokio::time::sleep(Duration::from_secs(1)).await;
                        }
                    } => {}
                }
            }

            let _ = watcher.unwatch(&path);
        });

        Arc::new(Self { _rt: rt, tx, handle: Mutex::new(Some(handle)) })
    }

    pub fn stop(&self) {
        let _ = self.tx.try_send(WatcherCommand::Stop);
        if let Ok(mut handle_guard) = self.handle.lock() {
            if let Some(handle) = handle_guard.take() {
                handle.abort();
            }
        }
    }
}