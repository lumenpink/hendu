use hendu::commands::handle_command;
use hendu::stats::BotStats;

#[test]
fn test_stats_increments() {
    let stats = BotStats::new();

    assert_eq!(stats.audios_transcribed(), 0);
    assert_eq!(stats.images_described(), 0);
    assert_eq!(stats.stickers_described(), 0);

    stats.inc_audio();
    stats.inc_image();
    stats.inc_image();
    stats.inc_sticker();

    assert_eq!(stats.audios_transcribed(), 1);
    assert_eq!(stats.images_described(), 2);
    assert_eq!(stats.stickers_described(), 1);
    assert_eq!(stats.total_processed(), 4);

    let summary = stats.summary();
    assert!(summary.contains("Áudios transcritos: 1"));
    assert!(summary.contains("Imagens descritas: 2"));
    assert!(summary.contains("Figurinhas descritas: 1"));
}

#[test]
fn test_handle_command_help() {
    let stats = BotStats::new();

    let res = handle_command("!ajuda", &stats);
    assert!(res.is_some());
    let text = res.unwrap();
    assert!(text.contains("Hendu"));
    assert!(text.contains("!status"));

    let res_alt = handle_command("  !help  ", &stats);
    assert!(res_alt.is_some());
}

#[test]
fn test_handle_command_about() {
    let stats = BotStats::new();

    let res = handle_command("!sobre", &stats);
    assert!(res.is_some());
    let text = res.unwrap();
    assert!(text.contains("Guarani"));
    assert!(text.contains("acessibilidade"));
}

#[test]
fn test_handle_command_status() {
    let stats = BotStats::new();
    stats.inc_audio();

    let res = handle_command("!status", &stats);
    assert!(res.is_some());
    let text = res.unwrap();
    assert!(text.contains("Status do Hendu"));
    assert!(text.contains("Tempo ativo:"));
    assert!(text.contains("Áudios transcritos: 1"));
}

#[test]
fn test_handle_command_ping() {
    let stats = BotStats::new();

    assert_eq!(handle_command("!ping", &stats).as_deref(), Some("pong"));
    assert_eq!(handle_command("ping", &stats).as_deref(), Some("pong"));
    assert_eq!(handle_command(" PING ", &stats).as_deref(), Some("pong"));
}

#[test]
fn test_handle_command_non_command() {
    let stats = BotStats::new();

    assert!(handle_command("Bom dia pessoal!", &stats).is_none());
    assert!(handle_command("", &stats).is_none());
    assert!(handle_command("!outro_comando_qualquer", &stats).is_none());
}
