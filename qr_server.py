import http.server
import socketserver
import subprocess
import os

PORT = 8098

HTML = """<!DOCTYPE html>
<html lang="pt-BR">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Hendu — Conectar WhatsApp</title>
    <style>
        body {
            font-family: system-ui, -apple-system, sans-serif;
            background: #0f172a;
            color: #f8fafc;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            min-height: 100vh;
            margin: 0;
            padding: 20px;
            text-align: center;
        }
        .card {
            background: #1e293b;
            padding: 30px;
            border-radius: 20px;
            box-shadow: 0 10px 25px rgba(0,0,0,0.5);
            max-width: 420px;
            width: 100%;
        }
        h1 { margin-top: 0; font-size: 1.5rem; color: #38bdf8; }
        p { color: #94a3b8; font-size: 0.95rem; line-height: 1.5; }
        .qr-wrapper {
            background: white;
            padding: 16px;
            border-radius: 12px;
            display: inline-block;
            margin: 20px 0;
        }
        img { display: block; width: 280px; height: 280px; }
        .badge {
            background: #0369a1;
            color: #e0f2fe;
            padding: 4px 12px;
            border-radius: 999px;
            font-size: 0.8rem;
            font-weight: bold;
        }
    </style>
</head>
<body>
    <div class="card">
        <span class="badge">Norma GOST</span>
        <h1>🌿 Hendu Bot WhatsApp</h1>
        <p>Aponte a câmera do WhatsApp para conectar:</p>
        <div class="qr-wrapper">
            <img id="qr-img" src="/qr.png" alt="QR Code WhatsApp Hendu">
        </div>
        <p id="timer" style="font-size: 0.8rem; color: #64748b;">Atualizando automaticamente a cada 4 segundos...</p>
    </div>

    <script>
        setInterval(() => {
            const img = document.getElementById('qr-img');
            img.src = '/qr.png?t=' + Date.now();
        }, 4000);
    </script>
</body>
</html>
"""

class Handler(http.server.SimpleHTTPRequestHandler):
    def do_GET(self):
        if self.path == "/" or self.path.startswith("/index"):
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.end_headers()
            self.wfile.write(HTML.encode("utf-8"))
        elif self.path.startswith("/qr.png"):
            try:
                out = subprocess.check_output(
                    "docker compose logs hendu | grep '2@' | tail -n 1 | sed 's/.*2@/2@/'",
                    shell=True,
                    cwd="/home/lumen/src/hendu"
                ).decode("utf-8").strip()
                
                if out:
                    qr_file = "/tmp/hendu_qr.png"
                    subprocess.run(["qrencode", "-o", qr_file, "-s", "8", out], check=True)
                    with open(qr_file, "rb") as f:
                        data = f.read()
                    self.send_response(200)
                    self.send_header("Content-Type", "image/png")
                    self.send_header("Cache-Control", "no-cache, no-store, must-revalidate")
                    self.end_headers()
                    self.wfile.write(data)
                else:
                    self.send_response(404)
                    self.end_headers()
            except Exception as e:
                self.send_response(500)
                self.end_headers()
                self.wfile.write(str(e).encode())
        else:
            self.send_response(404)
            self.end_headers()

if __name__ == "__main__":
    socketserver.TCPServer.allow_reuse_address = True
    with socketserver.TCPServer(("", PORT), Handler) as httpd:
        print(f"Serving at http://localhost:{PORT}")
        httpd.serve_forever()
