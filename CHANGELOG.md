# 📝 Changelog

Todas as mudanças notáveis neste projeto serão documentadas neste arquivo.

O formato é baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.0.0/),
e este projeto adere ao [Versionamento Semântico](https://semver.org/lang/pt-BR/).

## [Unreleased]

### 📚 Documentação
- Aprimoramento completo da documentação do projeto
- Adicionado guia detalhado de contribuição (CONTRIBUTING.md)
- Criada política de segurança abrangente (SECURITY.md)
- Documentação completa da arquitetura do sistema (ARCHITECTURE.md)
- Documentação da API interna (docs/API.md)
- Guia de deployment e distribuição (DEPLOYMENT.md)
- Índice organizado da documentação (docs/README.md)

### ✨ Melhorias
- Estrutura de documentação bem organizada
- Links cruzados entre documentos
- Padrões consistentes de formatação
- Exemplos de código detalhados

## [1.0.0] - 2024-01-15

### ✨ Adicionado
- Sistema de captura de teclas multiplataforma (Windows, macOS X11, Linux X11)
- Mascaramento automático de PII (CPF, email, telefone, cartão de crédito)
- Banco de dados local criptografado com SQLCipher
- Busca híbrida combinando FTS5 e busca semântica
- Interface moderna em React + TypeScript
- Arquitetura baseada em Tauri para performance e segurança

### 🔒 Segurança
- Criptografia AES-256 para banco de dados
- Mascaramento automático de informações sensíveis
- Processamento 100% local (sem dados enviados para cloud)
- Gestão segura de memória com zeroização

### 🏗️ Arquitetura
- Backend em Rust para performance e segurança de memória
- Frontend em React com TypeScript para type safety
- Comunicação via canais MPSC entre componentes
- Busca semântica com embeddings locais usando rust-bert

### 📦 Distribuição
- Instaladores nativos (.msi, .dmg, .AppImage, .deb)
- Assinatura de código para Windows e macOS
- CI/CD automatizado via GitHub Actions
- Atualizações automáticas via Tauri updater

### 🧪 Testes
- Testes unitários e de integração
- Benchmarks de performance com criterion
- Testes E2E com simulação de eventos

## [0.1.0] - 2024-01-01

### ✨ Adicionado
- Configuração inicial do projeto
- Estrutura básica do Tauri
- Setup do frontend React
- Configuração do backend Rust
- CI/CD básico

---

## 📋 Tipos de Mudanças

- **✨ Adicionado** para novas funcionalidades
- **🔄 Modificado** para mudanças em funcionalidades existentes
- **🗑️ Removido** para funcionalidades removidas
- **🐛 Corrigido** para correções de bugs
- **🔒 Segurança** para vulnerabilidades corrigidas
- **📚 Documentação** para mudanças na documentação
- **🧪 Testes** para adições ou mudanças em testes
- **🏗️ Arquitetura** para mudanças na arquitetura
- **📦 Distribuição** para mudanças no processo de build/release
- **⚡ Performance** para melhorias de performance

## 🔗 Links

- [Repositório](https://github.com/keyai/keyai-desktop)
- [Issues](https://github.com/keyai/keyai-desktop/issues)
- [Releases](https://github.com/keyai/keyai-desktop/releases)
- [Documentação](docs/README.md) 