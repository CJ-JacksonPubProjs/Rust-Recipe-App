use iced::widget::{column, container, scrollable, text, text_input, button, image, row};
use iced::{Element, Length, Theme, Task};
use iced::widget::operation::{self, RelativeOffset};
use rusqlite::{Connection, Result as SqlResult};
use std::collections::HashSet;

const MAX_RESULTS: usize = 50;

const RESULTS_ID: &str = "results";

fn scroll_to_top() -> Task<Message> {
    operation::snap_to(RESULTS_ID, RelativeOffset::START)
}

#[derive(Debug, Clone)]
struct Recipe {
    id: i32,
    name: String,
    ingredients: String,
    instructions: String,
    tags: String,
    // lowercase copies, used only for searching
    name_lc: String,
    ingredients_lc: String,
    tags_lc: String,
}


fn load_recipes(path: &str) -> SqlResult<Vec<Recipe>> {
    let conn = Connection::open(path)?;
    let mut stmt =
        conn.prepare("SELECT id, name, ingredients, directions, tags FROM recipes")?;

    let rows = stmt.query_map([], |row| {
        let name: String = row.get("name")?;
        let ingredients: String = row.get("ingredients")?;
        let tags: String = row.get::<_, Option<String>>("tags")?.unwrap_or_default();

        Ok(Recipe {
            id: row.get("id")?,
            instructions: row.get("directions")?,
            name_lc: name.to_lowercase(),
            ingredients_lc: ingredients.to_lowercase(),
            tags_lc: tags.to_lowercase(),
            name,
            ingredients,
            tags,
        })
    })?;

    rows.collect()
}

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("Rust Recipe Project")
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
    page: usize,
    include_input: String,
    exclude_input: String,
    tags_input: String,
}

impl App {
    fn new() -> Self {
        let recipes = load_recipes("recipes.sqlite").unwrap_or_else(|err| {
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
            page: 0,
            include_input: String::new(),
            exclude_input: String::new(),   
            tags_input: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    SearchChanged(String),
    ToggleAdvanced,
    RecipeSelected(i32),
    BackToResults,
    Next,
    Back,
    IncludeChanged(String),
    ExcludeChanged(String),
    TagsChanged(String), 
}

struct Filters {
        query: String,
        include: Vec<String>,
        exclude: Vec<String>,
        tags: Vec<String>,
    }

    impl Filters {
        fn is_empty(&self) -> bool {
            self.query.is_empty()
                && self.include.is_empty()
                && self.exclude.is_empty()
                && self.tags.is_empty()
        }

        fn matches(&self, recipe: &Recipe) -> bool {
            let text_ok = self.query.is_empty()
                || recipe.name_lc.contains(&self.query)
                || recipe.ingredients_lc.contains(&self.query);

            let include_ok = self
                .include
                .iter()
                .all(|t| recipe.ingredients_lc.contains(t.as_str()));

            let exclude_ok = !self
                .exclude
                .iter()
                .any(|t| recipe.ingredients_lc.contains(t.as_str()));

            let tags_ok = self
                .tags
                .iter()
                .all(|t| {
                    recipe
                        .tags_lc
                        .split(',')
                        .map(|rt| {
                            rt.trim()
                                .trim_matches(|c: char| c == '[' || c == ']' || c == '"' || c == '\'' || c.is_whitespace())
                        })
                        .any(|clean_tag| clean_tag == t.as_str())
                    });
            text_ok && include_ok && exclude_ok && tags_ok
        }
}

fn parse_terms(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(|term| term.trim().to_lowercase())
        .filter(|term| !term.is_empty())
        .collect()
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SearchChanged(value) => 
            {
                self.search = value;
                self.selected = None;
                self.page = 0;
            }
            Message::IncludeChanged(value) => {
                self.include_input = value;
                self.page = 0;
                self.selected = None;
            }
            Message::ExcludeChanged(value) => {
                self.exclude_input = value;
                self.page = 0;
                self.selected = None;
            }
            Message::TagsChanged(value) => {
                self.tags_input = value;
                self.page = 0;
                self.selected = None;
            }
            Message::ToggleAdvanced => self.show_advanced = !self.show_advanced,
            Message::RecipeSelected(id) => self.selected = Some(id),
            Message::BackToResults => self.selected = None,
            Message::Next =>
            {
                self.page += 1;
                return scroll_to_top();
            }
            Message::Back =>
            {
                self.page = self.page.saturating_sub(1);
                return scroll_to_top();
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
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

        let arrow_icon = image(image::Handle::from_path("assets/arrow.png"))
            .width(Length::Fixed(48.0))
            .height(Length::Fixed(48.0));

        let advanced_toggle = button(arrow_icon)
            .on_press(Message::ToggleAdvanced)
            .style(button::text);

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

        if filters.is_empty() {
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
        if self.show_advanced {
            body = body.push(self.advanced_panel());
        }
        let results: Element<'_, Message> = match self.selected {
            Some(id) => self.recipe_detail(id),
            None => column![
                scrollable(list).id(RESULTS_ID).width(Length::Fill).height(Length::Fill),
                nav,
            ]
        .spacing(10)
        .width(Length::Fill)
        .into(),
        };
        body = body.push(results);

        container(column![search_row, body].spacing(10).padding(20))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn recipe_matches(&self, recipe: &Recipe, query: &str) -> bool {
        let title = recipe.name.to_lowercase();
        let ingredients = recipe.ingredients.to_lowercase();

        let text_ok = query.is_empty() || title.contains(query) || ingredients.contains(query);
        let included_ok = self.included.iter().all(|i| ingredients.contains(i.as_str()));
        let excluded_ok = !self.excluded.iter().any(|e| ingredients.contains(e.as_str()));

        text_ok && included_ok && excluded_ok
    }

    fn advanced_panel(&self) -> Element<'_, Message> {
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


    fn recipe_detail(&self, id: i32) -> Element<'_, Message> {
        let back = button(text("Back to results")).on_press(Message::BackToResults);

        let content = match self.recipes.iter().find(|r| r.id == id) {
            Some(recipe) => column![
                text(recipe.name.as_str()).size(26),
                text("Ingredients").size(18),
                text(recipe.ingredients.as_str()).size(14),
                text("Instructions").size(18),
                text(recipe.instructions.as_str()).size(14),
                text("Tags").size(18),
                text(recipe.tags.as_str()).size(14),
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
 
