use crate::ai::GeminiClient;
use crate::commands::{
    handle_command, is_demand_trigger, is_group_chat, should_process_spontaneous, DemandAction,
};
use crate::config::Config;
use crate::stats::BotStats;
use std::path::Path;
use std::sync::Arc;
use tracing::{error, info, warn};
use whatsapp_rust::prelude::*;

pub struct WhatsAppService {
    config: Config,
}

impl WhatsAppService {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn ensure_parent_dir(path: &Path) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
            && !parent.exists()
        {
            std::fs::create_dir_all(parent)?;
        }
        Ok(())
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        Self::ensure_parent_dir(&self.config.db_path)?;

        let db_str = self
            .config
            .db_path
            .to_str()
            .ok_or("Invalid database path UTF-8")?;

        info!("Inicializando armazenamento SQLite em: {}", db_str);
        let store = SqliteStore::new(db_str).await?;

        let gemini_client = Arc::new(GeminiClient::new(
            self.config.gemini_api_key.clone(),
            self.config.gemini_model.clone(),
            self.config.gemini_endpoint.clone(),
        ));
        let stats = Arc::new(BotStats::new());

        let group_mode = self.config.group_mode;
        let audio_transcription_enabled = self.config.audio_transcription_enabled;
        let image_description_enabled = self.config.image_description_enabled;
        let sticker_description_enabled = self.config.sticker_description_enabled;

        info!("Construindo cliente WhatsApp Hendu...");
        let bot = Bot::builder()
            .with_backend(store)
            .on_qr_code(|code, _timeout| async move {
                info!("=== ESCANEIE O QR CODE NO WHATSAPP PARA EMPARELHAR ===");
                println!("\n{code}\n");
            })
            .on_message(move |ctx| {
                let gemini = Arc::clone(&gemini_client);
                let stats = Arc::clone(&stats);
                async move {
                    let chat_jid = ctx.info.source.chat.to_string();
                    let is_group = is_group_chat(&chat_jid);

                    // 1. Tratamento de mensagens de texto, comandos comunitários e gatilhos sob demanda
                    if let Some(text) = ctx.message.text_content() {
                        info!("Mensagem de texto recebida: {}", text);

                        if let Some(reply) = handle_command(text, &stats) {
                            info!("Executando resposta para comando de texto...");
                            if let Err(e) = ctx.reply(reply).await {
                                error!("Falha ao responder comando: {:?}", e);
                            }
                            return;
                        }

                        if let Some(action) = is_demand_trigger(text) {
                            info!("Gatilho sob demanda detectado: {:?}", action);
                            let quoted = ctx
                                .message
                                .extended_text_message
                                .as_option()
                                .and_then(|m| m.context_info.as_option())
                                .and_then(|ci| ci.quoted_message.as_option());

                            match quoted {
                                Some(q) => match action {
                                    DemandAction::Transcribe => {
                                        if !audio_transcription_enabled {
                                            let _ = ctx
                                                .reply_quoting(
                                                    "⚠️ Transcrição de áudio está desativada na configuração.",
                                                )
                                                .await;
                                            return;
                                        }

                                        if let Some(audio) = q.audio_message.as_option() {
                                            let mime = audio
                                                .mimetype
                                                .as_deref()
                                                .unwrap_or("audio/ogg; codecs=opus")
                                                .to_string();
                                            info!("Baixando áudio citado sob demanda...");
                                            match ctx.client.download(audio).await {
                                                Ok(bytes) => {
                                                    match gemini.transcribe_audio(&bytes, &mime).await {
                                                        Ok(transcription) => {
                                                            stats.inc_audio();
                                                            let reply_body = format!(
                                                                "🎙️ Transcrição:\n\n{transcription}"
                                                            );
                                                            if let Err(e) = ctx.reply_quoting(reply_body).await {
                                                                error!("Falha ao responder transcrição sob demanda: {:?}", e);
                                                            }
                                                        }
                                                        Err(e) => {
                                                            warn!("Falha Gemini áudio sob demanda: {:?}", e)
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    error!("Falha ao baixar áudio citado: {:?}", e)
                                                }
                                            }
                                        } else {
                                            let _ = ctx
                                                .reply_quoting(
                                                    "⚠️ A mensagem respondida não contém um áudio para transcrição.",
                                                )
                                                .await;
                                        }
                                    }
                                    DemandAction::Describe => {
                                        if let Some(image) = q.image_message.as_option() {
                                            if !image_description_enabled {
                                                let _ = ctx
                                                    .reply_quoting(
                                                        "⚠️ Audiodescrição de imagens está desativada.",
                                                    )
                                                    .await;
                                                return;
                                            }
                                            let mime = image
                                                .mimetype
                                                .as_deref()
                                                .unwrap_or("image/jpeg")
                                                .to_string();
                                            info!("Baixando imagem citada sob demanda...");
                                            match ctx.client.download(image).await {
                                                Ok(bytes) => {
                                                    match gemini.describe_image(&bytes, &mime).await {
                                                        Ok(desc) => {
                                                            stats.inc_image();
                                                            let reply_body = format!(
                                                                "🖼️ Audiodescrição:\n\n{desc}"
                                                            );
                                                            if let Err(e) = ctx.reply_quoting(reply_body).await {
                                                                error!("Falha ao responder audiodescrição sob demanda: {:?}", e);
                                                            }
                                                        }
                                                        Err(e) => {
                                                            warn!("Falha Gemini imagem sob demanda: {:?}", e)
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    error!("Falha ao baixar imagem citada: {:?}", e)
                                                }
                                            }
                                        } else if let Some(sticker) = q.sticker_message.as_option() {
                                            if !sticker_description_enabled {
                                                let _ = ctx
                                                    .reply_quoting(
                                                        "⚠️ Descrição de figurinhas está desativada.",
                                                    )
                                                    .await;
                                                return;
                                            }
                                            let mime = sticker
                                                .mimetype
                                                .as_deref()
                                                .unwrap_or("image/webp")
                                                .to_string();
                                            info!("Baixando figurinha citada sob demanda...");
                                            match ctx.client.download(sticker).await {
                                                Ok(bytes) => {
                                                    match gemini.describe_sticker(&bytes, &mime).await {
                                                        Ok(desc) => {
                                                            stats.inc_sticker();
                                                            let reply_body =
                                                                format!("🎭 Figurinha:\n\n{desc}");
                                                            if let Err(e) = ctx.reply_quoting(reply_body).await {
                                                                error!("Falha ao responder figurinha sob demanda: {:?}", e);
                                                            }
                                                        }
                                                        Err(e) => {
                                                            warn!("Falha Gemini figurinha sob demanda: {:?}", e)
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    error!("Falha ao baixar figurinha citada: {:?}", e)
                                                }
                                            }
                                        } else {
                                            let _ = ctx
                                                .reply_quoting(
                                                    "⚠️ A mensagem respondida não contém imagem ou figurinha para audiodescrição.",
                                                )
                                                .await;
                                        }
                                    }
                                },
                                None => {
                                    let _ = ctx
                                        .reply_quoting(
                                            "💡 Responda a um áudio, imagem ou figurinha com !transcrever ou !descrever.",
                                        )
                                        .await;
                                }
                            }
                            return;
                        }
                    }

                    // 2. Verificação de permissão para processamento espontâneo de mídia
                    if !should_process_spontaneous(is_group, group_mode) {
                        return;
                    }

                    // 2. Tratamento de mensagens de áudio (PTT ou arquivo de áudio)
                    if audio_transcription_enabled
                        && let Some(audio) = ctx.message.audio_message.as_option()
                    {
                        let mime = audio
                            .mimetype
                            .as_deref()
                            .unwrap_or("audio/ogg; codecs=opus")
                            .to_string();

                        info!("Mensagem de áudio detectada. Baixando mídia do WhatsApp...");
                        match ctx.client.download(audio).await {
                            Ok(bytes) => {
                                info!(
                                    "Áudio baixado com sucesso ({} bytes). Transcrevendo com Gemini...",
                                    bytes.len()
                                );

                                match gemini.transcribe_audio(&bytes, &mime).await {
                                    Ok(transcription) => {
                                        stats.inc_audio();
                                        info!(
                                            "Transcrição gerada com sucesso ({} caracteres)",
                                            transcription.len()
                                        );
                                        let reply_body =
                                            format!("🎙️ Transcrição:\n\n{transcription}");
                                        if let Err(e) = ctx.reply_quoting(reply_body).await {
                                            error!(
                                                "Falha ao enviar resposta de transcrição: {:?}",
                                                e
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        warn!("Falha ao transcrever áudio via Gemini: {:?}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Falha ao baixar áudio do WhatsApp CDN: {:?}", e);
                            }
                        }
                    }

                    // 3. Tratamento de imagens (fotos, capturas de tela)
                    if image_description_enabled
                        && let Some(image) = ctx.message.image_message.as_option()
                    {
                        let mime = image
                            .mimetype
                            .as_deref()
                            .unwrap_or("image/jpeg")
                            .to_string();

                        info!("Mensagem com imagem detectada. Baixando mídia do WhatsApp...");
                        match ctx.client.download(image).await {
                            Ok(bytes) => {
                                info!(
                                    "Imagem baixada com sucesso ({} bytes). Gerando audiodescrição...",
                                    bytes.len()
                                );

                                match gemini.describe_image(&bytes, &mime).await {
                                    Ok(description) => {
                                        stats.inc_image();
                                        info!(
                                            "Audiodescrição gerada com sucesso ({} caracteres)",
                                            description.len()
                                        );
                                        let reply_body =
                                            format!("🖼️ Audiodescrição:\n\n{description}");
                                        if let Err(e) = ctx.reply_quoting(reply_body).await {
                                            error!(
                                                "Falha ao enviar resposta de audiodescrição: {:?}",
                                                e
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        warn!("Falha ao descrever imagem via Gemini: {:?}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Falha ao baixar imagem do WhatsApp CDN: {:?}", e);
                            }
                        }
                    }

                    // 4. Tratamento de figurinhas (stickers e memes)
                    if sticker_description_enabled
                        && let Some(sticker) = ctx.message.sticker_message.as_option()
                    {
                        let mime = sticker
                            .mimetype
                            .as_deref()
                            .unwrap_or("image/webp")
                            .to_string();

                        info!("Figurinha/sticker detectada. Baixando mídia do WhatsApp...");
                        match ctx.client.download(sticker).await {
                            Ok(bytes) => {
                                info!(
                                    "Figurinha baixada com sucesso ({} bytes). Descrevendo meme/figurinha...",
                                    bytes.len()
                                );

                                match gemini.describe_sticker(&bytes, &mime).await {
                                    Ok(description) => {
                                        stats.inc_sticker();
                                        info!(
                                            "Descrição de figurinha gerada com sucesso ({} caracteres)",
                                            description.len()
                                        );
                                        let reply_body =
                                            format!("🎭 Figurinha:\n\n{description}");
                                        if let Err(e) = ctx.reply_quoting(reply_body).await {
                                            error!(
                                                "Falha ao enviar resposta de descrição de figurinha: {:?}",
                                                e
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        warn!("Falha ao descrever figurinha via Gemini: {:?}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Falha ao baixar figurinha do WhatsApp CDN: {:?}", e);
                            }
                        }
                    }
                }
            })
            .build()
            .await?;

        info!("Hendu WhatsApp daemon pronto. Executando...");
        bot.run().await;

        Ok(())
    }
}
