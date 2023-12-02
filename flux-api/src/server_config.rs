use sqlx::{Pool, Sqlite};

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub jwt_secret_key: String,
    pub db_pool: Pool<Sqlite>,
}
