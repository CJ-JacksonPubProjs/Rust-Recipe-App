use iced::widget::{Checkbox, checkbox, column, container, scrollable, text, text_input, button, image, row};
use iced::{Element, Length, Theme};
use rusqlite::{Connection, Result as SqlResult};
use std::collections::HashSet;

const MAX_RESULTS: usize = 50;
const COMMON_INGREDIENTS: &[&str] = &[
    "chicken", "beef", "pork", "egg", "milk", "butter",
    "garlic", "onion", "tomato", "cheese", "flour", "rice",
];

#[derive(Debug, Clone)]
struct Recipe {
    id: i32,
    title: String,
    ingredients: String,
    instructions: String,
}

fn load_recipes(path: &str) -> SqlResult<Vec<Recipe>> {
    let conn = Connection::open(path)?;
    let mut stmt = conn.prepare("SELECT id, title, ingredients, instructions FROM recipes")?;

    let rows = stmt.query_map([], |row| {
        Ok(Recipe {
            id: row.get(0)?,
            title: row.get(1)?,
            ingredients: row.get(2)?,
            instructions: row.get(3)?,
        })
    })?;

    rows.collect()
}

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("Rust Recipe ProjectV")
        .theme(Theme::TokyoNight )
        .run()
}

struct App {
    recipes: Vec<Recipe>,
    search: String,
    show_advanced: bool,
    included: HashSet<String>,
    excluded: HashSet<String>,
    selected: Option<i32>,
}

impl App {
    fn new() -> Self {
        let recipes = load_recipes("13k-recipes.db").unwrap_or_else(|err| {
            eprintln!("Failed to load recipes: {err}");
            Vec::new()
        });

        Self {
            recipes,
            search: String::new(),
            show_advanced: false,
            included: HashSet::new(),
            excluded: HashSet::new(),
            selected: None,            
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    SearchChanged(String),
    ToggleAdvanced,
    IncludeToggled(String, bool),
    ExcludeToggled(String, bool),
    RecipeSelected(i32),
    BackToResults,
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::SearchChanged(value) => {
            self.search = value;
            self.selected = None;
            }
            Message::ToggleAdvanced => self.show_advanced = !self.show_advanced,
            Message::IncludeToggled(name, checked) => {
                if checked {
                    self.excluded.remove(&name);
                    self.included.insert(name);
                } else {
                    self.included.remove(&name);
                }
            }
            Message::ExcludeToggled(name, checked) => {
                if checked {
                    self.included.remove(&name);
                    self.excluded.insert(name);
                } else {
                    self.excluded.remove(&name);
                }
            }
            Message::RecipeSelected(id) => self.selected = Some(id),
            Message::BackToResults => self.selected = None,
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let query = self.search.to_lowercase();

        let arrow_icon = image(image::Handle::from_path("assets/arrow.png"))
            .width(Length::Fixed(48.0))
            .height(Length::Fixed(48.0));

        let advanced_toggle = button(arrow_icon)
            .on_press(Message::ToggleAdvanced)
            .style(button::text);

        let matches: Vec<&Recipe> = self
            .recipes
            .iter()
            .filter(|r| self.recipe_matches(r, &query))
            .take(MAX_RESULTS)
            .collect();

        let search_box = text_input("Search by title or ingredient...", &self.search)
            .on_input(Message::SearchChanged)
            .padding(10)
            .size(16);

        let search_row = row![search_box, advanced_toggle].spacing(10);

        let mut list = column![].spacing(16).padding(20);

        if self.search.is_empty() && self.included.is_empty() && self.excluded.is_empty() {
            list = list.push(
                text(format!(
                    "{} recipes loaded — type to search ({} shown)",
                    self.recipes.len(),
                    matches.len()
                ))
                .size(14),
            );
        } else {
            list = list.push(text(format!("{} matches", matches.len())).size(14));
        }

        for recipe in matches {
            list = list.push(
            button(text(recipe.title.as_str()).size(18))
                .on_press(Message::RecipeSelected(recipe.id))
                .width(Length::Fill)
                .style(button::secondary),
            );
        }
        
        let mut body = row![].spacing(10);
        if self.show_advanced {
            body = body.push(self.advanced_panel());
        }
        let results: Element<'_, Message> = match self.selected {
            Some(id) => self.recipe_detail(id),
            None => scrollable(list).width(Length::Fill).into(),
        };
        body = body.push(results);

        container(column![search_row, body].spacing(10).padding(20))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn recipe_matches(&self, recipe: &Recipe, query: &str) -> bool {
        let title = recipe.title.to_lowercase();
        let ingredients = recipe.ingredients.to_lowercase();

        let text_ok = query.is_empty() || title.contains(query) || ingredients.contains(query);
        let included_ok = self.included.iter().all(|i| ingredients.contains(i.as_str()));
        let excluded_ok = !self.excluded.iter().any(|e| ingredients.contains(e.as_str()));

        text_ok && included_ok && excluded_ok
    }

    fn advanced_panel(&self) -> Element<'_, Message> {
        let mut rows = column![text("Advanced search").size(18)].spacing(8);

        for &ingredient in COMMON_INGREDIENTS {
            let include = Checkbox::new(self.included.contains(ingredient))
                .label("Include")
                .on_toggle(move |on| Message::IncludeToggled(ingredient.to_string(), on));

            let exclude: Checkbox<'_, Message> = Checkbox::new(self.excluded.contains(ingredient))
                .label("Exclude")
                .on_toggle(move |on| Message::ExcludeToggled(ingredient.to_string(), on));

            rows = rows.push(
            column![text(ingredient).size(16), row![include, exclude].spacing(16)].spacing(2),
            );
        }

        container(scrollable(rows))
            .padding(10)
            .width(Length::Fixed(260.0))
            .style(container::bordered_box)
            .into()
    }

    fn recipe_detail(&self, id: i32) -> Element<'_, Message> {
        let back = button(text("Back to results")).on_press(Message::BackToResults);

        let content = match self.recipes.iter().find(|r| r.id == id) {
            Some(recipe) => column![
                text(recipe.title.as_str()).size(26),
                text("Ingredients").size(18),
                text(recipe.ingredients.as_str()).size(14),
                text("Instructions").size(18),
                text(recipe.instructions.as_str()).size(14),
            ]
            .spacing(10)
            .width(Length::Fill),
            None => column![text("Recipe not found")],
        };

        scrollable(column![back, content].spacing(16).padding(20))
            .width(Length::Fill)
            .into()
    }
}
 
