use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub password: String,
    pub name: String,
    pub permissions_group: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FoundUser {
    pub id: Option<i64>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub name: Option<String>,
    pub permissions_group: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicUser {
    pub id: i64,
    pub username: String,
    pub name: String,
}
