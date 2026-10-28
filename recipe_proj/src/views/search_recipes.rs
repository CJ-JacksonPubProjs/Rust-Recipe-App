use crate::db_backend::Recipe;
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Length};

use crate::app::{parse_terms, App, Filters, Message};
use crate::views::recipe_detail;

pub const MAX_RESULTS: usize = 50;
pub const RESULTS_ID: &str = "results";

impl App {
    pub fn search_view(&self) -> Element<'_, Message> {
        let filters = Filters {
            query: self.search.to_lowercase(),
            include: parse_terms(&self.include_input),
            exclude: parse_terms(&self.exclude_input),
            tags: parse_terms(&self.tags_input),
        };
        let all_matches: Vec<&Recipe> = self
            .recipes
            .iter()
            .filter(|r| filters.matches(r))
            .collect();

        let advanced_toggle = button("advanced search")
            .on_press(Message::ToggleAdvanced)
            .style(button::secondary);

        let total_matches = all_matches.len();

        let start = self.page * MAX_RESULTS;
        let has_next = start + MAX_RESULTS < total_matches;
        let total_pages = total_matches.div_ceil(MAX_RESULTS).max(1);

        let page_items: Vec<&Recipe> = all_matches
            .into_iter()
            .skip(start)
            .take(MAX_RESULTS)
            .collect();

        let search_box = text_input("Search by title or ingredient...", &self.search)
            .on_input(Message::SearchChanged)
            .padding(10)
            .size(16);

        let search_row = row![search_box, advanced_toggle].spacing(10);

        let mut list = column![].spacing(16).padding(20);

        if self.loading {
            list = list.push(text("Loading recipes...").size(14));
        } else if let Some(err) = &self.error {
            list = list.push(text(format!("Failed to load recipes: {err}")).size(14));
        } else if filters.is_empty() {
            list = list.push(
                text(format!(
                    "{} recipes loaded — type to search ({} shown)",
                    self.recipes.len(),
                    page_items.len()
                ))
                .size(14),
            );
        } else {
            list = list.push(text(format!("{} matches", total_matches)).size(14));
        }

        for recipe in page_items {
            list = list.push(
                button(text(recipe.name.as_str()).size(18))
                    .on_press(Message::RecipeSelected(recipe.id))
                    .width(Length::Fill)
                    .style(button::secondary),
            );
        }

        let nav = row![
            button(text("Previous")).on_press_maybe((self.page > 0).then_some(Message::Back)),
            text(format!("Page {} of {}", self.page + 1, total_pages)),
            button(text("Next")).on_press_maybe(has_next.then_some(Message::Next)),
        ]
        .spacing(10);

        let mut body = row![].spacing(10);

        let results: Element<'_, Message> = match self.selected {
            Some(id) => recipe_detail(&self.recipes, id),
            None => column![
                scrollable(list)
                    .id(RESULTS_ID)
                    .width(Length::Fill)
                    .height(Length::Fill),
                nav,
            ]
            .spacing(10)
            .width(Length::Fill)
            .into(),
        };

        body = body.push(results);
        if self.show_advanced {
            body = body.push(self.advanced_panel());
        }

        container(column![search_row, body].spacing(10).padding(20))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn advanced_panel(&self) -> Element<'_, Message> {
        let include = text_input("e.g. chicken, garlic", &self.include_input)
            .on_input(Message::IncludeChanged)
            .padding(8);

        let exclude = text_input("e.g. mushrooms, olives", &self.exclude_input)
            .on_input(Message::ExcludeChanged)
            .padding(8);

        let tags = text_input("e.g. vegan, dessert", &self.tags_input)
            .on_input(Message::TagsChanged)
            .padding(8);

        container(
            column![
                text("Advanced search").size(18),
                text("Must include").size(14),
                include,
                text("Must not include").size(14),
                exclude,
                text("Tags").size(14),
                tags,
            ]
            .spacing(8),
        )
        .padding(10)
        .width(Length::Fixed(260.0))
        .style(container::bordered_box)
        .into()
    }
}