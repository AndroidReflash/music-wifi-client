mod player;
mod network;
mod offline;

uniffi::setup_scaffolding!();

pub use network::{Command, NetworkError, QuicSession, SseCallback};
pub use offline::{FolderWatcher, FolderWatcherCallback};
pub use player::{
    cover_of, output_devices, track_cover, AudioOutput, BandsListener, FrameConverter, PcmSpec,
    Player, PlayerError, PlayerEvent, PlayerListener, PlayerSnapshot, PlayerState, QuicSource,
    RepeatMode, TrackData, TrackDecoder, DEVICE_BUFFER_FRAMES,
};

// логи

// android
#[no_mangle]
pub extern "C" fn JNI_OnLoad(_: *mut std::ffi::c_void, _: *mut std::ffi::c_void) -> i32 {
    #[cfg(target_os = "android")]
    {
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Debug)
                .with_tag("RustPlayer"),
        );
    }
    0x00010006 // JNI_VERSION_1_6
}

// десктоп
#[uniffi::export]
pub fn init_rust_logging() {
    use env_logger::{Builder, Env};
        use log::LevelFilter;

        // Читаем RUST_LOG из окружения, а если её нет — ставим "debug"
        let env = Env::default().default_filter_or("debug");

        let _ = Builder::from_env(env)
            .filter_level(LevelFilter::Debug) // Явно разрешаем Debug
            .target(env_logger::Target::Stdout)
            .try_init();

    log::info!("Rust logger initialized!");
}