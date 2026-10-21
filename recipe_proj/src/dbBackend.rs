use tokio_rusqlite::Connection;

#[derive(Debug)]
struct Test {
    id: i32,
    recipe_names: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let conn = Connection::open("recipes.sqlite").await?;

    let recipe = conn
        .call(|conn| {
            let mut stmt = conn.prepare("SELECT id, name FROM recipes")?;
            let recipe_iter = stmt.query_map([], |row| {
                Ok(Test {
                    id: row.get(0)?,
                    recipe_names: row.get(1)?,
                })
            })?;

            // Collect results from the iterator into a Vector
            let mut list = Vec::new();
            for recipes in recipe_iter {
                list.push(recipes?);
            }
            Ok(list)
        })
        .await?;
    println!("--- Database Records ---");
    for recipes in recipe {
        println!("ID: {:<5} | Name: {}", recipes.id, recipes.recipe_name);
    }

    Ok(())
}


