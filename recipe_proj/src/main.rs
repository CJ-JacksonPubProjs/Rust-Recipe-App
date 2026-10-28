mod db_backend;

use db_backend::{Database, Loaded, Recipe};
use iced::widget::operation::{self, RelativeOffset};
use iced::widget::{button, column, container, row, scrollable, text, text_editor, text_input};
use iced::{Element, Length, Task, Theme};

const MAX_RESULTS: usize = 50;
const RESULTS_ID: &str = "results";

fn scroll_to_top() -> Task<Message> {
    operation::snap_to(RESULTS_ID, RelativeOffset::START)
}

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("Rust Recipe Project")
        .theme(Theme::TokyoNight)
        .run()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Tab {
    #[default]
    Search,
    Create,
    Saved,
}

struct App {
    db: Option<Database>,
    tab: Tab,
    loading: bool,
    error: Option<String>,

    recipes: Vec<Recipe>,
    my_recipes: Vec<Recipe>,

    // search tab
    search: String,
    show_advanced: bool,
    selected: Option<i32>,
    page: usize,
    include_input: String,
    exclude_input: String,
    tags_input: String,

    // saved tab
    selected_saved: Option<i32>,

    // create tab
    new_name: String,
    new_ingredients: text_editor::Content,
    new_directions: text_editor::Content,
    new_tags: String,
    status: Option<String>,
}

#[derive(Debug, Clone)]
enum Message {
    Loaded(Result<Loaded, String>),
    TabSelected(Tab),
    // search tab
    SearchChanged(String),
    ToggleAdvanced,
    IncludeChanged(String),
    ExcludeChanged(String),
    TagsChanged(String),
    Next,
    Back,
    // shared by the search and saved tabs
    RecipeSelected(i32),
    BackToResults,
    // create tab
    NameEdited(String),
    IngredientsEdited(text_editor::Action),
    DirectionsEdited(text_editor::Action),
    NewTagsEdited(String),
    SubmitRecipe,
    RecipeSaved(Result<Vec<Recipe>, String>),
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

        let tags_ok = self.tags.iter().all(|t| {
            recipe
                .tags_lc
                .split(',')
                .map(|rt| {
                    rt.trim().trim_matches(|c: char| {
                        c == '[' || c == ']' || c == '"' || c == '\'' || c.is_whitespace()
                    })
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

fn recipe_detail(recipes: &[Recipe], id: i32) -> Element<'_, Message> {
    let back = button(text("Back to results")).on_press(Message::BackToResults);

    let content = match recipes.iter().find(|r| r.id == id) {
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

impl App {
    fn new() -> (Self, Task<Message>) {
        let app = Self {
            db: None,
            tab: Tab::default(),
            loading: true,
            error: None,
            recipes: Vec::new(),
            my_recipes: Vec::new(),
            search: String::new(),
            show_advanced: false,
            selected: None,
            page: 0,
            include_input: String::new(),
            exclude_input: String::new(),
            tags_input: String::new(),
            selected_saved: None,
            new_name: String::new(),
            new_ingredients: text_editor::Content::new(),
            new_directions: text_editor::Content::new(),
            new_tags: String::new(),
            status: None,
        };

        let task = Task::perform(
            db_backend::startup("recipes.sqlite".to_string()),
            Message::Loaded,
        );
        (app, task)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Loaded(result) => {
                self.loading = false;
                match result {
                    Ok(loaded) => {
                        self.db = Some(loaded.db);
                        self.recipes = loaded.recipes;
                        self.my_recipes = loaded.my_recipes;
                    }
                    Err(err) => {
                        eprintln!("Failed to load recipes: {err}");
                        self.error = Some(err);
                    }
                }
            }

            Message::TabSelected(tab) => self.tab = tab,

            // search tab
            Message::SearchChanged(value) => {
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
            Message::Next => {
                self.page += 1;
                return scroll_to_top();
            }
            Message::Back => {
                self.page = self.page.saturating_sub(1);
                return scroll_to_top();
            }

            // shared by search + saved
            Message::RecipeSelected(id) => match self.tab {
                Tab::Saved => self.selected_saved = Some(id),
                _ => self.selected = Some(id),
            },
            Message::BackToResults => match self.tab {
                Tab::Saved => self.selected_saved = None,
                _ => self.selected = None,
            },

            // create tab
            Message::NameEdited(v) => self.new_name = v,
            Message::IngredientsEdited(action) => self.new_ingredients.perform(action),
            Message::DirectionsEdited(action) => self.new_directions.perform(action),
            Message::NewTagsEdited(v) => self.new_tags = v,

            Message::SubmitRecipe => {
                let Some(db) = self.db.clone() else {
                    return Task::none();
                };
                let name = self.new_name.trim().to_string();
                let ingredients = self.new_ingredients.text();
                let directions = self.new_directions.text();
                let tags = self.new_tags.trim().to_string();

                self.status = Some("Saving...".to_string());

                return Task::perform(
                    async move {
                        db.add_recipe(name, ingredients, directions, tags).await?;
                        db.load_my_recipes().await
                    },
                    Message::RecipeSaved,
                );
            }
            Message::RecipeSaved(result) => match result {
                Ok(list) => {
                    self.my_recipes = list;
                    self.new_name.clear();
                    self.new_ingredients = text_editor::Content::new();
                    self.new_directions = text_editor::Content::new();
                    self.new_tags.clear();
                    self.status = Some("Recipe saved!".to_string());
                }
                Err(err) => self.status = Some(format!("Failed to save: {err}")),
            },
        } // <-- this closes `match message`

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let content = match self.tab {
            Tab::Search => self.search_view(),
            Tab::Create => self.create_view(),
            Tab::Saved => self.saved_view(),
        };

        row![
            self.sidebar(),
            container(content).width(Length::Fill).height(Length::Fill),
        ]
        .spacing(10)
        .padding(10)
        .into()
    }

    fn sidebar(&self) -> Element<'_, Message> {
        let tab_button = |label: &'static str, tab: Tab| {
            button(text(label).width(Length::Fill))
                .on_press(Message::TabSelected(tab))
                .width(Length::Fill)
                .style(if self.tab == tab {
                    button::primary
                } else {
                    button::secondary
                })
        };

        container(
            column![
                tab_button("New recipe", Tab::Create),
                tab_button("Search", Tab::Search),
                tab_button("My recipes", Tab::Saved),
            ]
            .spacing(8)
            .padding(10),
        )
        .width(Length::Fixed(180.0))
        .height(Length::Fill)
        .style(container::bordered_box)
        .into()
    }

    fn search_view(&self) -> Element<'_, Message> {
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

        // let arrow_icon = image(image::Handle::from_path("assets/arrow.png"))
        //     .width(Length::Fixed(48.0))
        //     .height(Length::Fixed(48.0));

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

    fn create_view(&self) -> Element<'_, Message> {
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

    fn saved_view(&self) -> Element<'_, Message> {
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