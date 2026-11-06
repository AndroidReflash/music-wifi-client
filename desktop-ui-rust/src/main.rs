mod app;
mod platform;
mod repository;
pub mod settings;
mod ui;

fn main() -> iced::Result {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    app::run()
}
