use hendu::config::Config;
use std::collections::HashMap;

#[test]
fn test_default_config_lookup() {
    let empty_lookup = HashMap::<String, String>::new();
    let config = Config::from_lookup(|k| empty_lookup.get(k).cloned())
        .expect("Configuração default deve ser válida");
    assert_eq!(config.db_path.to_str().unwrap(), "data/whatsapp.db");
    assert_eq!(config.log_level, "info");
}

#[test]
fn test_custom_config_lookup() {
    let mut custom = HashMap::new();
    custom.insert(
        "HENDU_DB_PATH".to_string(),
        "/tmp/custom_whatsapp.db".to_string(),
    );
    custom.insert("HENDU_LOG_LEVEL".to_string(), "debug".to_string());

    let config = Config::from_lookup(|k| custom.get(k).cloned())
        .expect("Configuração customizada deve ser válida");
    assert_eq!(config.db_path.to_str().unwrap(), "/tmp/custom_whatsapp.db");
    assert_eq!(config.log_level, "debug");
}

#[test]
fn test_empty_db_path_fails() {
    let mut custom = HashMap::new();
    custom.insert("HENDU_DB_PATH".to_string(), "   ".to_string());

    let result = Config::from_lookup(|k| custom.get(k).cloned());
    assert!(result.is_err(), "Caminho de banco vazio deve retornar erro");
}

#[test]
fn test_from_env_smoke() {
    // Validação de sanidade do método from_env real
    let res = Config::from_env();
    assert!(res.is_ok(), "from_env default deve carregar sem erro");
}
