use iced::widget::{column, pick_list, scrollable, text};
use iced::{Element, Length, Theme};

use crate::app::{App, Message};

impl App {
    pub fn settings_view(&self) -> Element<'_, Message> {
        let theme_picker = pick_list(Theme::ALL, Some(&self.theme), Message::ThemeChanged)
            .placeholder("Choose a theme")
            .width(Length::Fixed(260.0));

        let content = column![
            text("Settings").size(26),
            text("Theme").size(14),
            theme_picker,
        ]
        .spacing(8)
        .padding(20);

        scrollable(content).width(Length::Fill).into()
    }
}