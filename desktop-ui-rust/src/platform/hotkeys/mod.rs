pub mod wayland;
pub mod x11;

pub fn wayland_session() -> bool {
    std::env::var_os("WAYLAND_DISPLAY").is_some()
        || matches!(
            std::env::var("XDG_SESSION_TYPE"),
            Ok(value) if value.eq_ignore_ascii_case("wayland")
        )
}
