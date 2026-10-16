use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub db_path: PathBuf,
    pub log_level: String,
    pub gemini_api_key: Option<String>,
    pub gemini_model: String,
    pub gemini_endpoint: String,
    pub audio_transcription_enabled: bool,
    pub image_description_enabled: bool,
    pub sticker_description_enabled: bool,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        Self::from_lookup(|k| env::var(k).ok())
    }

    fn clean_non_empty(s: String) -> Option<String> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }

    fn parse_bool_env(raw: Option<String>, default: bool) -> bool {
        raw.map(|v| {
            let trimmed = v.trim().to_lowercase();
            trimmed != "0" && trimmed != "false" && trimmed != "no"
        })
        .unwrap_or(default)
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

        let gemini_api_key = lookup("GEMINI_API_KEY").and_then(Self::clean_non_empty);

        let gemini_model = lookup("GEMINI_MODEL")
            .and_then(Self::clean_non_empty)
            .unwrap_or_else(|| "gemini-3.5-flash-lite".to_string());

        let gemini_endpoint = lookup("GEMINI_ENDPOINT")
            .and_then(Self::clean_non_empty)
            .unwrap_or_else(|| "https://generativelanguage.googleapis.com".to_string());

        let audio_transcription_enabled =
            Self::parse_bool_env(lookup("HENDU_AUDIO_TRANSCRIPTION_ENABLED"), true);
        let image_description_enabled =
            Self::parse_bool_env(lookup("HENDU_IMAGE_DESCRIPTION_ENABLED"), true);
        let sticker_description_enabled =
            Self::parse_bool_env(lookup("HENDU_STICKER_DESCRIPTION_ENABLED"), true);

        Ok(Self {
            db_path,
            log_level,
            gemini_api_key,
            gemini_model,
            gemini_endpoint,
            audio_transcription_enabled,
            image_description_enabled,
            sticker_description_enabled,
        })
    }
}
