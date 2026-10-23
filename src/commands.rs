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
            "🌟 *Hendu — Assistente de Acessibilidade*\n\n\
            Hendu é um assistente auto-hospedado para WhatsApp focado em acessibilidade no seu dia a dia:\n\n\
            🎙️ *Áudios*: Envie ou encaminhe um áudio e eu transcrevo em texto legível.\n\
            🖼️ *Fotos*: Envie uma foto e eu descrevo o que está na imagem e os textos visíveis.\n\
            🎭 *Figurinhas*: Envie um sticker e eu descrevo o meme, a piada visual e os textos.\n\n\
            📌 *Comandos disponíveis*:\n\
            • `!ajuda`: Exibe esta mensagem de apoio.\n\
            • `!sobre`: Explica a missão e o nome do bot.\n\
            • `!status`: Mostra o tempo ativo e estatísticas da sessão.\n\
            • `!transcrever`: Transcreve o áudio respondido/citado.\n\
            • `!descrever` / `!ver`: Descreve a imagem ou figurinha respondida.\n\
            • `!ping`: Testa se o bot está online."
                .to_string(),
        ),
        "!sobre" | "!about" | "sobre" | "about" => Some(
            "🌿 *Sobre o Hendu*\n\n\
            *Hendu* é uma palavra Guarani que significa *escutar, ouvir, prestar atenção, compreender* (pronúncia oxítona: *rren-DÚ*).\n\n\
            Assistente auto-hospedado de acessibilidade para WhatsApp que transforma mensagens de voz em texto legível e descreve imagens e figurinhas com inteligência artificial especializada e sem compartilhamento de informações.\n\n\
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
