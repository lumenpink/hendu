import os, sys, json, base64, requests

def get_env():
    env_file = os.path.expanduser("~/.env")
    data = {}
    if os.path.exists(env_file):
        with open(env_file, "r") as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith("#") and "=" in line:
                    k, v = line.split("=", 1)
                    data[k.strip()] = v.strip().strip("\"'")
    return data

def describe(file_path, prompt, mime_type):
    env = get_env()
    api_key = env.get("GEMINI_API_KEY")
    # Hendu usa gemini-2.5-flash ou o modelo configurado
    model = env.get("HENDU_GEMINI_MODEL", "gemini-2.5-flash")
    if not api_key:
        print("GEMINI_API_KEY não encontrada no .env", file=sys.stderr)
        return None

    with open(file_path, "rb") as f:
        b64 = base64.b64encode(f.read()).decode("utf-8")

    url = f"https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={api_key}"
    payload = {
        "contents": [{
            "parts": [
                {"text": prompt},
                {"inline_data": {"mime_type": mime_type, "data": b64}}
            ]
        }]
    }
    resp = requests.post(url, json=payload, timeout=30)
    if resp.status_code == 200:
        data = resp.json()
        try:
            return data["candidates"][0]["content"]["parts"][0]["text"].strip()
        except Exception as e:
            print("Erro parsing resposta:", e, data, file=sys.stderr)
            return None
    else:
        print(f"Erro API ({resp.status_code}):", resp.text, file=sys.stderr)
        return None

if __name__ == "__main__":
    if len(sys.argv) < 4:
        print("Uso: python descrever_exemplos.py <caminho_imagem> <mime_type> <tipo: foto|figurinha>")
        sys.exit(1)
    
    path, mime, kind = sys.argv[1], sys.argv[2], sys.argv[3]
    if kind == "foto":
        prompt = "Descreva esta imagem de forma objetiva, acolhedora e clara. Destaque o elemento principal, cenário, detalhes importantes e transcreva com exatidão qualquer texto visível na imagem. Retorne apenas a descrição direta, sem saudações ou comentários extras."
    else:
        prompt = "Descreva esta figurinha/meme de WhatsApp de forma concisa e expressiva. Descreva o elemento visual, transcreva qualquer texto presente e explique com clareza o humor, a emoção e o contexto do meme. Retorne apenas a descrição direta, sem saudações."
    
    res = describe(path, prompt, mime)
    print(res)
