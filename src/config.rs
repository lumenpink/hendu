use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub db_path: PathBuf,
    pub log_level: String,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        Self::from_lookup(|k| env::var(k).ok())
    }

    pub fn from_lookup<F>(lookup: F) -> Result<Self, String>
    where
        F: Fn(&str) -> Option<String>,
    {
        let db_path_str =
            lookup("HENDU_DB_PATH").unwrap_or_else(|| "data/whatsapp.db".to_string());
        if db_path_str.trim().is_empty() {
            return Err("HENDU_DB_PATH cannot be empty".to_string());
        }
        let db_path = PathBuf::from(db_path_str);

        let log_level = lookup("HENDU_LOG_LEVEL").unwrap_or_else(|| "info".to_string());

        Ok(Self { db_path, log_level })
    }
}
