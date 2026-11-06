use std::f32::consts::PI;

use iced::mouse;
use iced::widget::canvas::{self, Canvas, Geometry, Path, Stroke};
use iced::{Color, Element, Point, Rectangle, Renderer, Size, Theme, Vector};

use crate::app::{App, Message};

pub fn view<'a>(app: &'a App, color: Color) -> Element<'a, Message> {
    let bars = if app.settings.show_visualizer {
        if app.smoothed_bands.is_empty() && !app.bands.is_empty() {
            smooth_preview(&app.bands)
        } else {
            app.smoothed_bands.clone()
        }
    } else {
        Vec::new()
    };
    let alpha = if app.settings.show_visualizer { 0.3 } else { 0.0 };
    Canvas::new(Visualizer {
        bands: bars,
        color: Color { a: alpha, ..color },
        ring: app.settings.visualizer_ring,
    })
    .width(iced::Length::Fill)
    .height(iced::Length::Fill)
    .into()
}

pub fn smooth_preview(bands: &[f64]) -> Vec<f32> {
    let keep = ((bands.len() as f32) * 0.3) as usize;
    let src = if bands.len() > keep {
        &bands[..bands.len() - keep]
    } else {
        bands
    };
    let total = 180usize;
    let mut out = Vec::with_capacity(total);
    for i in 0..total {
        let t = if total <= 1 { 0.0 } else { i as f32 / (total - 1) as f32 };
        let curved = t.powf(0.45);
        let exact = curved * (src.len().saturating_sub(1) as f32);
        let lo = (exact.floor() as usize).min(src.len().saturating_sub(1));
        let hi = (lo + 1).min(src.len().saturating_sub(1));
        let frac = exact - lo as f32;
        let v = if src.is_empty() {
            0.0
        } else {
            src[lo] as f32 + (src[hi] as f32 - src[lo] as f32) * frac
        };
        out.push(v.max(0.0));
    }
    out
}

pub fn smooth_bands(current: &mut Vec<f32>, bands: &[f64]) {
    let target = smooth_preview(bands);
    if current.len() != target.len() {
        *current = target;
        return;
    }
    for (c, t) in current.iter_mut().zip(target.iter()) {
        if *t > *c {
            *c += (*t - *c) * 0.7;
        } else {
            *c += (*t - *c) * 0.18;
        }
    }
}

struct Visualizer {
    bands: Vec<f32>,
    color: Color,
    ring: bool,
}

impl canvas::Program<Message> for Visualizer {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        if self.bands.is_empty() || self.color.a <= 0.0 {
            return vec![frame.into_geometry()];
        }
        if self.ring {
            draw_ring(&mut frame, &self.bands, self.color, bounds.size());
        } else {
            draw_bars(&mut frame, &self.bands, self.color, bounds.size());
        }
        vec![frame.into_geometry()]
    }
}

fn draw_bars(frame: &mut canvas::Frame, bands: &[f32], color: Color, size: Size) {
    let total = 180usize.min(bands.len());
    if total == 0 {
        return;
    }
    let half = total / 2;
    let mut values: Vec<f32> = bands[..half].iter().copied().rev().collect();
    values.extend_from_slice(&bands[..(total - half)]);
    let total = values.len();
    let gap = 1.5;
    let width = size.width - gap * (total.saturating_sub(1) as f32);
    let band_w = (width / total as f32).max(1.0);
    let center_y = size.height / 2.0;
    for (i, amp) in values.iter().enumerate() {
        let loud = amp.max(0.0).powf(0.4);
        let h = (loud * size.height).clamp(1.0, size.height);
        let x = i as f32 * (band_w + gap);
        let y = center_y - h / 2.0;
        frame.fill_rectangle(Point::new(x, y), Size::new(band_w.max(1.0), h), color);
    }
}

fn draw_ring(frame: &mut canvas::Frame, bands: &[f32], color: Color, size: Size) {
    let total = 180usize.min(bands.len());
    if total == 0 {
        return;
    }
    let side = size.width.min(size.height);
    let center = Point::new(size.width / 2.0, size.height / 2.0);
    let inner = side * 0.24;
    let outer = side * 0.5;
    let max_len = (outer - inner).max(0.0);
    let spacing = (2.0 * PI * inner) / total as f32;
    let thickness = (spacing * 0.5).max(1.0);
    let stroke = Stroke::default().with_color(color).with_width(thickness);
    for (i, amp) in bands[..total].iter().enumerate() {
        let t = i as f32 / (total.saturating_sub(1).max(1) as f32);
        let freq_w = 1.0 + 0.3 * t.powf(1.5);
        let loud = amp.max(0.0).powf(0.4) * freq_w;
        let len = (loud * max_len).clamp(1.0, max_len);
        let angle = ((i as f32 + 0.5) * 360.0 / total as f32 - 90.0) * PI / 180.0;
        let dir = Vector::new(angle.cos(), angle.sin());
        let start = Point::new(center.x + dir.x * inner, center.y + dir.y * inner);
        let end = Point::new(center.x + dir.x * (inner + len), center.y + dir.y * (inner + len));
        frame.stroke(&Path::line(start, end), stroke);
    }
}
