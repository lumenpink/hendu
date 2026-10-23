use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GeminiError {
    #[error("Chave de API do Gemini não configurada")]
    MissingApiKey,
    #[error("Falha na requisição de rede: {0}")]
    Network(#[from] reqwest::Error),
    #[error("A API do Gemini retornou erro HTTP {status}: {body}")]
    ApiError { status: u16, body: String },
    #[error("Resposta do Gemini sem candidatos ou vazia")]
    EmptyResponse,
    #[error("Falha ao deserializar resposta do Gemini: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InlineData {
    pub mime_type: String,
    pub data: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Part {
    Text { text: String },
    InlineData { inline_data: InlineData },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Content {
    pub parts: Vec<Part>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerateContentRequest {
    pub contents: Vec<Content>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CandidatePart {
    pub text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CandidateContent {
    #[serde(default)]
    pub parts: Vec<CandidatePart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Candidate {
    pub content: Option<CandidateContent>,
    #[serde(rename = "finishReason")]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct GenerateContentResponse {
    #[serde(default)]
    pub candidates: Vec<Candidate>,
}

impl GenerateContentResponse {
    pub fn extract_text(&self) -> Option<String> {
        for candidate in &self.candidates {
            if let Some(content) = &candidate.content {
                let texts: Vec<&str> = content
                    .parts
                    .iter()
                    .filter_map(|p| p.text.as_deref())
                    .collect();
                if !texts.is_empty() {
                    return Some(texts.join(" ").trim().to_string());
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct GeminiClient {
    client: Client,
    api_key: Option<String>,
    model: String,
    endpoint: String,
}

impl GeminiClient {
    pub fn new(api_key: Option<String>, model: String, endpoint: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(45))
                .build()
                .unwrap_or_else(|_| Client::new()),
            api_key,
            model,
            endpoint,
        }
    }

    pub fn build_transcription_request(
        audio_bytes: &[u8],
        mime_type: &str,
    ) -> GenerateContentRequest {
        let b64_data = BASE64_STANDARD.encode(audio_bytes);
        let prompt = "Transcreva este áudio em português de forma literal e precisa. Não inclua notas introdutórias nem comentários adicionais. Retorne estritamente o texto falado.";

        GenerateContentRequest {
            contents: vec![Content {
                role: Some("user".to_string()),
                parts: vec![
                    Part::Text {
                        text: prompt.to_string(),
                    },
                    Part::InlineData {
                        inline_data: InlineData {
                            mime_type: mime_type.to_string(),
                            data: b64_data,
                        },
                    },
                ],
            }],
        }
    }

    pub fn build_image_description_request(
        image_bytes: &[u8],
        mime_type: &str,
    ) -> GenerateContentRequest {
        let b64_data = BASE64_STANDARD.encode(image_bytes);
        let prompt = "Descreva esta imagem de forma objetiva, acolhedora e clara. Destaque o elemento principal, cenário, detalhes importantes e transcreva com exatidão qualquer texto visível na imagem. Retorne apenas a descrição direta, sem saudações ou comentários extras.";

        GenerateContentRequest {
            contents: vec![Content {
                role: Some("user".to_string()),
                parts: vec![
                    Part::Text {
                        text: prompt.to_string(),
                    },
                    Part::InlineData {
                        inline_data: InlineData {
                            mime_type: mime_type.to_string(),
                            data: b64_data,
                        },
                    },
                ],
            }],
        }
    }

    pub fn build_sticker_description_request(
        sticker_bytes: &[u8],
        mime_type: &str,
    ) -> GenerateContentRequest {
        let b64_data = BASE64_STANDARD.encode(sticker_bytes);
        let prompt = "Descreva esta figurinha/meme de WhatsApp de forma concisa e expressiva. Descreva o elemento visual, transcreva qualquer texto presente e explique com clareza o humor, a emoção e o contexto do meme. Retorne apenas a descrição direta, sem saudações.";

        GenerateContentRequest {
            contents: vec![Content {
                role: Some("user".to_string()),
                parts: vec![
                    Part::Text {
                        text: prompt.to_string(),
                    },
                    Part::InlineData {
                        inline_data: InlineData {
                            mime_type: mime_type.to_string(),
                            data: b64_data,
                        },
                    },
                ],
            }],
        }
    }

    async fn send_generate_content(
        &self,
        request_payload: &GenerateContentRequest,
    ) -> Result<String, GeminiError> {
        let api_key = self.api_key.as_deref().ok_or(GeminiError::MissingApiKey)?;
        let url = format!(
            "{}/v1beta/models/{}:generateContent?key={}",
            self.endpoint.trim_end_matches('/'),
            self.model,
            api_key
        );

        let response = self.client.post(&url).json(request_payload).send().await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(GeminiError::ApiError {
                status: status.as_u16(),
                body,
            });
        }

        let resp_data: GenerateContentResponse = response.json().await?;
        resp_data.extract_text().ok_or(GeminiError::EmptyResponse)
    }

    pub async fn transcribe_audio(
        &self,
        audio_bytes: &[u8],
        mime_type: &str,
    ) -> Result<String, GeminiError> {
        let req = Self::build_transcription_request(audio_bytes, mime_type);
        self.send_generate_content(&req).await
    }

    pub async fn describe_image(
        &self,
        image_bytes: &[u8],
        mime_type: &str,
    ) -> Result<String, GeminiError> {
        let req = Self::build_image_description_request(image_bytes, mime_type);
        self.send_generate_content(&req).await
    }

    pub async fn describe_sticker(
        &self,
        sticker_bytes: &[u8],
        mime_type: &str,
    ) -> Result<String, GeminiError> {
        let req = Self::build_sticker_description_request(sticker_bytes, mime_type);
        self.send_generate_content(&req).await
    }
}
