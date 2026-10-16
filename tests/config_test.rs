use hendu::config::Config;
use hendu::wa::WhatsAppService;
use std::collections::HashMap;
use std::path::Path;

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

#[test]
fn test_ensure_parent_dir_creates_nested() {
    let target = Path::new("/tmp/hendu_test_dir_nested/sub/whatsapp.db");
    let res = WhatsAppService::ensure_parent_dir(target);
    assert!(res.is_ok(), "Deve criar diretórios aninhados sem erro");
    assert!(target.parent().unwrap().exists());

    // Limpeza
    let _ = std::fs::remove_dir_all("/tmp/hendu_test_dir_nested");
}

#[test]
fn test_ensure_parent_dir_root_or_current() {
    let target = Path::new("whatsapp.db");
    let res = WhatsAppService::ensure_parent_dir(target);
    assert!(
        res.is_ok(),
        "Caminho sem diretório pai deve ser ignorado com sucesso"
    );
}
