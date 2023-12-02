use crate::error::Error::*;
use crate::error::Result;
use crate::models::user::FoundUser;
use crate::models::user::User;
use sqlx::{Pool, Sqlite, SqlitePool};

pub async fn create_db_pool(database_url: String) -> Pool<Sqlite> {
    SqlitePool::connect(database_url.as_str())
        .await
        .expect("Failed to connect to database")
}

pub async fn get_user_by_username(
    db_pool: Pool<Sqlite>,
    username: String,
) -> Result<Option<FoundUser>> {
    let result = sqlx::query_as!(
        FoundUser,
        "SELECT id, username, password, name, permissions_group FROM users WHERE username = ?",
        username
    )
    .fetch_all(&db_pool)
    .await;
    // dbg!(&result);
    // let result: Result<Vec<FoundUser>> = Err(DBConnectionFail);
    if let Ok(mut r) = result {
        if r.is_empty() {
            Ok(None)
        } else if r.len() > 1 {
            Err(DBFoundMultipleExpectedUnique)
        } else {
            Ok(Some(r.remove(0)))
        }
    } else {
        Err(DBConnectionFail)
    }
}

pub async fn create_new_user(db_pool: Pool<Sqlite>, user: User) -> Result<i64> {
    let result = sqlx::query!(
        "INSERT INTO users (name, username, password, permissions_group) VALUES (?, ?, ?, ?)",
        user.name,
        user.username,
        user.password,
        user.permissions_group
    )
    .execute(&db_pool)
    .await;

    // dbg!(&result);
    if let Ok(res) = result {
        Ok(res.last_insert_rowid())
    } else {
        Err(DBFailedToCreateUser)
    }

    // let result: Result<Vec<FoundUser>> = Err(DBConnectionFail);
    // if let Ok(mut r) = result {
    //     if r.is_empty() {
    //         Ok(None)
    //     } else if r.len() > 1 {
    //         Err(DBFoundMultipleExpectedUnique)
    //     } else {
    //         Ok(Some(r.remove(0)))
    //     }
    // } else {
    //     Err(DBConnectionFail)
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::migrate;
    use std::fs;

    fn delete_database(database_path: &str) -> std::io::Result<()> {
        fs::remove_file(database_path)?;
        Ok(())
    }

    #[tokio::test]
    async fn reset_database() {
        // std::env::set_var("RUST_BACKTRACE", "1");
        // Drop the database
        let _ = delete_database("database.sqlite");

        let _ = dotenv::dotenv();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
        let pool = create_db_pool(database_url).await;
        // Run the migrations
        let _ = migrate!("./migrations").run(&pool).await;
    }
}
