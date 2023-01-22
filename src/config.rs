use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub lieferando: Lieferando,
    pub email: Email,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lieferando {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    pub username: String,
    pub password: String,
    pub server: String,
}
