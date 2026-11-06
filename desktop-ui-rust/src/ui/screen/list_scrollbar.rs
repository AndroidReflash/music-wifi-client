use iced::mouse;
use iced::widget::canvas::{self, Canvas, Event, Frame, Geometry, Path};
use iced::{Color, Element, Point, Rectangle, Renderer, Size, Theme};

use crate::app::Message;

pub fn list_scrollbar<'a>(
    viewport: Option<iced::widget::scrollable::Viewport>,
    interface: Color,
) -> Element<'a, Message> {
    Canvas::new(Program { viewport, interface })
        .width(12)
        .height(iced::Length::Fill)
        .into()
}

struct Program {
    viewport: Option<iced::widget::scrollable::Viewport>,
    interface: Color,
}

#[derive(Default)]
pub struct State {
    dragging: Option<f32>,
}

impl canvas::Program<Message> for Program {
    type State = State;

    fn update(
        &self,
        state: &mut Self::State,
        event: Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        let Some(viewport) = self.viewport else {
            return (canvas::event::Status::Ignored, None);
        };
        let content = viewport.content_bounds();
        let view = viewport.bounds();
        let track = bounds.height;
        if content.height <= view.height || view.height <= 0.0 || track <= 0.0 {
            return (canvas::event::Status::Ignored, None);
        }
        let ratio = (view.height / content.height).clamp(0.0, 1.0);
        let scroller_h = (track * ratio).clamp(12.0, track);
        let travel = (track - scroller_h).max(0.0);
        let rel = viewport.relative_offset().y.clamp(0.0, 1.0);

        let relative_at = |pos_y: f32, grab: f32| {
            if travel <= 0.0 {
                0.0
            } else {
                ((pos_y - bounds.y - scroller_h * grab) / travel).clamp(0.0, 1.0)
            }
        };

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position() {
                    if bounds.contains(pos) {
                        let scroller_y = bounds.y + rel * travel;
                        let grab = if pos.y >= scroller_y && pos.y <= scroller_y + scroller_h {
                            (pos.y - scroller_y) / scroller_h
                        } else {
                            0.5
                        };
                        state.dragging = Some(grab);
                        return (
                            canvas::event::Status::Captured,
                            Some(Message::ListScrollTo(relative_at(pos.y, grab))),
                        );
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(grab) = state.dragging {
                    if let Some(pos) = cursor.position() {
                        return (
                            canvas::event::Status::Captured,
                            Some(Message::ListScrollTo(relative_at(pos.y, grab))),
                        );
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if state.dragging.take().is_some() {
                    return (canvas::event::Status::Captured, None);
                }
            }
            _ => {}
        }
        (canvas::event::Status::Ignored, None)
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let rail = Path::rectangle(Point::ORIGIN, bounds.size());
        frame.fill(&rail, Color { a: 0.08, ..self.interface });

        if let Some(viewport) = self.viewport {
            let content = viewport.content_bounds();
            let view = viewport.bounds();
            if content.height > view.height && view.height > 0.0 && bounds.height > 0.0 {
                let ratio = (view.height / content.height).clamp(0.0, 1.0);
                let scroller_h = (bounds.height * ratio).clamp(12.0, bounds.height);
                let travel = (bounds.height - scroller_h).max(0.0);
                let rel = viewport.relative_offset().y.clamp(0.0, 1.0);
                let y = rel * travel;
                let scroller = Path::rectangle(
                    Point::new(1.0, y),
                    Size::new((bounds.width - 2.0).max(1.0), scroller_h),
                );
                frame.fill(&scroller, Color { a: 0.5, ..self.interface });
            }
        }

        vec![frame.into_geometry()]
    }
}
