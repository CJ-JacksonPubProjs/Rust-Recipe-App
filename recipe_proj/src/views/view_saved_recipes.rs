use iced::widget::{button, column, scrollable, text, text_input};
use iced::{Element, Length};

use crate::app::{App, Filters, Message, RecipeOrigin};
use crate::db_backend::Recipe;
use crate::views::recipe_detail;

impl App {
    pub fn saved_view(&self) -> Element<'_, Message> {
        match self.selected_saved {
            Some(id) => recipe_detail(&self.my_recipes, id, RecipeOrigin::Saved),
            None => {
                let filters = Filters {
                    query: self.saved_search.to_lowercase(),
                    include: Vec::new(),
                    exclude: Vec::new(),
                    tags: Vec::new(),
                };

                // newest first, then filtered
                let matches: Vec<&Recipe> = self
                    .my_recipes
                    .iter()
                    .rev()
                    .filter(|r| filters.matches(r))
                    .collect();

                let search_box =
                    text_input("Search saved recipes by title or ingredient...", &self.saved_search)
                        .on_input(Message::SavedSearchChanged)
                        .padding(10)
                        .size(16);

                let summary = if filters.is_empty() {
                    format!("{} saved recipes", self.my_recipes.len())
                } else {
                    format!("{} matches", matches.len())
                };

                let mut list = column![text(summary).size(14)].spacing(16);

                for recipe in matches {
                    list = list.push(
                        button(text(recipe.name.as_str()).size(18))
                            .on_press(Message::RecipeSelected(recipe.id))
                            .width(Length::Fill)
                            .style(button::secondary),
                    );
                }

                column![
                    search_box,
                    scrollable(list).width(Length::Fill).height(Length::Fill),
                ]
                .spacing(10)
                .padding(20)
                .into()
            }
        }
    }
}