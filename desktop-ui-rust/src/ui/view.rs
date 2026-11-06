use iced::Element;

use crate::app::dialog::Dialog;
use crate::app::{App, Message};

use super::{dialogs, screen, widgets};

pub fn view(app: &App) -> Element<'_, Message> {
    let base = screen::layout(app);
    match &app.dialog {
        Some(Dialog::ChangeIp(state)) => {
            widgets::modal(base, dialogs::change_ip::view(app, state), Message::DialogDismiss)
        }
        Some(Dialog::Equalizer(state)) => {
            widgets::modal(base, dialogs::equalizer::view(app, state), Message::DialogDismiss)
        }
        Some(Dialog::FolderPicker(state)) => {
            widgets::modal(base, dialogs::folder_picker::view(app, state), Message::DialogDismiss)
        }
        Some(Dialog::ColorPicker(state)) => {
            widgets::modal(base, dialogs::color_picker::view(app, state), Message::ColorPickerCancel)
        }
        Some(Dialog::HotkeysCapture(state)) => {
            widgets::modal(base, dialogs::hotkeys::view(app, state), Message::DialogDismiss)
        }
        None => base,
    }
}
