# 🗺️ Roadmap de Evolução: Hendu

Este documento formaliza as linhas de pesquisa, desenvolvimento e expansão do **Hendu** para acessibilidade comunitária soberana no WhatsApp, orientadas pelas diretrizes de qualidade da **Norma GOST**.

---

## 📌 Ciclo de Lançamentos

```
[Fase 1: Fundação] ──▶ [Fase 2: Vídeo] ──▶ [Fase 3: Voz Neural] ──▶ [Fase 4: Linguagem Simples] ──▶ [Fase 5: LIBRAS]
    (Concluído)        (Em Estudo)         (Em Estudo)             (Em Planejamento)          (Pesquisa)
```

---

## ✅ Fase 1 — Fundação, Mídias Básicas & Controle Soberano *(Concluído)*
- [x] Conexão bidirecional com WhatsApp Web via WebSocket nativo (`whatsapp-rust`) e sessão SQLite (`whatsapp.db`).
- [x] Transcrição de áudios e mensagens de voz em português via Gemini Multimodal.
- [x] Audiodescrição de fotos e capturas de tela para pessoas cegas ou com baixa visão.
- [x] Audiodescrição contextual de figurinhas (stickers e memes), interpretando textos e humor visual.
- [x] Modos de operação em grupo: `automatic` (espontâneo) e `ondemand` (sob demanda com `!transcrever`/`!descrever`).
- [x] Controle de Acesso (ACL) e Modo Dona: reconhecimento soberano da administradora, comandos no privado sem prefixo `!` e normalização de 8/9 dígitos.
- [x] Fluxo acolhedor de apresentação para desconhecidos (1 mensagem encaminhada com pronúncia e significado Guarani).
- [x] Empacotamento estático em Rust musl sobre imagem `scratch` com consumo limitado a 64MB de RAM.

---

## 🚀 Fase 2 — Vídeos Curtos e Status
*Objetivo:* Acessibilizar vídeos encaminhados nos grupos de WhatsApp, que costumam misturar fala e ações visuais.
- [ ] **Extração Multimodal de Vídeo:**
  - Extração do áudio de vídeos curtos (`.mp4`, `.mov`) para transcrição textual.
  - Amostragem de frames-chave para audiodescrição da cena em movimento (ações dos personagens, legendas queimadas no vídeo e cenário).
- [ ] **Resumo de Vídeos Longos:**
  - Identificação de tópicos principais e minutagem para vídeos compartilhados.

---

## 🎙️ Fase 3 — Resposta por Voz (Text-to-Speech Neural / Resposta Falada)
*Objetivo:* Permitir que pessoas cegas que preferem ouvir possam receber a audiodescrição diretamente em áudio, sem depender de leitor de tela do celular.
- [ ] **Geração de Áudio Neural (Gemini TTS):**
  - Opção para responder com mensagem de áudio narrada em voz natural em português.
  - Comando ativador (ex: `!ouvir` ou configuração no perfil do usuário).
- [ ] **Codificação OGG Opus Nativa:**
  - Envio de áudios no formato de nota de voz nativa do WhatsApp (com waveform verde).

---

## 📄 Fase 4 — OCR e Linguagem Simples para Saúde e Cidadania
*Objetivo:* Apoiar pessoas idosas, com baixa escolaridade ou neurodivergentes na compreensão de documentos críticos.
- [ ] **Leitura Acessível de Bulas e Receitas Médicas:**
  - Recebimento de foto de receita médica manuscrita ou bula de medicamento.
  - Extração de posologia, horários e advertências em formato de tópicos claros.
- [ ] **Modo Linguagem Simples:**
  - Reescrever comunicados oficiais, formulários e textos jurídicos burocráticos em linguagem direta e acessível.
- [ ] **Leitura de PDFs Anexos:**
  - Extração de texto de documentos PDF anexados na conversa.

---

## 🤟 Fase 5 — Apoio e Glosas em LIBRAS
*Objetivo:* Construir pontes pedagógicas com a Língua Brasileira de Sinais.
- [ ] **Dicionário Descritivo de Sinais:**
  - Explicar sinais correspondentes a termos técnicos ou gírias regionais.
- [ ] **Formatação em Glosas de LIBRAS:**
  - Conversão de sentenças em português para a estrutura gramatical de LIBRAS (tópico-comentário) para facilitar o aprendizado.

---

## 🏛️ Governança e Contribuições
Todas as novas funcionalidades seguirão estritamente as **9 fases de DD-TDD da Norma GOST** (Ticket, Branch, Spec, Red Test, Green Implement, Refactor, Mutation Test, Quality Gate, Finish & Close).
Sugestões e novas demandas comunitárias podem ser formalizadas na forja:
👉 **[codeberg.org/lumenpink/hendu/issues](https://codeberg.org/lumenpink/hendu/issues)**
