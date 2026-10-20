# 🌿 Hendu

> **Bot de WhatsApp em Rust para Acessibilidade Comunitária**  
> *Transcrição de mensagens de áudio para pessoas surdas e audiodescrição de fotos e figurinhas para pessoas cegas ou de baixa visão.*

[![Conformidade: GOST 1.5.0](https://img.shields.io/badge/GOST-1.5.0-red.svg)](.gost/GOST.md)
[![SemVer 2.0.0](https://img.shields.io/badge/SemVer-2.0.0-green.svg)](VERSION)
[![Licença: GPL-3.0](https://img.shields.io/badge/Licen%C3%A7a-GPL--3.0-blue.svg)](LICENSE)
[![Forja: Codeberg](https://img.shields.io/badge/Forja-Codeberg-2185d0.svg)](https://codeberg.org/lumenpink/hendu)

---

## 📖 Origem e Missão

* **Pronúncia:** *rren-DÚ* (oxítona).
* **Etimologia:** Palavra de origem Guarani que expressa os atos conjugados de *escutar, ouvir, prestar atenção e compreender*.
* **Propósito:** Oferecer autonomia comunicacional e quebrar barreiras no aplicativo de mensagens mais utilizado do Brasil, sem intermediários corporativos, sem rastreamento de dados e com custódia soberana.

---

## ✨ Funcionalidades Principais

* 🎙️ **Transcrição de Voz:** Converte notas de voz (PTT) e arquivos de áudio em texto formatado em português.
* 🖼️ **Audiodescrição de Fotos:** Analisa imagens e descreve pessoas, objetos, textos em cartazes, cores e cenários para leitores de tela.
* 🎭 **Audiodescrição de Figurinhas & Memes:** Interpreta o humor, as piadas visuais e os textos presentes em stickers.
* 🛡️ **Modo Dona & Controle de Acesso (ACL):**
  * Comandos administrativos executados sem necessidade do prefixo `!` no privado da administradora.
  * Normalização inteligente de números telefônicos brasileiros (reconhecendo formatos com 8 e 9 dígitos).
  * Solicitação de acesso acolhedora para contatos e grupos desconhecidos (limite de 1 mensagem de apresentação e silenciamento até aprovação).
* 👥 **Modos Granulares de Grupo:** Cada grupo autorizado pode operar em modo `ondemand` (sob demanda com comandos) ou `automatic` (espontâneo).
* 📦 **Ultra-leveza & Eficiência:** Binário estático Musl rodando sobre imagem `scratch` com consumo inferior a 64MB de memória RAM.

---

## 💬 Comandos Disponíveis

| Comando | Contexto | Descrição |
| :--- | :--- | :--- |
| `!ajuda` / `ajuda` | Qualquer chat | Exibe o manual rápido de apoio à acessibilidade. |
| `!status` / `status` | Qualquer chat | Tempo ativo (uptime) e total de mídias processadas. |
| `!sobre` / `sobre` | Qualquer chat | Significado Guarani, missão social e licença. |
| `!ping` / `ping` | Qualquer chat | Teste de conectividade e latência (`pong`). |
| `!transcrever` | Respondendo a um áudio | Força a transcrição do áudio citado em texto. |
| `!descrever` / `!ver` | Respondendo a foto/sticker | Audiodescreve a imagem ou meme citado. |
| `permitir [modo]` | PV da Administradora | Autoriza o contato ou grupo (modos `ondemand` ou `automatic`). |
| `bloquear` / `recusar` | PV da Administradora | Revoga a autorização do contato ou grupo. |
| `modo [ondemand\|auto]` | Dentro de grupo / PV | Alterna o comportamento do grupo entre sob demanda e automático. |
| `lista` | PV da Administradora | Exibe a lista completa de permissões salvas na ACL. |

---

## 🚀 Como Executar

### 1. Pré-requisitos
* Docker e Docker Compose instalados no host.
* Uma chave de API do Google Gemini (`GEMINI_API_KEY`).
* Número do WhatsApp onde o Hendu será emparelhado.

### 2. Configuração de Ambiente (`.env`)
Copie o modelo de variáveis:
```bash
cp .env.example .env
```
Preencha o arquivo com suas credenciais:
```bash
GEMINI_API_KEY=sua_chave_aqui
HENDU_GEMINI_MODEL=gemini-3.5-flash-lite
HENDU_OWNER_PHONE=5548991381979
HENDU_DB_PATH=/data/whatsapp.db
HENDU_ACL_PATH=/data/acl.json
HENDU_GROUP_MODE=ondemand
RUST_LOG=info
```

### 3. Iniciar o Serviço
```bash
docker compose up -d
```

### 4. Emparelhamento WhatsApp
Visualize os logs para exibir o QR code:
```bash
docker compose logs -f hendu
```
No celular, acesse **Aparelhos conectados → Conectar aparelho** e aponte para o QR code no terminal ou na interface local.

---

## 📋 Conformidade com a Norma GOST

Este projeto adota e cumpre integralmente a norma **[GOST](https://codeberg.org/lumenpink/gost)** (Gosplan Standard for Technical Software) na versão **1.5.0**:
* Todas as alterações de código e novas funcionalidades seguem o ciclo rigoroso de **9 fases de DD-TDD** (Ticket, Branch, Spec, Red Test, Green Implement, Refactor, Mutation Test, Quality Gate, Finish & Close).
* Validação contínua através da suíte de ferramentas de qualidade (`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` e `cccp check --strict`).

---

## 📄 Licenças

Este projeto é software livre licenciado sob a [GNU General Public License v3.0](LICENSE) (GPL-3.0-or-later).

| Componente | Licença | Arquivo |
| :--- | :--- | :--- |
| **Código-fonte e binários** | GPL-3.0-or-later | [LICENSE](LICENSE) |
| **Logotipos e marcas** | CC-BY-ND-4.0 | [LICENSE-LOGO](LICENSE-LOGO) |
| **Mídias e documentação** | CC-BY-SA-4.0 | [LICENSE-MEDIA](LICENSE-MEDIA) |
