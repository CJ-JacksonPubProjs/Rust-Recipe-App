use iced::widget::{column, container, scrollable, text, text_input};
use iced::{Element, Length};
use rusqlite::{Connection, Result as SqlResult};

const MAX_RESULTS: usize = 50;

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
        .title("PantryMatch")
        .run()
}

struct App {
    recipes: Vec<Recipe>,
    search: String,
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
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    SearchChanged(String),
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::SearchChanged(value) => self.search = value,
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let query = self.search.to_lowercase();

        let matches: Vec<&Recipe> = self
            .recipes
            .iter()
            .filter(|r| {
                query.is_empty()
                    || r.title.to_lowercase().contains(&query)
                    || r.ingredients.to_lowercase().contains(&query)
            })
            .take(MAX_RESULTS)
            .collect();

        let search_box = text_input("Search by title or ingredient...", &self.search)
            .on_input(Message::SearchChanged)
            .padding(10)
            .size(16);

        let mut list = column![].spacing(16).padding(20);

        if self.search.is_empty() {
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
                column![
                    text(format!("#{} — {}", recipe.id, recipe.title)).size(22),
                    text(format!("Ingredients: {}", recipe.ingredients)).size(14),
                    text(format!("Instructions: {}", recipe.instructions)).size(14),
                ]
                .spacing(4),
            );
        }

        container(column![search_box, scrollable(list)].spacing(10).padding(20))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
 
