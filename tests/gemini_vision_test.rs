use hendu::ai::gemini::{GeminiClient, GeminiError};
use hendu::config::Config;
use std::collections::HashMap;

#[test]
fn test_config_vision_defaults() {
    let env: HashMap<String, String> = HashMap::new();
    let config = Config::from_lookup(|k| env.get(k).cloned()).unwrap();

    assert!(config.image_description_enabled);
    assert!(config.sticker_description_enabled);
}

#[test]
fn test_config_vision_flags_disabled() {
    let mut env = HashMap::new();
    env.insert(
        "HENDU_IMAGE_DESCRIPTION_ENABLED".to_string(),
        "false".to_string(),
    );
    env.insert(
        "HENDU_STICKER_DESCRIPTION_ENABLED".to_string(),
        "0".to_string(),
    );

    let config = Config::from_lookup(|k| env.get(k).cloned()).unwrap();

    assert!(!config.image_description_enabled);
    assert!(!config.sticker_description_enabled);
}

#[test]
fn test_build_image_description_request_various_mimes() {
    for mime in &["image/jpeg", "image/png", "image/webp"] {
        let image_bytes = b"fake-raw-image-bytes";
        let req = GeminiClient::build_image_description_request(image_bytes, mime);
        let json = serde_json::to_string(&req).expect("serialização deve funcionar");

        assert!(json.contains(mime));
        assert!(json.contains("inline_data"));
        assert!(json.contains("audiodescrição"));
    }
}

#[test]
fn test_build_sticker_description_request() {
    let sticker_bytes = b"fake-webp-sticker-bytes";
    let mime = "image/webp";

    let req = GeminiClient::build_sticker_description_request(sticker_bytes, mime);
    let json = serde_json::to_string(&req).expect("serialização deve funcionar");

    assert!(json.contains("image/webp"));
    assert!(json.contains("inline_data"));
    assert!(json.contains("figurinha"));
}

#[tokio::test]
async fn test_describe_image_missing_api_key_returns_error() {
    let client = GeminiClient::new(
        None,
        "gemini-3.5-flash-lite".to_string(),
        "https://generativelanguage.googleapis.com".to_string(),
    );

    let res = client.describe_image(b"img-bytes", "image/jpeg").await;
    match res {
        Err(GeminiError::MissingApiKey) => {}
        other => panic!("Esperado MissingApiKey, recebido: {:?}", other),
    }
}

#[tokio::test]
async fn test_describe_sticker_missing_api_key_returns_error() {
    let client = GeminiClient::new(
        None,
        "gemini-3.5-flash-lite".to_string(),
        "https://generativelanguage.googleapis.com".to_string(),
    );

    let res = client.describe_sticker(b"sticker-bytes", "image/webp").await;
    match res {
        Err(GeminiError::MissingApiKey) => {}
        other => panic!("Esperado MissingApiKey, recebido: {:?}", other),
    }
}
