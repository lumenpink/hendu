use hendu::commands::{DemandAction, is_demand_trigger};
use hendu::config::{Config, GroupMode};
use std::collections::HashMap;

#[test]
fn test_group_mode_default_is_automatic() {
    let env: HashMap<String, String> = HashMap::new();
    let config = Config::from_lookup(|k| env.get(k).cloned()).unwrap();

    assert_eq!(config.group_mode, GroupMode::Automatic);
}

#[test]
fn test_group_mode_ondemand_variants() {
    for variant in &["ondemand", "on_demand", "on-demand", "manual", " ONDEMAND "] {
        let mut env = HashMap::new();
        env.insert("HENDU_GROUP_MODE".to_string(), variant.to_string());
        let config = Config::from_lookup(|k| env.get(k).cloned()).unwrap();

        assert_eq!(
            config.group_mode,
            GroupMode::OnDemand,
            "Variante '{}' deveria resultar em GroupMode::OnDemand",
            variant
        );
    }
}

#[test]
fn test_group_mode_automatic_variants() {
    for variant in &["automatic", "auto", "default", ""] {
        let mut env = HashMap::new();
        env.insert("HENDU_GROUP_MODE".to_string(), variant.to_string());
        let config = Config::from_lookup(|k| env.get(k).cloned()).unwrap();

        assert_eq!(
            config.group_mode,
            GroupMode::Automatic,
            "Variante '{}' deveria resultar em GroupMode::Automatic",
            variant
        );
    }
}

#[test]
fn test_demand_trigger_transcribe() {
    assert_eq!(
        is_demand_trigger("!transcrever"),
        Some(DemandAction::Transcribe)
    );
    assert_eq!(
        is_demand_trigger("  !TRANSCREVER  "),
        Some(DemandAction::Transcribe)
    );
}

#[test]
fn test_demand_trigger_describe() {
    assert_eq!(
        is_demand_trigger("!descrever"),
        Some(DemandAction::Describe)
    );
    assert_eq!(is_demand_trigger("!ver"), Some(DemandAction::Describe));
    assert_eq!(is_demand_trigger("  !VER  "), Some(DemandAction::Describe));
}

#[test]
fn test_demand_trigger_none() {
    assert_eq!(is_demand_trigger("olá pessoal"), None);
    assert_eq!(is_demand_trigger("!ajuda"), None);
    assert_eq!(is_demand_trigger(""), None);
}

#[test]
fn test_should_process_spontaneous_combinations() {
    use hendu::commands::should_process_spontaneous;

    // Em DM (is_group = false), sempre processa espontaneamente
    assert!(should_process_spontaneous(false, GroupMode::Automatic));
    assert!(should_process_spontaneous(false, GroupMode::OnDemand));

    // Em Grupo (is_group = true):
    // Automatic -> processa espontâneo
    assert!(should_process_spontaneous(true, GroupMode::Automatic));
    // OnDemand -> NÃO processa espontâneo (silêncio/privacidade)
    assert!(!should_process_spontaneous(true, GroupMode::OnDemand));
}

#[test]
fn test_is_group_chat_detection() {
    use hendu::commands::is_group_chat;

    assert!(is_group_chat("120363025648593@g.us"));
    assert!(!is_group_chat("5511999999999@s.whatsapp.net"));
    assert!(!is_group_chat("status@broadcast"));
    assert!(!is_group_chat(""));
}
