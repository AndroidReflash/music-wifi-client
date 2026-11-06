use iced::widget::{container, text};
use iced::{Element, Length, Size};

fn main() -> iced::Result {
    iced::application("musicwifi", update, view)
        .window(iced::window::Settings {
            size: Size::new(900.0, 600.0),
            min_size: Some(Size::new(900.0, 600.0)),
            ..Default::default()
        })
        .theme(|_| iced::Theme::Dark)
        .centered()
        .run()
}

#[derive(Debug, Default)]
struct State;

#[derive(Debug, Clone)]
enum Message {}

fn update(_state: &mut State, message: Message) {
    match message {}
}

fn view(_state: &State) -> Element<'_, Message> {
    container(text("musicwifi-rust"))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
