use hendu::config::Config;
use hendu::wa::WhatsAppService;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = match Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Erro ao carregar configurações: {e}");
            std::process::exit(1);
        }
    };

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.log_level));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .init();

    info!("Iniciando Hendu WhatsApp Bot v0.1.0...");

    let service = WhatsAppService::new(config);

    tokio::select! {
        res = service.run() => {
            if let Err(e) = res {
                error!("Erro fatal no serviço WhatsApp: {:?}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Sinal de interrupção recebido (Ctrl+C). Encerrando Hendu...");
        }
    }

    info!("Hendu finalizado com sucesso.");
    Ok(())
}
