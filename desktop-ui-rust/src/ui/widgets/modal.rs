use iced::widget::{center, container, mouse_area, opaque, stack, Space};
use iced::{Background, Border, Color, Element, Length};

use crate::app::Message;

pub fn modal<'a>(
    base: Element<'a, Message>,
    content: Element<'a, Message>,
    on_dismiss: Message,
) -> Element<'a, Message> {
    let backdrop = mouse_area(
        container(Space::new(Length::Fill, Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.6))),
                ..container::Style::default()
            }),
    )
    .on_press(on_dismiss);

    let dialog = center(
        container(content)
            .width(Length::Fixed(340.0))
            .max_height(460.0)
            .padding(12)
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::BLACK)),
                border: Border {
                    color: Color::WHITE,
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..container::Style::default()
            }),
    );

    stack![base, opaque(backdrop), opaque(dialog)].into()
}
