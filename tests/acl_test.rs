use hendu::acl::{
    AclManager, AclStatus, GroupOperationMode, OwnerCheck, OwnerCommand, normalize_brazilian_phone,
    parse_owner_command,
};

#[test]
fn test_normalize_brazilian_phone_8_and_9_digits() {
    // 55 48 991381979 (9 dígitos) e 55 48 91381979 (8 dígitos) devem normalizar para o mesmo padrão
    let n9 = normalize_brazilian_phone("5548991381979");
    let n8 = normalize_brazilian_phone("554891381979");
    assert_eq!(n9, n8);

    // Formato com caracteres especiais
    let formatted = normalize_brazilian_phone("+55 (48) 99138-1979");
    assert_eq!(formatted, n9);
}

#[test]
fn test_owner_check_matching() {
    let owner = OwnerCheck::new("5548991381979");

    // Match pelo número canônico
    assert!(owner.is_owner_phone("554891381979"));
    assert!(owner.is_owner_phone("5548991381979"));
    assert!(owner.is_owner_phone("+55 48 9138-1979"));

    // Não match para outro número
    assert!(!owner.is_owner_phone("5548999999999"));
}

#[test]
fn test_parse_owner_command_private_without_bang() {
    // No PV da dona, não precisa de "!"
    assert_eq!(
        parse_owner_command("permitir"),
        Some(OwnerCommand::AllowCurrent(None))
    );
    assert_eq!(
        parse_owner_command("permitir ondemand"),
        Some(OwnerCommand::AllowCurrent(Some(
            GroupOperationMode::OnDemand
        )))
    );
    assert_eq!(
        parse_owner_command("permitir automatic"),
        Some(OwnerCommand::AllowCurrent(Some(
            GroupOperationMode::Automatic
        )))
    );
    assert_eq!(
        parse_owner_command("permitir 5548999998888"),
        Some(OwnerCommand::AllowTarget("5548999998888".to_string(), None))
    );
    assert_eq!(
        parse_owner_command("bloquear"),
        Some(OwnerCommand::BlockCurrent)
    );
    assert_eq!(
        parse_owner_command("recusar"),
        Some(OwnerCommand::BlockCurrent)
    );
    assert_eq!(
        parse_owner_command("modo ondemand"),
        Some(OwnerCommand::SetMode(GroupOperationMode::OnDemand))
    );
    assert_eq!(
        parse_owner_command("modo automatic"),
        Some(OwnerCommand::SetMode(GroupOperationMode::Automatic))
    );
    assert_eq!(parse_owner_command("lista"), Some(OwnerCommand::List));

    // Também funciona com "!"
    assert_eq!(
        parse_owner_command("!permitir"),
        Some(OwnerCommand::AllowCurrent(None))
    );
    assert_eq!(parse_owner_command("!lista"), Some(OwnerCommand::List));

    // Texto aleatório não é comando
    assert_eq!(parse_owner_command("olá bom dia"), None);
}

#[test]
fn test_acl_welcome_message_contains_pronunciation_and_hides_phone() {
    let welcome = hendu::acl::format_welcome_message();
    // Deve conter significado e pronúncia Guarani
    assert!(welcome.contains("rren-DÚ"));
    assert!(welcome.contains("Guarani"));
    assert!(welcome.contains("escutar"));
    // NÃO deve expor o número pessoal da dona
    assert!(!welcome.contains("5548991381979"));
    assert!(!welcome.contains("91381979"));
}

#[test]
fn test_acl_state_machine_flow() {
    let mut acl = AclManager::new_in_memory();
    let contact_jid = "554888887777@s.whatsapp.net";

    // 1. Inicialmente desconhecido
    assert_eq!(acl.get_contact_status(contact_jid), AclStatus::Unknown);

    // 2. Registra primeiro contato -> Aguardando apresentação
    let action = acl.process_incoming(contact_jid, "Olá bot");
    assert_eq!(action, hendu::acl::AclAction::SendWelcome);
    assert_eq!(
        acl.get_contact_status(contact_jid),
        AclStatus::AwaitingIntro
    );

    // 3. Envia apresentação -> Hendu encaminha à dona e transiciona para Pendente
    let action2 = acl.process_incoming(
        contact_jid,
        "Sou a Maria e gostaria de testar o bot de acessibilidade.",
    );
    match action2 {
        hendu::acl::AclAction::ForwardIntroToOwner { intro, .. } => {
            assert!(intro.contains("Maria"));
        }
        other => panic!("Esperado ForwardIntroToOwner, obtido {:?}", other),
    }
    assert_eq!(
        acl.get_contact_status(contact_jid),
        AclStatus::PendingApproval
    );

    // 4. Nova mensagem enquanto pendente -> Silêncio (não responde e não flooda a dona)
    let action3 = acl.process_incoming(contact_jid, "E aí, já aprovou?");
    assert_eq!(action3, hendu::acl::AclAction::Silence);

    // 5. Dona aprova o contato
    acl.set_contact_status(contact_jid, AclStatus::Approved);
    assert_eq!(acl.get_contact_status(contact_jid), AclStatus::Approved);

    // 6. Mensagem de contato aprovado -> Permitido processar
    let action4 = acl.process_incoming(contact_jid, "Opa agora sim!");
    assert_eq!(action4, hendu::acl::AclAction::Allow);
}

#[test]
fn test_extract_target_from_quoted() {
    let group_msg = "🔔 *Novo pedido de acesso para GRUPO!*\n\n👥 *Grupo:* 12036304@g.us\n👤 *Solicitante:* 55489999@s.whatsapp.net";
    assert_eq!(
        hendu::acl::extract_target_from_quoted(group_msg),
        Some("12036304@g.us".to_string())
    );

    let contact_msg = "🔔 *Novo pedido de acesso individual!*\n\n👤 *Contato:* 554898765432@s.whatsapp.net\n📝 *Apresentação:* Olá";
    assert_eq!(
        hendu::acl::extract_target_from_quoted(contact_msg),
        Some("554898765432@s.whatsapp.net".to_string())
    );
}

#[test]
fn test_group_acl_mode_granular() {
    let mut acl = AclManager::new_in_memory();
    let group_jid = "123456789-group@g.us";

    // Inicialmente não autorizado
    assert_eq!(acl.get_group_status(group_jid), AclStatus::Unknown);

    // Dona autoriza grupo em ondemand
    acl.set_group_status(group_jid, AclStatus::Approved, GroupOperationMode::OnDemand);
    assert_eq!(acl.get_group_status(group_jid), AclStatus::Approved);
    assert_eq!(
        acl.get_group_mode(group_jid),
        Some(GroupOperationMode::OnDemand)
    );

    // Dona altera grupo para automatic
    acl.set_group_mode(group_jid, GroupOperationMode::Automatic);
    assert_eq!(
        acl.get_group_mode(group_jid),
        Some(GroupOperationMode::Automatic)
    );
}
