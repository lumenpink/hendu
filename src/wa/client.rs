use crate::config::Config;
use std::path::Path;
use tracing::{error, info};
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

        info!("Construindo cliente WhatsApp Hendu...");
        let bot = Bot::builder()
            .with_backend(store)
            .on_qr_code(|code, _timeout| async move {
                info!("=== ESCANEIE O QR CODE NO WHATSAPP PARA EMPARELHAR ===");
                println!("\n{code}\n");
            })
            .on_message(|ctx| async move {
                if let Some(text) = ctx.message.text_content() {
                    info!("Mensagem de texto recebida: {}", text);
                    if text.trim() == "ping" {
                        info!("Respondendo a ping com pong...");
                        if let Err(e) = ctx.reply("pong").await {
                            error!("Falha ao responder ping: {:?}", e);
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
