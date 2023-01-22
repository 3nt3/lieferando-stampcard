use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub lieferando: Lieferando,
    pub email: Email,
    pub headless: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lieferando {
    pub email: String,
    pub password: String,
    pub restaurant_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    pub username: String,
    pub password: String,
    pub server: String,
}
