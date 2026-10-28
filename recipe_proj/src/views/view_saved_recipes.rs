use iced::widget::{button, column, scrollable, text};
use iced::{Element, Length};

use crate::app::{App, Message};
use crate::views::recipe_detail;

impl App {
    pub fn saved_view(&self) -> Element<'_, Message> {
        match self.selected_saved {
            Some(id) => recipe_detail(&self.my_recipes, id),
            None => {
                let mut list = column![text(format!("{} saved recipes", self.my_recipes.len()))
                    .size(14)]
                .spacing(16)
                .padding(20);

                // newest first
                for recipe in self.my_recipes.iter().rev() {
                    list = list.push(
                        button(text(recipe.name.as_str()).size(18))
                            .on_press(Message::RecipeSelected(recipe.id))
                            .width(Length::Fill)
                            .style(button::secondary),
                    );
                }

                scrollable(list)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
        }
    }
}