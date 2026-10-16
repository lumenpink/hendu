use hendu::config::Config;
use std::env;

#[test]
fn test_default_config() {
    // Limpar variáveis de ambiente para testar defaults
    env::remove_var("HENDU_DB_PATH");
    env::remove_var("HENDU_LOG_LEVEL");

    let config = Config::from_env().expect("Configuração default deve ser válida");
    assert_eq!(config.db_path.to_str().unwrap(), "data/whatsapp.db");
    assert_eq!(config.log_level, "info");
}

#[test]
fn test_custom_env_config() {
    env::set_var("HENDU_DB_PATH", "/tmp/custom_whatsapp.db");
    env::set_var("HENDU_LOG_LEVEL", "debug");

    let config = Config::from_env().expect("Configuração customizada deve ser válida");
    assert_eq!(config.db_path.to_str().unwrap(), "/tmp/custom_whatsapp.db");
    assert_eq!(config.log_level, "debug");

    // Cleanup
    env::remove_var("HENDU_DB_PATH");
    env::remove_var("HENDU_LOG_LEVEL");
}

#[test]
fn test_empty_db_path_fails() {
    env::set_var("HENDU_DB_PATH", "");
    let result = Config::from_env();
    assert!(result.is_err(), "Caminho de banco vazio deve retornar erro");
    env::remove_var("HENDU_DB_PATH");
}
