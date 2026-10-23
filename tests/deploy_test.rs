use std::fs;
use std::path::Path;

#[test]
fn test_dockerfile_spec() {
    let dockerfile_path = Path::new("Dockerfile");
    assert!(
        dockerfile_path.exists(),
        "Dockerfile deve existir na raiz do repositorio"
    );

    let content = fs::read_to_string(dockerfile_path).expect("Falha ao ler Dockerfile");

    assert!(
        content.contains("AS builder") || content.contains("as builder"),
        "Dockerfile deve utilizar multi-stage build com stage builder"
    );
    assert!(
        content.contains("FROM scratch"),
        "Dockerfile deve utilizar imagem final scratch para conformidade GOST"
    );
    assert!(
        content.contains("ca-certificates.crt"),
        "Dockerfile deve copiar os certificados CA para comunicacao TLS HTTPS com a API Gemini"
    );
    assert!(
        content.contains("ENTRYPOINT [\"/hendu\"]"),
        "Dockerfile deve declarar ENTRYPOINT [\"/hendu\"]"
    );
    assert!(
        content.contains("USER 10001:10001"),
        "Dockerfile deve executar sob usuario nao-root (10001:10001) por seguranca"
    );
}

#[test]
fn test_compose_yaml_spec() {
    let compose_path = Path::new("compose.yaml");
    assert!(
        compose_path.exists(),
        "compose.yaml deve existir na raiz do repositorio"
    );

    let content = fs::read_to_string(compose_path).expect("Falha ao ler compose.yaml");

    assert!(
        content.contains("hendu:"),
        "compose.yaml deve declarar o servico hendu"
    );
    assert!(
        content.contains("./data:/data"),
        "compose.yaml deve montar o volume persistente ./data:/data"
    );
    assert!(
        content.contains(".env"),
        "compose.yaml deve carregar o arquivo .env"
    );
    assert!(
        content.contains("memory: 64M"),
        "compose.yaml deve limitar o consumo de memoria em 64M para conformidade GOST"
    );
}

#[test]
fn test_env_example_spec() {
    let env_path = Path::new(".env.example");
    assert!(
        env_path.exists(),
        ".env.example deve existir na raiz do repositorio"
    );

    let content = fs::read_to_string(env_path).expect("Falha ao ler .env.example");

    assert!(
        content.contains("GEMINI_API_KEY="),
        ".env.example deve conter GEMINI_API_KEY"
    );
    assert!(
        content.contains("HENDU_GROUP_MODE="),
        ".env.example deve conter HENDU_GROUP_MODE"
    );
    assert!(
        content.contains("HENDU_DB_PATH="),
        ".env.example deve conter HENDU_DB_PATH"
    );
    assert!(
        content.contains("gemini-3.5-flash-lite"),
        ".env.example deve recomendar o modelo oficial gemini-3.5-flash-lite"
    );
}

#[test]
fn test_dockerignore_spec() {
    let dockerignore_path = Path::new(".dockerignore");
    assert!(
        dockerignore_path.exists(),
        ".dockerignore deve existir na raiz do repositorio"
    );

    let content = fs::read_to_string(dockerignore_path).expect("Falha ao ler .dockerignore");

    assert!(
        content.contains("target"),
        ".dockerignore deve ignorar target/"
    );
    assert!(content.contains(".git"), ".dockerignore deve ignorar .git/");
}

#[test]
fn test_copy_humanizada_spec() {
    let site_path = Path::new("site/index.html");
    assert!(site_path.exists());
    let content = fs::read_to_string(site_path).expect("Falha ao ler site/index.html");
    let content_l = content.to_lowercase();
    assert!(
        !content_l.contains("soberan"),
        "site/index.html não deve conter 'soberano'"
    );
    assert!(
        !content_l.contains("rust puro"),
        "site/index.html não deve conter 'rust puro'"
    );
    assert!(
        !content_l.contains("na borda"),
        "site/index.html não deve conter 'na borda'"
    );
    assert!(
        !content_l.contains("wasm"),
        "site/index.html não deve conter 'wasm'"
    );
}
