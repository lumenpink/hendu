# Especificação Técnica: Controle de Acesso (ACL), Modo Dona e Gestão Granular por Grupo

- **Ticket:** #9
- **Módulo:** `hendu::acl`
- **Autoridade:** Gosplan GOST 1.4.0 / Perfil Kollontai
- **Status:** Aprovado para Implementação

---

## 1. Escopo e Objetivos
1. **Identificação da Dona:**
   - Número soberano configurável via `HENDU_OWNER_PHONE` (padrão: `5548991381979`).
   - Normalização de números brasileiros: trata números de 8 e 9 dígitos após DDD (ex: `5548991381979` == `554891381979`).
   - Suporte a JID de usuário (`...@s.whatsapp.net`) e JID de privacidade LID (`...@lid`), resolvendo via tabela `lid_pn_mapping` no SQLite.
   - Reconhecimento automático de mensagens enviadas pelo próprio bot (`is_from_me = true`).

2. **Ergonomia no PV da Dona:**
   - Comandos enviados no chat privado pela dona não exigem prefixo `!`:
     - `permitir` / `permitir ondemand` / `permitir automatic`
     - `bloquear` / `recusar`
     - `modo ondemand` / `modo automatic`
     - `lista`
     - `status`
     - `ajuda`
   - O prefixo `!` permanece suportado opcionalmente (ex: `!permitir`).

3. **Ciclo de Vida e Estados de Acesso (ACL):**
   - **Desconhecido (`Unknown`):**
     - Ao enviar mensagem no PV ou falar em grupo não registrado, Hendu responde com mensagem de acolhimento explicando sua missão, origem do nome (Guarani: ouvir, compreender, prestar atenção) e pronúncia (*rren-DÚ*), sem revelar o telefone da dona.
     - Abre janela para o remetente enviar **uma única mensagem de apresentação**.
     - Transiciona para `AwaitingIntro`.
   - **Aguardando Apresentação (`AwaitingIntro`):**
     - A próxima mensagem do remetente é capturada como texto de apresentação.
     - Hendu confirma o recebimento ao remetente e notifica a dona no PV com os detalhes e citação para resposta rápida.
     - Transiciona para `PendingApproval`.
   - **Pendente de Aprovação (`PendingApproval`):**
     - Silêncio absoluto para o remetente (evita spam).
   - **Autorizado (`Approved`):**
     - No caso de contato: pode usar comandos e recursos de acessibilidade.
     - No caso de grupo: opera no modo configurado (`ondemand` ou `automatic`).
   - **Bloqueado (`Blocked`):**
     - Totalmente ignorado.

4. **Gestão de Grupos:**
   - Cada grupo possui configuração independente salva no banco (`ondemand` ou `automatic`).
   - Modo padrão de ativação de grupo: `ondemand`.

5. **Persistência SQLite:**
   - Tabelas gerenciadas pelo módulo `acl`:
     - `acl_contacts`: `phone_canonical TEXT PRIMARY KEY`, `jid TEXT`, `lid TEXT`, `status TEXT`, `intro_text TEXT`, `created_at INTEGER`, `updated_at INTEGER`.
     - `acl_groups`: `group_jid TEXT PRIMARY KEY`, `name TEXT`, `status TEXT`, `mode TEXT`, `intro_text TEXT`, `created_at INTEGER`, `updated_at INTEGER`.
