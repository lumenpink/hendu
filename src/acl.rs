use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Normaliza um telefone brasileiro para uma representação canônica única,
/// resolvendo a divergência entre números de 8 e 9 dígitos após o DDD.
pub fn normalize_brazilian_phone(s: &str) -> String {
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();

    // Se começa com 55 (DDI Brasil)
    if digits.starts_with("55") {
        // Formato com 9 dígitos após DDD (ex: 55 + 48 + 9 + 8 dígitos = 13 dígitos)
        // Se o 5º caractere (índice 4) for '9', removemos para unificar com contas antigas de 8 dígitos (12 dígitos)
        if digits.len() == 13 && digits.chars().nth(4) == Some('9') {
            let mut normalized = String::with_capacity(12);
            normalized.push_str(&digits[0..4]);
            normalized.push_str(&digits[5..]);
            return normalized;
        }
        return digits;
    }

    // Se veio sem DDI (ex: 11 dígitos locais: 48 + 9 + 8 dígitos)
    if digits.len() == 11 && digits.chars().nth(2) == Some('9') {
        let mut normalized = String::with_capacity(12);
        normalized.push_str("55");
        normalized.push_str(&digits[0..2]);
        normalized.push_str(&digits[3..]);
        return normalized;
    }

    // Se veio com 10 dígitos (48 + 8 dígitos)
    if digits.len() == 10 {
        return format!("55{digits}");
    }

    digits
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnerCheck {
    canonical_phone: String,
    owner_lids: Vec<String>,
}

impl OwnerCheck {
    pub fn new(owner_phone: &str) -> Self {
        Self {
            canonical_phone: normalize_brazilian_phone(owner_phone),
            owner_lids: Vec::new(),
        }
    }

    pub fn with_lid(mut self, lid: &str) -> Self {
        self.owner_lids.push(lid.to_string());
        self
    }

    pub fn add_lid(&mut self, lid: &str) {
        if !self.owner_lids.iter().any(|l| l == lid) {
            self.owner_lids.push(lid.to_string());
        }
    }

    pub fn is_owner_phone(&self, phone: &str) -> bool {
        let candidate = normalize_brazilian_phone(phone);
        candidate == self.canonical_phone
    }

    pub fn is_owner_jid(&self, jid: &str) -> bool {
        // Se for JID de LID: 69239218139364@lid ou 69239218139364:1@lid
        if jid.contains("@lid") {
            let user_part = jid.split('@').next().unwrap_or(jid);
            let user_clean = user_part.split(':').next().unwrap_or(user_part);
            return self.owner_lids.iter().any(|lid| {
                let clean = lid.split('@').next().unwrap_or(lid);
                let clean = clean.split(':').next().unwrap_or(clean);
                clean == user_clean
            });
        }

        // Se for JID padrão: 554891381979@s.whatsapp.net ou 554891381979:1@s.whatsapp.net
        let user_part = jid.split('@').next().unwrap_or(jid);
        let user_clean = user_part.split(':').next().unwrap_or(user_part);
        self.is_owner_phone(user_clean)
    }

    pub fn canonical_phone(&self) -> &str {
        &self.canonical_phone
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum GroupOperationMode {
    #[default]
    OnDemand,
    Automatic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AclStatus {
    #[default]
    Unknown,
    AwaitingIntro,
    PendingApproval,
    Approved,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnerCommand {
    AllowCurrent(Option<GroupOperationMode>),
    AllowTarget(String, Option<GroupOperationMode>),
    BlockCurrent,
    BlockTarget(String),
    SetMode(GroupOperationMode),
    List,
}

pub fn parse_owner_command(input: &str) -> Option<OwnerCommand> {
    let trimmed = input.trim();
    // Remove prefixo '!' se presente para que comandos funcionem no PV sem ele
    let command_str = trimmed.strip_prefix('!').unwrap_or(trimmed).trim();
    let lower = command_str.to_lowercase();
    let parts: Vec<&str> = lower.split_whitespace().collect();

    if parts.is_empty() {
        return None;
    }

    match parts[0] {
        "permitir" | "allow" => {
            if parts.len() == 1 {
                Some(OwnerCommand::AllowCurrent(None))
            } else if parts[1] == "ondemand" || parts[1] == "manual" {
                Some(OwnerCommand::AllowCurrent(Some(
                    GroupOperationMode::OnDemand,
                )))
            } else if parts[1] == "automatic" || parts[1] == "auto" {
                Some(OwnerCommand::AllowCurrent(Some(
                    GroupOperationMode::Automatic,
                )))
            } else {
                let target = parts[1].to_string();
                let mode = if parts.len() > 2 {
                    match parts[2] {
                        "automatic" | "auto" => Some(GroupOperationMode::Automatic),
                        "ondemand" | "manual" => Some(GroupOperationMode::OnDemand),
                        _ => None,
                    }
                } else {
                    None
                };
                Some(OwnerCommand::AllowTarget(target, mode))
            }
        }
        "bloquear" | "recusar" | "block" => {
            if parts.len() == 1 {
                Some(OwnerCommand::BlockCurrent)
            } else {
                Some(OwnerCommand::BlockTarget(parts[1].to_string()))
            }
        }
        "modo" | "mode" => {
            if parts.len() > 1 {
                match parts[1] {
                    "ondemand" | "manual" => {
                        Some(OwnerCommand::SetMode(GroupOperationMode::OnDemand))
                    }
                    "automatic" | "auto" => {
                        Some(OwnerCommand::SetMode(GroupOperationMode::Automatic))
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
        "lista" | "whitelist" | "list" => Some(OwnerCommand::List),
        _ => None,
    }
}

pub fn format_welcome_message() -> String {
    "🌿 *Olá! Eu sou o Hendu* (pronuncia-se *rren-DÚ*, oxítona).\n\n\
    *Hendu* é uma palavra de origem Guarani que significa *escutar, ouvir, prestar atenção e compreender*. \
    Hendu é um assistente auto-hospedado de acessibilidade para WhatsApp: transcrevo mensagens de áudio em texto legível \
    e descrevo fotos e figurinhas com inteligência artificial especializada e sem compartilhamento de informações.\n\n\
    No momento, este contato/grupo ainda não está na minha lista de autorizados.\n\n\
    💬 *Envie uma mensagem a seguir se apresentando* (quem é você ou qual é a proposta deste grupo) \
    para que eu possa encaminhar o pedido para a minha administradora avaliar o acesso."
        .to_string()
}

pub fn extract_target_from_quoted(quoted: &str) -> Option<String> {
    if let Some(pos) = quoted.find("*Grupo:*") {
        let after = &quoted[pos + 8..];
        let target = after.split_whitespace().next()?;
        return Some(target.trim().to_string());
    }
    if let Some(pos) = quoted.find("*Contato:*") {
        let after = &quoted[pos + 10..];
        let target = after.split_whitespace().next()?;
        return Some(target.trim().to_string());
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AclAction {
    SendWelcome,
    ForwardIntroToOwner {
        target_jid: String,
        target_name: Option<String>,
        intro: String,
    },
    Silence,
    Allow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactEntry {
    pub jid: String,
    pub status: AclStatus,
    pub intro: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupEntry {
    pub group_jid: String,
    pub name: Option<String>,
    pub status: AclStatus,
    pub mode: GroupOperationMode,
    pub intro: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct AclStorage {
    contacts: HashMap<String, ContactEntry>,
    groups: HashMap<String, GroupEntry>,
}

#[derive(Debug)]
pub struct AclManager {
    storage: AclStorage,
    file_path: Option<PathBuf>,
}

impl AclManager {
    pub fn new_in_memory() -> Self {
        Self {
            storage: AclStorage::default(),
            file_path: None,
        }
    }

    pub fn load_or_create(path: &Path) -> Result<Self, std::io::Error> {
        let storage = if path.exists() {
            let data = fs::read_to_string(path)?;
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            AclStorage::default()
        };

        Ok(Self {
            storage,
            file_path: Some(path.to_path_buf()),
        })
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(ref path) = self.file_path {
            if let Some(parent) = path.parent()
                && !parent.exists()
            {
                fs::create_dir_all(parent)?;
            }
            let data =
                serde_json::to_string_pretty(&self.storage).map_err(std::io::Error::other)?;
            fs::write(path, data)?;
        }
        Ok(())
    }

    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    fn canonical_key(jid: &str) -> String {
        let user_part = jid.split('@').next().unwrap_or(jid);
        let user_clean = user_part.split(':').next().unwrap_or(user_part);
        if jid.ends_with("@g.us") {
            jid.to_string()
        } else {
            normalize_brazilian_phone(user_clean)
        }
    }

    pub fn get_contact_status(&self, jid: &str) -> AclStatus {
        let key = Self::canonical_key(jid);
        self.storage
            .contacts
            .get(&key)
            .map(|c| c.status)
            .unwrap_or(AclStatus::Unknown)
    }

    pub fn set_contact_status(&mut self, jid: &str, status: AclStatus) {
        let key = Self::canonical_key(jid);
        let now = Self::now();
        let entry = self
            .storage
            .contacts
            .entry(key)
            .or_insert_with(|| ContactEntry {
                jid: jid.to_string(),
                status,
                intro: None,
                created_at: now,
                updated_at: now,
            });
        entry.status = status;
        entry.updated_at = now;
        let _ = self.save();
    }

    pub fn get_group_status(&self, group_jid: &str) -> AclStatus {
        self.storage
            .groups
            .get(group_jid)
            .map(|g| g.status)
            .unwrap_or(AclStatus::Unknown)
    }

    pub fn get_group_mode(&self, group_jid: &str) -> Option<GroupOperationMode> {
        self.storage.groups.get(group_jid).map(|g| g.mode)
    }

    pub fn set_group_status(
        &mut self,
        group_jid: &str,
        status: AclStatus,
        mode: GroupOperationMode,
    ) {
        let now = Self::now();
        let entry = self
            .storage
            .groups
            .entry(group_jid.to_string())
            .or_insert_with(|| GroupEntry {
                group_jid: group_jid.to_string(),
                name: None,
                status,
                mode,
                intro: None,
                created_at: now,
                updated_at: now,
            });
        entry.status = status;
        entry.mode = mode;
        entry.updated_at = now;
        let _ = self.save();
    }

    pub fn set_group_mode(&mut self, group_jid: &str, mode: GroupOperationMode) {
        let now = Self::now();
        if let Some(entry) = self.storage.groups.get_mut(group_jid) {
            entry.mode = mode;
            entry.updated_at = now;
            let _ = self.save();
        }
    }

    pub fn process_incoming(&mut self, jid: &str, text: &str) -> AclAction {
        let is_group = jid.ends_with("@g.us");
        if is_group {
            let status = self.get_group_status(jid);
            match status {
                AclStatus::Approved => AclAction::Allow,
                AclStatus::Blocked | AclStatus::PendingApproval => AclAction::Silence,
                AclStatus::Unknown => {
                    self.set_group_status(
                        jid,
                        AclStatus::AwaitingIntro,
                        GroupOperationMode::OnDemand,
                    );
                    AclAction::SendWelcome
                }
                AclStatus::AwaitingIntro => {
                    let now = Self::now();
                    if let Some(entry) = self.storage.groups.get_mut(jid) {
                        entry.intro = Some(text.to_string());
                        entry.status = AclStatus::PendingApproval;
                        entry.updated_at = now;
                    }
                    let _ = self.save();
                    AclAction::ForwardIntroToOwner {
                        target_jid: jid.to_string(),
                        target_name: None,
                        intro: text.to_string(),
                    }
                }
            }
        } else {
            let key = Self::canonical_key(jid);
            let status = self.get_contact_status(jid);
            match status {
                AclStatus::Approved => AclAction::Allow,
                AclStatus::Blocked | AclStatus::PendingApproval => AclAction::Silence,
                AclStatus::Unknown => {
                    self.set_contact_status(jid, AclStatus::AwaitingIntro);
                    AclAction::SendWelcome
                }
                AclStatus::AwaitingIntro => {
                    let now = Self::now();
                    if let Some(entry) = self.storage.contacts.get_mut(&key) {
                        entry.intro = Some(text.to_string());
                        entry.status = AclStatus::PendingApproval;
                        entry.updated_at = now;
                    }
                    let _ = self.save();
                    AclAction::ForwardIntroToOwner {
                        target_jid: jid.to_string(),
                        target_name: None,
                        intro: text.to_string(),
                    }
                }
            }
        }
    }

    pub fn list_summary(&self) -> String {
        let mut msg = String::from("📋 *Lista de Controle de Acesso (ACL)*\n\n");

        msg.push_str("👤 *Contatos:*\n");
        let mut has_contacts = false;
        for (key, contact) in &self.storage.contacts {
            has_contacts = true;
            let status_str = match contact.status {
                AclStatus::Approved => "✅ Aprovado",
                AclStatus::PendingApproval => "⏳ Pendente",
                AclStatus::AwaitingIntro => "💬 Aguardando Apresentação",
                AclStatus::Blocked => "⛔ Bloqueado",
                AclStatus::Unknown => "❓ Desconhecido",
            };
            msg.push_str(&format!("• `{}`: {}\n", key, status_str));
        }
        if !has_contacts {
            msg.push_str("_(Nenhum contato registrado)_\n");
        }

        msg.push_str("\n👥 *Grupos:*\n");
        let mut has_groups = false;
        for (jid, group) in &self.storage.groups {
            has_groups = true;
            let mode_str = match group.mode {
                GroupOperationMode::OnDemand => "sob demanda",
                GroupOperationMode::Automatic => "automático",
            };
            let status_str = match group.status {
                AclStatus::Approved => format!("✅ Aprovado ({})", mode_str),
                AclStatus::PendingApproval => "⏳ Pendente".to_string(),
                AclStatus::AwaitingIntro => "💬 Aguardando Apresentação".to_string(),
                AclStatus::Blocked => "⛔ Bloqueado".to_string(),
                AclStatus::Unknown => "❓ Desconhecido".to_string(),
            };
            msg.push_str(&format!("• `{}`: {}\n", jid, status_str));
        }
        if !has_groups {
            msg.push_str("_(Nenhum grupo registrado)_\n");
        }

        msg
    }
}
