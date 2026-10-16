use hendu::ai::gemini::{
    GeminiClient, GenerateContentRequest, GenerateContentResponse,
};
use hendu::config::Config;
use std::collections::HashMap;

#[test]
fn test_config_gemini_defaults() {
    let env = HashMap::new();
    let config = Config::from_lookup(|k| env.get(k).cloned()).unwrap();

    assert_eq!(config.gemini_model, "gemini-3.5-flash-lite");
    assert_eq!(
        config.gemini_endpoint,
        "https://generativelanguage.googleapis.com"
    );
    assert!(config.gemini_api_key.is_none());
}

#[test]
fn test_config_gemini_custom() {
    let mut env = HashMap::new();
    env.insert("GEMINI_API_KEY".to_string(), "test-secret-key".to_string());
    env.insert("GEMINI_MODEL".to_string(), "gemini-3.8-flash".to_string());
    env.insert(
        "GEMINI_ENDPOINT".to_string(),
        "https://custom-proxy.internal".to_string(),
    );

    let config = Config::from_lookup(|k| env.get(k).cloned()).unwrap();

    assert_eq!(config.gemini_api_key.as_deref(), Some("test-secret-key"));
    assert_eq!(config.gemini_model, "gemini-3.8-flash");
    assert_eq!(config.gemini_endpoint, "https://custom-proxy.internal");
}

#[test]
fn test_build_audio_transcription_request() {
    let audio_bytes = b"fake-ogg-opus-bytes";
    let mime_type = "audio/ogg";

    let req = GeminiClient::build_transcription_request(audio_bytes, mime_type);
    let json = serde_json::to_string(&req).expect("serialização deve funcionar");

    assert!(json.contains("audio/ogg"));
    assert!(json.contains("inline_data"));
    assert!(json.contains("Transcreva"));
}

#[test]
fn test_parse_gemini_success_response() {
    let raw_json = r#"{
        "candidates": [
            {
                "content": {
                    "parts": [
                        {
                            "text": "Olá pessoal, a reunião de amanhã foi transferida para as 15h."
                        }
                    ],
                    "role": "model"
                },
                "finishReason": "STOP"
            }
        ]
    }"#;

    let resp: GenerateContentResponse =
        serde_json::from_str(raw_json).expect("parsing deve funcionar");
    let text = resp.extract_text().expect("deve extrair texto");

    assert_eq!(
        text,
        "Olá pessoal, a reunião de amanhã foi transferida para as 15h."
    );
}

#[test]
fn test_parse_gemini_empty_response() {
    let raw_json = r#"{"candidates": []}"#;
    let resp: GenerateContentResponse =
        serde_json::from_str(raw_json).expect("parsing deve funcionar");
    assert!(resp.extract_text().is_none());
}
