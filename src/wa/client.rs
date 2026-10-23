use crate::acl::{
    AclManager, AclStatus, GroupOperationMode, OwnerCheck, OwnerCommand,
    extract_target_from_quoted, format_welcome_message, parse_owner_command,
};
use crate::ai::GeminiClient;
use crate::commands::{
    DemandAction, handle_command, is_demand_trigger, is_group_chat, should_process_spontaneous,
};
use crate::config::Config;
use crate::stats::BotStats;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
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

        Self::ensure_parent_dir(&self.config.acl_path)?;
        let acl_manager = Arc::new(Mutex::new(
            AclManager::load_or_create(&self.config.acl_path)
                .unwrap_or_else(|_| AclManager::new_in_memory()),
        ));

        let mut owner_check = OwnerCheck::new(&self.config.owner_phone);
        owner_check.add_lid("69239218139364@lid");
        let owner_check = Arc::new(owner_check);

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
                let acl = Arc::clone(&acl_manager);
                let owner = Arc::clone(&owner_check);
                async move {
                    let chat_jid = ctx.info.source.chat.to_string();
                    let sender_jid = ctx.info.source.sender.to_string();
                    let is_group = is_group_chat(&chat_jid);
                    let is_from_me = ctx.info.source.is_from_me;
                    let is_owner = is_from_me
                        || owner.is_owner_jid(&sender_jid)
                        || (!is_group && owner.is_owner_jid(&chat_jid));

                    // 1. Tratamento de mensagens de texto, comandos e ACL
                    if let Some(text) = ctx.message.text_content() {
                        info!("Mensagem de texto recebida: {}", text);

                        if is_owner {
                            if let Some(cmd) = parse_owner_command(text) {
                                let quoted_text = ctx
                                    .message
                                    .extended_text_message
                                    .as_option()
                                    .and_then(|m| m.context_info.as_option())
                                    .and_then(|ci| ci.quoted_message.as_option())
                                    .and_then(|qm| {
                                        qm.conversation.as_deref().or_else(|| {
                                            qm.extended_text_message
                                                .as_option()
                                                .and_then(|etm| etm.text.as_deref())
                                        })
                                    });

                                match cmd {
                                    OwnerCommand::AllowCurrent(mode) => {
                                        let target = quoted_text
                                            .and_then(extract_target_from_quoted)
                                            .unwrap_or_else(|| chat_jid.clone());
                                        let mut acl_guard = acl.lock().await;
                                        if target.ends_with("@g.us") {
                                            let m = mode.unwrap_or(GroupOperationMode::OnDemand);
                                            acl_guard.set_group_status(
                                                &target,
                                                AclStatus::Approved,
                                                m,
                                            );
                                            let _ = ctx
                                                .reply(format!(
                                                    "✅ Grupo `{}` autorizado com sucesso no modo *{:?}*!",
                                                    target, m
                                                ))
                                                .await;
                                        } else {
                                            acl_guard.set_contact_status(
                                                &target,
                                                AclStatus::Approved,
                                            );
                                            let _ = ctx
                                                .reply(format!(
                                                    "✅ Contato `{}` autorizado com sucesso!",
                                                    target
                                                ))
                                                .await;
                                        }
                                        return;
                                    }
                                    OwnerCommand::AllowTarget(target, mode) => {
                                        let mut acl_guard = acl.lock().await;
                                        if target.ends_with("@g.us") {
                                            let m = mode.unwrap_or(GroupOperationMode::OnDemand);
                                            acl_guard.set_group_status(
                                                &target,
                                                AclStatus::Approved,
                                                m,
                                            );
                                            let _ = ctx
                                                .reply(format!(
                                                    "✅ Grupo `{}` autorizado no modo *{:?}*!",
                                                    target, m
                                                ))
                                                .await;
                                        } else {
                                            let target_jid = if target.contains('@') {
                                                target
                                            } else {
                                                format!("{}@s.whatsapp.net", target)
                                            };
                                            acl_guard.set_contact_status(
                                                &target_jid,
                                                AclStatus::Approved,
                                            );
                                            let _ = ctx
                                                .reply(format!(
                                                    "✅ Contato `{}` autorizado com sucesso!",
                                                    target_jid
                                                ))
                                                .await;
                                        }
                                        return;
                                    }
                                    OwnerCommand::BlockCurrent => {
                                        let target = quoted_text
                                            .and_then(extract_target_from_quoted)
                                            .unwrap_or_else(|| chat_jid.clone());
                                        let mut acl_guard = acl.lock().await;
                                        if target.ends_with("@g.us") {
                                            acl_guard.set_group_status(
                                                &target,
                                                AclStatus::Blocked,
                                                GroupOperationMode::OnDemand,
                                            );
                                        } else {
                                            acl_guard.set_contact_status(
                                                &target,
                                                AclStatus::Blocked,
                                            );
                                        }
                                        let _ = ctx
                                            .reply(format!("⛔ `{}` foi bloqueado com sucesso.", target))
                                            .await;
                                        return;
                                    }
                                    OwnerCommand::BlockTarget(target) => {
                                        let mut acl_guard = acl.lock().await;
                                        if target.ends_with("@g.us") {
                                            acl_guard.set_group_status(
                                                &target,
                                                AclStatus::Blocked,
                                                GroupOperationMode::OnDemand,
                                            );
                                        } else {
                                            let target_jid = if target.contains('@') {
                                                target.clone()
                                            } else {
                                                format!("{}@s.whatsapp.net", target)
                                            };
                                            acl_guard.set_contact_status(
                                                &target_jid,
                                                AclStatus::Blocked,
                                            );
                                        }
                                        let _ = ctx
                                            .reply(format!("⛔ `{}` foi bloqueado com sucesso.", target))
                                            .await;
                                        return;
                                    }
                                    OwnerCommand::SetMode(mode) => {
                                        let target = quoted_text
                                            .and_then(extract_target_from_quoted)
                                            .unwrap_or_else(|| chat_jid.clone());
                                        acl.lock().await.set_group_mode(&target, mode);
                                        let _ = ctx
                                            .reply(format!(
                                                "⚙️ Modo do grupo `{}` alterado para *{:?}*.",
                                                target, mode
                                            ))
                                            .await;
                                        return;
                                    }
                                    OwnerCommand::List => {
                                        let summary = acl.lock().await.list_summary();
                                        let _ = ctx.reply(summary).await;
                                        return;
                                    }
                                }
                            }

                            if let Some(reply) = handle_command(text, &stats) {
                                let _ = ctx.reply(reply).await;
                                return;
                            }
                        } else {
                            // Não é a dona: rejeitar comandos administrativos
                            if parse_owner_command(text).is_some() {
                                let _ = ctx
                                    .reply("⚠️ Comandos administrativos são restritos à administradora do bot.")
                                    .await;
                                return;
                            }

                            // Filtro ACL
                            if is_group {
                                let group_status = acl.lock().await.get_group_status(&chat_jid);
                                match group_status {
                                    AclStatus::Unknown => {
                                        let _ = ctx.reply(format_welcome_message()).await;
                                        acl.lock().await.set_group_status(
                                            &chat_jid,
                                            AclStatus::AwaitingIntro,
                                            GroupOperationMode::OnDemand,
                                        );
                                        return;
                                    }
                                    AclStatus::AwaitingIntro => {
                                        let intro = text.to_string();
                                        let _ = acl.lock().await.process_incoming(&chat_jid, &intro);
                                        let _ = ctx
                                            .reply(
                                                "🌿 *Obrigado!* A apresentação do grupo foi encaminhada para a administradora. Aguarde a avaliação.",
                                            )
                                            .await;
                                        let owner_jid_str = format!(
                                            "{}@s.whatsapp.net",
                                            owner.canonical_phone()
                                        );
                                        if let Ok(owner_jid) = owner_jid_str.parse::<Jid>() {
                                            let notify = format!(
                                                "🔔 *Novo pedido de acesso para GRUPO!*\n\n👥 *Grupo:* {}\n👤 *Solicitante:* {}\n📝 *Apresentação:*\n\"{}\"\n\n👉 Para autorizar no modo sob demanda: responda `permitir` ou `permitir ondemand`\n👉 Para autorizar no modo automático: responda `permitir automatic`\n👉 Para recusar: responda `bloquear`",
                                                chat_jid, sender_jid, intro
                                            );
                                            let _ = ctx.client.send_text(owner_jid, notify).await;
                                        }
                                        return;
                                    }
                                    AclStatus::PendingApproval | AclStatus::Blocked => {
                                        return;
                                    }
                                    AclStatus::Approved => {}
                                }
                            } else {
                                let contact_status = acl.lock().await.get_contact_status(&chat_jid);
                                match contact_status {
                                    AclStatus::Unknown => {
                                        let _ = ctx.reply(format_welcome_message()).await;
                                        acl.lock().await.set_contact_status(
                                            &chat_jid,
                                            AclStatus::AwaitingIntro,
                                        );
                                        return;
                                    }
                                    AclStatus::AwaitingIntro => {
                                        let intro = text.to_string();
                                        let _ = acl.lock().await.process_incoming(&chat_jid, &intro);
                                        let _ = ctx
                                            .reply(
                                                "🌿 *Obrigado!* Sua mensagem foi encaminhada para a administradora. Assim que ela avaliar, você poderá utilizar o Hendu.",
                                            )
                                            .await;
                                        let owner_jid_str = format!(
                                            "{}@s.whatsapp.net",
                                            owner.canonical_phone()
                                        );
                                        if let Ok(owner_jid) = owner_jid_str.parse::<Jid>() {
                                            let notify = format!(
                                                "🔔 *Novo pedido de acesso individual!*\n\n👤 *Contato:* {}\n📝 *Apresentação:*\n\"{}\"\n\n👉 Para autorizar: responda `permitir`\n👉 Para recusar: responda `bloquear`",
                                                chat_jid, intro
                                            );
                                            let _ = ctx.client.send_text(owner_jid, notify).await;
                                        }
                                        return;
                                    }
                                    AclStatus::PendingApproval | AclStatus::Blocked => {
                                        return;
                                    }
                                    AclStatus::Approved => {}
                                }
                            }

                            if let Some(reply) = handle_command(text, &stats) {
                                let _ = ctx.reply(reply).await;
                                return;
                            }
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
                                                        "⚠️ Descrição de imagens está desativada.",
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
                                                                "🖼️ Descrição da imagem:\n\n{desc}"
                                                            );
                                                            if let Err(e) = ctx.reply_quoting(reply_body).await {
                                                                error!("Falha ao responder descrição de imagem sob demanda: {:?}", e);
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
                                                                format!("🎭 Descrição da figurinha:\n\n{desc}");
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
                                                    "⚠️ A mensagem respondida não contém imagem ou figurinha para descrever.",
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

                    // 2. Verificação de permissão para processamento espontâneo de mídia com base na ACL
                    let effective_group_mode = if is_group {
                        let acl_guard = acl.lock().await;
                        if acl_guard.get_group_status(&chat_jid) != AclStatus::Approved {
                            return;
                        }
                        match acl_guard.get_group_mode(&chat_jid) {
                            Some(GroupOperationMode::OnDemand) => crate::config::GroupMode::OnDemand,
                            Some(GroupOperationMode::Automatic) => crate::config::GroupMode::Automatic,
                            None => group_mode,
                        }
                    } else {
                        if !is_owner {
                            let acl_guard = acl.lock().await;
                            if acl_guard.get_contact_status(&chat_jid) != AclStatus::Approved {
                                return;
                            }
                        }
                        group_mode
                    };

                    if !should_process_spontaneous(is_group, effective_group_mode) {
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
                                            format!("🖼️ Descrição da imagem:\n\n{description}");
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
                                            format!("🎭 Descrição da figurinha:\n\n{description}");
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
