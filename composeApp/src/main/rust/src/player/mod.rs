mod audioconverter;
mod converter;
mod decode;
mod decoder;
mod fft_processor;
mod file_scanner;
mod output;
mod player_core;
mod queue;
mod quic_source;
mod spectrum;

#[cfg(target_os = "android")]
mod android_context;

pub use converter::FrameConverter;
pub use decoder::{cover_of, PcmSpec, TrackDecoder};
pub use file_scanner::{scan_music_folder, TrackData};
pub use output::{AudioOutput, DEVICE_BUFFER_FRAMES};
pub use player_core::{
    output_devices, track_cover, BandsListener, Player, PlayerError, PlayerEvent, PlayerListener,
    PlayerSnapshot, PlayerState,
};
pub use queue::RepeatMode;
pub use quic_source::QuicSource;
