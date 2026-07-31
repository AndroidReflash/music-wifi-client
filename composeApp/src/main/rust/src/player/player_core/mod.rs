//! Плеер: очередь, состояние и команды.
//!
//! Управляющий поток живёт здесь, а всё, что умеет ждать, — в [`super::decode`]. Ни
//! один вызов отсюда не дожидается декодера, поэтому пауза, стоп и смена дорожки
//! срабатывают и при молчащем сервере.

mod api;
mod error;
mod worker;

pub use api::{
    output_devices, track_cover, BandsListener, Player, PlayerEvent, PlayerListener, PlayerSnapshot,
    PlayerState,
};
pub use error::PlayerError;
pub use worker::Incoming;
