use iced::widget::{column, container, image, row, stack, text, Space};
use iced::{Background, Color, Element, Length};

use crate::app::{App, Message};

use super::{controls, cover, marquee, seek_slider, song_list, top_bar, visualizer};

pub fn layout<'a>(app: &'a App) -> Element<'a, Message> {
    let interface = interface_color(app);

    let top = top_bar::top_bar(app, interface);
    let left = song_list::song_list(app, interface);

    let cover_visible = app.settings.show_cover_image
        && matches!(
            (app.cover.as_ref(), app.current_track_index(), app.cover_for_index),
            (Some(_), Some(current), Some(for_index)) if for_index == current
        );
    let note_visible = !cover_visible && !app.settings.show_cover && !app.settings.show_visualizer;

    let square: Element<'a, Message> = {
        let mut layers = stack![];
        if note_visible {
            layers = layers.push(
                container(crate::ui::icons::note::note(interface))
                    .width(Length::Fill)
                    .height(Length::Fill),
            );
        }

        if cover_visible {
            if let (Some(handle), Some(current), Some(for_index)) =
                (app.cover.as_ref(), app.current_track_index(), app.cover_for_index)
            {
                if for_index == current {
                    layers = layers.push(
                        container(
                            image(handle.clone())
                                .content_fit(iced::ContentFit::Cover)
                                .width(Length::Fill)
                                .height(Length::Fill),
                        )
                        .padding(1)
                        .width(Length::Fill)
                        .height(Length::Fill),
                    );
                }
            }
        }

        if app.settings.show_cover {
            layers = layers.push(cover::view(
                interface,
                app.settings.cover_small,
                app.spin_deg,
            ));
        }

        layers = layers.push(visualizer::view(app, interface));

        container(layers)
            .width(Length::Fill)
            .height(Length::FillPortion(1))
            .into()
    };

    let mut right_col = column![
        square,
        marquee::marquee(app),
        controls::controls(app, interface),
        seek_slider::seek_slider(app, interface),
    ]
    .spacing(4)
    .padding(iced::Padding {
        top: 0.0,
        right: 10.0,
        bottom: 0.0,
        left: 10.0,
    })
    .width(Length::FillPortion(1))
    .height(Length::Fill)
    .align_x(iced::Alignment::Center);

    if let Some(error) = app
        .player_error
        .as_deref()
        .or(app.playlist_error.as_deref())
        .or(app.network_error.as_deref())
        .or(app.sound_error.as_deref())
    {
        right_col = right_col.push(
            text(error)
                .size(12)
                .color(Color::from_rgb(1.0, 0.45, 0.45)),
        );
    }

    let right = right_col;

    let divider = container(Space::new(1.0, Length::Fill))
        .width(1)
        .height(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(Background::Color(interface)),
            ..container::Style::default()
        });
    let body = row![left, divider, right].width(Length::Fill).height(Length::Fill);

    column![top, body]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn interface_color(app: &App) -> Color {
    let (r, g, b, a) = app.settings.theme_color.rgba();
    Color::from_rgba8(r, g, b, a as f32 / 255.0)
}
