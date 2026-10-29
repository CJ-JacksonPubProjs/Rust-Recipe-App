use crate::db_backend::{Database, Loaded, Recipe};
use iced::widget::operation::{self, RelativeOffset};
use iced::widget::{button, column, container, row, text, text_editor};
use iced::{Element, Length, Task};

use crate::views::search_recipes::RESULTS_ID;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Search,
    Create,
    Saved,
}

pub enum RecipeOrigin {
    Search,
    Saved,
}

#[derive(Debug, Clone)]
pub enum Message {
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
    // shared by search and saved tabs
    RecipeSelected(i32),
    BackToResults,
    // create tab
    NameEdited(String),
    IngredientsEdited(text_editor::Action),
    DirectionsEdited(text_editor::Action),
    NewTagsEdited(String),
    SubmitRecipe,
    RecipeSaved(Result<Vec<Recipe>, String>),
    //different detailed view messages dependent on RecipeOrigin
    SaveRecipe(i32),
    DeleteRecipe(i32),
    // Async database callbacks
    RecipeSavedToList(Result<Vec<Recipe>, String>),
    RecipeDeletedFromList(Result<Vec<Recipe>, String>),
}

pub struct Filters {
    pub query: String,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub tags: Vec<String>,
}

impl Filters {
    pub fn is_empty(&self) -> bool {
        self.query.is_empty()
            && self.include.is_empty()
            && self.exclude.is_empty()
            && self.tags.is_empty()
    }

    pub fn matches(&self, recipe: &Recipe) -> bool {
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

pub fn parse_terms(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(|term| term.trim().to_lowercase())
        .filter(|term| !term.is_empty())
        .collect()
}

fn scroll_to_top() -> Task<Message> {
    operation::snap_to(RESULTS_ID, RelativeOffset::START)
}

pub struct App {
    pub db: Option<Database>,
    pub tab: Tab,
    pub loading: bool,
    pub error: Option<String>,

    pub recipes: Vec<Recipe>,
    pub my_recipes: Vec<Recipe>,

    // search tab
    pub search: String,
    pub show_advanced: bool,
    pub selected: Option<i32>,
    pub page: usize,
    pub include_input: String,
    pub exclude_input: String,
    pub tags_input: String,

    // saved tab
    pub selected_saved: Option<i32>,

    // create tab
    pub new_name: String,
    pub new_ingredients: text_editor::Content,
    pub new_directions: text_editor::Content,
    pub new_tags: String,
    pub status: Option<String>,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
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
            crate::db_backend::startup("recipes.sqlite".to_string()),
            Message::Loaded,
        );
        (app, task)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
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
            
            //handles tab navigation
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

            //controls save/delete for search + saved
            Message::SaveRecipe(id) => {
                let Some(db) = self.db.clone() else {
                    return Task::none();
                };

                let Some(recipe) = self.recipes.iter().find(|r| r.id == id).cloned() else {
                    return Task::none();
                };

                self.status = Some("Saving recipe...".to_string());

                return Task::perform(
                    async move {
                        // Reusing your existing add_recipe method
                        db.add_recipe(
                            recipe.name,
                            recipe.ingredients,
                            recipe.instructions, // or directions depending on your struct field
                            recipe.tags,
                        )
                        .await?;

                        // Reusing your existing load_my_recipes method
                        db.load_my_recipes().await
                    },
                    Message::RecipeSavedToList,
                );
        }

            Message::RecipeSavedToList(result) => match result {
                Ok(updated_list) => {
                    self.my_recipes = updated_list;
                    self.status = Some("Recipe saved to your list!".to_string());
                }
                Err(err) => {
                    self.status = Some(format!("Failed to save recipe: {err}"));
                }
            },

            Message::DeleteRecipe(id) => {
                let Some(db) = self.db.clone() else {
                    return Task::none();
                };

                self.status = Some("Deleting recipe...".to_string());

                return Task::perform(
                    async move {
                        db.delete_recipe(id).await?;
                        db.load_my_recipes().await
                    },
                    Message::RecipeDeletedFromList,
                );
            }

            Message::RecipeDeletedFromList(result) => match result {
                Ok(updated_list) => {
                    self.my_recipes = updated_list;
                    self.selected_saved = None; // Reset detail view back to list view
                    self.status = Some("Recipe deleted from your list.".to_string());
                }
                Err(err) => {
                    self.status = Some(format!("Failed to delete recipe: {err}"));
                }
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
        }

        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
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
}