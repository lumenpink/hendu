# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- Remoção de campo `permissions:` incompatível com Forgejo Actions (closes #15).
- Configuração do compose para utilizar o registro soberano git.2lp.in (closes #13).

### Fixed
- Configuração de `[profile.release]` com LTO e strip, e linking estático `+crt-static` para musl libc (closes #12).

### Added
- Controle de acesso (ACL) com modo dona soberano e comandos sem `!` no PV (#9).
- Apresentação pedagógica com pronúncia (*rren-DÚ*) e significado Guarani (#9).
- Fluxo de solicitação de acesso com limite de 1 mensagem de apresentação (#9).
- Gestão granular de grupos nos modos `ondemand` e `automatic` (#9).
- Inicialização do repositório em conformidade com o padrão GOST.
