use tokio_rusqlite::{rusqlite, Connection};

#[derive(Debug, Clone)]
pub struct Recipe {
    pub id: i32,
    pub name: String,
    pub ingredients: String,
    pub instructions: String,
    pub tags: String,
    // lowercase copies, used only for searching
    pub name_lc: String,
    pub ingredients_lc: String,
    pub tags_lc: String,
}

fn row_to_recipe(row: &rusqlite::Row<'_>) -> rusqlite::Result<Recipe> {
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
}

#[derive(Clone)]
pub struct Database {
    conn: Connection,
}

// Connection may not implement Debug, and Message needs Debug.
impl std::fmt::Debug for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Database")
    }
}

impl Database {
    pub async fn open(path: &str) -> Result<Self, String> {
        let conn = Connection::open(path).await.map_err(|e| e.to_string())?;
        let db = Self { conn };
        db.init().await?;
        Ok(db)
    }

    async fn init(&self) -> Result<(), String> {
        self.conn
            .call(|conn| -> rusqlite::Result<()> {
                conn.execute_batch(
                    "CREATE TABLE IF NOT EXISTS my_recipes (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        name TEXT NOT NULL,
                        ingredients TEXT NOT NULL,
                        directions TEXT NOT NULL,
                        tags TEXT
                    )",
                )?;
                Ok(())
            })
            .await
            .map_err(|e| e.to_string())
    }

    async fn load_table(&self, table: &'static str) -> Result<Vec<Recipe>, String> {
        self.conn
            .call(move |conn| -> rusqlite::Result<Vec<Recipe>> {
                let mut stmt = conn.prepare(&format!(
                    "SELECT id, name, ingredients, directions, tags FROM {table}"
                ))?;
                let rows = stmt.query_map([], row_to_recipe)?;
                rows.collect()
            })
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn load_recipes(&self) -> Result<Vec<Recipe>, String> {
        self.load_table("recipes").await
    }

    pub async fn load_my_recipes(&self) -> Result<Vec<Recipe>, String> {
        self.load_table("my_recipes").await
    }

    pub async fn add_recipe(
        &self,
        name: String,
        ingredients: String,
        directions: String,
        tags: String,
    ) -> Result<(), String> {
        self.conn
            .call(move |conn| -> rusqlite::Result<()> {
                conn.execute(
                    "INSERT INTO my_recipes (name, ingredients, directions, tags)
                     VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![name, ingredients, directions, tags],
                )?;
                Ok(())
            })
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn delete_recipe(&self, id: i32) -> Result<(), String> {
        self.conn
            .call(move |conn| -> rusqlite::Result<()> {
                conn.execute(
                    "DELETE FROM my_recipes WHERE id = ?1",
                    rusqlite::params![id],
                )?;
                Ok(())
            })
            .await
            .map_err(|e| e.to_string())
    }
}

/// Everything the app needs at startup.
#[derive(Debug, Clone)]
pub struct Loaded {
    pub db: Database,
    pub recipes: Vec<Recipe>,
    pub my_recipes: Vec<Recipe>,
}

pub async fn startup(path: String) -> Result<Loaded, String> {
    let db = Database::open(&path).await?;
    let recipes = db.load_recipes().await?;
    let my_recipes = db.load_my_recipes().await?;
    Ok(Loaded { db, recipes, my_recipes })
}