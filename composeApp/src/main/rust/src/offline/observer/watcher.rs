use crate::player::{scan_music_folder, TrackData};
use log::info;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio::time::timeout;

#[derive(uniffi::Object)]
pub struct FolderWatcher {
    /// Рантайм tokio живёт, пока жив наблюдатель: его деструктор останавливает рантайм и
    /// тем самым дожидается задач. Отдельной задачи-цикла после `stop`/`drop` быть не должно.
    rt: Runtime,
    tx: mpsc::Sender<WatcherCommand>,
    handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
    stopped: AtomicBool,
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
            let mut watcher = match RecommendedWatcher::new(
                move |res: Result<Event, notify::Error>| {
                    if let Ok(event) = res {
                        if matches!(
                            event.kind,
                            EventKind::Create(_) | EventKind::Remove(_) | EventKind::Modify(_)
                        ) {
                            let _ = tx_for_watcher.try_send(WatcherCommand::Scan);
                        }
                    }
                },
                Config::default().with_poll_interval(Duration::from_millis(500)),
            ) {
                Ok(watcher) => watcher,
                Err(err) => {
                    info!("folder watcher: create failed: {err}");
                    return;
                }
            };

            if let Err(err) = watcher.watch(&path, RecursiveMode::NonRecursive) {
                info!("folder watcher: watch failed: {err}");
                return;
            }

            let mut debounce_timer = None::<tokio::time::Instant>;

            loop {
                tokio::select! {
                    biased;
                    cmd = rx.recv() => {
                        match cmd {
                            Some(WatcherCommand::Scan) => {
                                debounce_timer = Some(
                                    tokio::time::Instant::now() + Duration::from_millis(300),
                                );
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

        Arc::new(Self {
            rt,
            tx,
            handle: Mutex::new(Some(handle)),
            stopped: AtomicBool::new(false),
        })
    }

    pub fn stop(&self) {
        if self.stopped.swap(true, Ordering::SeqCst) {
            return;
        }
        let _ = self.tx.try_send(WatcherCommand::Stop);
        if let Ok(mut handle_guard) = self.handle.lock() {
            if let Some(handle) = handle_guard.take() {
                let _ = self.rt.block_on(handle);
            }
        }
        info!("folder watcher: stopped");
    }
}

impl Drop for FolderWatcher {
    fn drop(&mut self) {
        if !self.stopped.swap(true, Ordering::SeqCst) {
            let _ = self.tx.try_send(WatcherCommand::Stop);
            if let Ok(mut handle_guard) = self.handle.lock() {
                if let Some(handle) = handle_guard.take() {
                    let _ = self.rt.block_on(handle);
                }
            }
            info!("folder watcher: dropped");
        }
    }
}
