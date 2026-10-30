use crate::db_backend::Recipe;
use iced::widget::{button, column, row, scrollable, text};
use iced::{Element, Length};

use crate::app::{Message, RecipeOrigin};

/// Turns `["a", "b, c", "d"]` into "a\nb, c\nd".
/// Falls back to the original text if it isn't in that list format.
pub fn format_list(raw: &str) -> String {
    let trimmed = raw.trim();
    let Some(inner) = trimmed
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
    else {
        return raw.to_string();
    };

    let mut items: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut chars = inner.chars();

    while let Some(c) = chars.next() {
        match quote {
            // escaped character inside a quoted item, keep it literally
            Some(_) if c == '\\' => {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }
            // closing quote ends the item
            Some(q) if c == q => {
                quote = None;
                items.push(std::mem::take(&mut current));
            }
            Some(_) => current.push(c),
            // opening quote
            None if c == '"' || c == '\'' => quote = Some(c),
            // brackets, commas, and whitespace between items are dropped
            None => {}
        }
    }

    if items.is_empty() {
        raw.to_string()
    } else {
        items.join("\n")
    }
}

pub fn recipe_detail<'a>(recipes: &'a [Recipe], id: i32, origin: RecipeOrigin) -> Element<'a, Message> {
    let back = button(text("Back to results")).on_press(Message::BackToResults);
    let is_search = matches!(origin, RecipeOrigin::Search);

    let content = match recipes.iter().find(|r| r.id == id) {
        Some(recipe) => {
            let fmt = |s: &str| {
                if is_search {
                    format_list(s)
                } else {
                    s.to_string()
                }
            };

            // Action button depends on where user navigated from
            let action_button = match origin {
                RecipeOrigin::Search => {
                    button(text("Save to My Recipes")).on_press(Message::SaveRecipe(recipe.id))
                }
                RecipeOrigin::Saved => button(text("Delete from My Recipes"))
                    .on_press(Message::DeleteRecipe(recipe.id))
                    .style(button::danger),
            };

            let action_row = row![action_button].spacing(10);

            column![
                text(recipe.name.as_str()).size(26),
                text("Ingredients").size(18),
                text(fmt(&recipe.ingredients)).size(14),
                text("Instructions").size(18),
                text(fmt(&recipe.instructions)).size(14),
                text("Tags").size(18),
                text(recipe.tags.as_str()).size(14),
                action_row,
            ]
            .spacing(10)
            .width(Length::Fill)
        }
        None => column![text("Recipe not found")],
    };

    scrollable(column![back, content].spacing(16).padding(20))
        .width(Length::Fill)
        .into()
}