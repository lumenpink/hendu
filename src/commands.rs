use crate::stats::BotStats;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemandAction {
    Transcribe,
    Describe,
}

pub fn is_demand_trigger(text: &str) -> Option<DemandAction> {
    let trimmed = text.trim();
    let lower = trimmed.to_lowercase();
    match lower.as_str() {
        "!transcrever" => Some(DemandAction::Transcribe),
        "!descrever" | "!ver" => Some(DemandAction::Describe),
        _ => None,
    }
}

pub fn should_process_spontaneous(is_group: bool, mode: crate::config::GroupMode) -> bool {
    !is_group || mode == crate::config::GroupMode::Automatic
}

pub fn is_group_chat(chat_str: &str) -> bool {
    chat_str.ends_with("@g.us")
}

pub fn handle_command(input: &str, stats: &BotStats) -> Option<String> {
    let trimmed = input.trim();
    let lower = trimmed.to_lowercase();

    match lower.as_str() {
        "!ping" | "ping" => Some("pong".to_string()),
        "!ajuda" | "!help" | "ajuda" | "help" => Some(
            "🌟 *Hendu — Acessibilidade Comunitária*\n\n\
            Eu sou um bot auto-hospedado focado em acessibilidade para pessoas com deficiência:\n\n\
            🎙️ *Áudios*: Envie ou encaminhe um áudio e eu transcreverei o texto para pessoas surdas.\n\
            🖼️ *Fotos*: Envie uma imagem e eu gerarei uma audiodescrição para leitores de tela.\n\
            🎭 *Figurinhas*: Envie um sticker e eu descreverei a imagem, o meme e o texto contido.\n\n\
            📌 *Comandos disponíveis*:\n\
            • `!ajuda`: Exibe esta mensagem de apoio.\n\
            • `!sobre`: Explica a missão e o nome do bot.\n\
            • `!status`: Mostra o tempo ativo e estatísticas da sessão.\n\
            • `!transcrever`: Transcreve o áudio respondido/citado.\n\
            • `!descrever` / `!ver`: Audiodescreve a imagem ou figurinha respondida.\n\
            • `!ping`: Testa se o bot está online."
                .to_string(),
        ),
        "!sobre" | "!about" | "sobre" | "about" => Some(
            "🌿 *Sobre o Hendu*\n\n\
            *Hendu* é uma palavra Guarani que significa *escutar, ouvir, prestar atenção, compreender* (pronúncia oxítona: *rren-DÚ*).\n\n\
            Desenvolvido sob o perfil *Kollontai* em Rust puro, seguro, leve e auto-hospedado, garantindo privacidade, autonomia técnica e acessibilidade para grupos de pessoas com deficiência (PCD).\n\n\
            Código aberto sob licença Apache-2.0."
                .to_string(),
        ),
        "!status" | "status" => Some(
            format!(
                "📊 *Status do Hendu*\n\n\
                ⏱️ Tempo ativo: {}\n\
                🔄 Total processado: {} mídias\n\n\
                🎙️ Áudios transcritos: {}\n\
                🖼️ Imagens descritas: {}\n\
                🎭 Figurinhas descritas: {}",
                stats.uptime_formatted(),
                stats.total_processed(),
                stats.audios_transcribed(),
                stats.images_described(),
                stats.stickers_described()
            ),
        ),
        _ => None,
    }
}
