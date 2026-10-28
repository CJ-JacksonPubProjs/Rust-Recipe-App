use iced::widget::{button, column, scrollable, text, text_editor, text_input};
use iced::{Element, Length};

use crate::app::{App, Message};

impl App {
    pub fn create_view(&self) -> Element<'_, Message> {
        let can_submit = self.db.is_some() && !self.new_name.trim().is_empty();

        let mut form = column![
            text("New recipe").size(26),
            text("Name").size(14),
            text_input("Recipe name", &self.new_name)
                .on_input(Message::NameEdited)
                .padding(8),
            text("Ingredients").size(14),
            text_editor(&self.new_ingredients)
                .placeholder("One per line")
                .on_action(Message::IngredientsEdited)
                .height(Length::Fixed(160.0)),
            text("Directions").size(14),
            text_editor(&self.new_directions)
                .placeholder("Steps...")
                .on_action(Message::DirectionsEdited)
                .height(Length::Fixed(220.0)),
            text("Tags").size(14),
            text_input("e.g. vegan, dessert", &self.new_tags)
                .on_input(Message::NewTagsEdited)
                .padding(8),
            button(text("Save recipe")).on_press_maybe(can_submit.then_some(Message::SubmitRecipe)),
        ]
        .spacing(8)
        .padding(20);

        if let Some(status) = &self.status {
            form = form.push(text(status.as_str()).size(14));
        }

        scrollable(form).width(Length::Fill).into()
    }
}