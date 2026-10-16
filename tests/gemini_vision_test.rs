use hendu::ai::gemini::GeminiClient;
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
fn test_build_image_description_request() {
    let image_bytes = b"fake-jpeg-image-bytes";
    let mime = "image/jpeg";

    let req = GeminiClient::build_image_description_request(image_bytes, mime);
    let json = serde_json::to_string(&req).expect("serialização deve funcionar");

    assert!(json.contains("image/jpeg"));
    assert!(json.contains("inline_data"));
    assert!(json.contains("audiodescrição"));
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
