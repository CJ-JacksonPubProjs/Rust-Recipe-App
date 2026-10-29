use crate::db_backend::Recipe;
use iced::widget::{button, column, row, scrollable, text};
use iced::{Element, Length};

use crate::app::{Message, RecipeOrigin};

pub fn recipe_detail<'a>(recipes: &'a [Recipe], id: i32, origin: RecipeOrigin) -> Element<'a, Message> {
    let back = button(text("Back to results")).on_press(Message::BackToResults);

    let content = match recipes.iter().find(|r| r.id == id) {
        Some(recipe) => {
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
                text(recipe.ingredients.as_str()).size(14),
                text("Instructions").size(18),
                text(recipe.instructions.as_str()).size(14),
                text("Tags").size(18),
                text(recipe.tags.as_str()).size(14),
                action_row, // Bottom row with the dynamic action
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