# 🔐 KeyAI Desktop

**Sistema de Registro e Busca de Teclas com Privacidade**

KeyAI Desktop v1.0 é um aplicativo de registro de teclas (keylogger) local focado em privacidade, que captura, mascara PII automaticamente e permite busca híbrida (textual e semântica) no histórico de digitação. Todos os dados permanecem no dispositivo do usuário.

## ✨ Características Principais

- 🔒 **Privacidade Total**: Todos os dados permanecem locais, sem sincronização em nuvem
- 🛡️ **Mascaramento Automático de PII**: CPF, e-mail, telefone e outros dados sensíveis são mascarados automaticamente
- 🔍 **Busca Híbrida**: Combina busca textual (FTS5) e busca semântica (embeddings)
- 🗄️ **Banco Criptografado**: SQLite com SQLCipher para máxima segurança
- 🖥️ **Multiplataforma**: Windows, macOS (X11) e Linux (X11)
- ⚡ **Performance**: Latência de busca ≤150ms com 1M de palavras
- 🎨 **Interface Moderna**: UI responsiva construída com React + TypeScript

## 🏗️ Arquitetura

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│     Agente      │───▶│     Masker      │───▶│   Database      │
│  (Captura de    │    │   (Filtro PII)  │    │ (SQLite Cripto) │
│    Teclas)      │    │                 │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                                              ▲
         │              ┌─────────────────┐              │
         └─────────────▶│   Frontend      │──────────────┘
                        │  (React + TS)   │
                        └─────────────────┘
```

### Componentes

- **Agente (Rust)**: Captura eventos de teclado usando `rdev`
- **Masker (Rust)**: Aplica regex para mascarar PII em tempo real
- **Database (Rust)**: SQLite criptografado com FTS5 e sqlite-vec
- **Frontend (React)**: Interface do usuário moderna e responsiva

## 🚀 Instalação

### Pré-requisitos

- **Windows**: Windows 10/11 (x64)
- **macOS**: macOS 10.15+ (Intel & Apple Silicon)
- **Linux**: Ubuntu 20.04+ com X11 (Wayland não suportado)

### Download

Baixe a versão mais recente em [Releases](https://github.com/keyai/keyai-desktop/releases):

- **Windows**: `KeyAI-Desktop_x.x.x_x64.msi`
- **macOS**: `KeyAI-Desktop_x.x.x.dmg`
- **Linux**: `KeyAI-Desktop_x.x.x.AppImage` ou `keyai-desktop_x.x.x_amd64.deb`

### Instalação por Plataforma

#### Windows
1. Baixe o arquivo `.msi`
2. Execute como administrador
3. Siga o assistente de instalação
4. O aplicativo será iniciado automaticamente

#### macOS
1. Baixe o arquivo `.dmg`
2. Abra e arraste para a pasta Applications
3. Na primeira execução, vá em **System Preferences > Security & Privacy**
4. Autorize o aplicativo na seção **Accessibility**

#### Linux (Ubuntu/Debian)
```bash
# Usando .deb
sudo dpkg -i keyai-desktop_x.x.x_amd64.deb
sudo apt-get install -f  # Resolver dependências

# Usando AppImage
chmod +x KeyAI-Desktop_x.x.x.AppImage
./KeyAI-Desktop_x.x.x.AppImage
```

## 🛠️ Desenvolvimento

### Pré-requisitos de Desenvolvimento

- Rust 1.78+
- Node.js 20+
- npm ou yarn

#### Dependências por Sistema

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install -y \
    libwebkit2gtk-4.0-dev \
    libwebkit2gtk-4.1-dev \
    libappindicator3-dev \
    librsvg2-dev \
    patchelf \
    libx11-dev \
    libxdo-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev
```

**macOS:**
```bash
# Xcode Command Line Tools
xcode-select --install
```

**Windows:**
- Visual Studio 2019+ com C++ Build Tools
- WebView2 Runtime

### Configuração do Ambiente

```bash
# Clone o repositório
git clone https://github.com/keyai/keyai-desktop.git
cd keyai-desktop

# Instalar dependências do frontend
cd frontend
npm install
cd ..

# Verificar configuração Rust
cargo check
```

### Comandos de Desenvolvimento

```bash
# Desenvolvimento com hot-reload
npm run tauri dev

# Build de produção
npm run tauri build

# Testes
cargo test

# Benchmarks
cargo bench

# Linting
cargo clippy
cargo fmt
```

### Estrutura do Projeto

```
keyai-desktop/
├── Cargo.toml              # Configuração Rust
├── tauri.conf.json         # Configuração Tauri
├── build.rs                # Build script
├── src/                    # Código Rust (backend)
│   ├── main.rs            # Entry point
│   ├── agent/             # Captura de teclas
│   ├── masker/            # Mascaramento PII
│   ├── db/                # Persistência
│   ├── search/            # Busca híbrida
│   └── commands/          # Comandos Tauri
├── frontend/              # Frontend React
│   ├── src/
│   │   ├── App.tsx       # Componente principal
│   │   ├── components/   # Componentes React
│   │   └── types.ts      # Tipos TypeScript
│   ├── package.json
│   └── vite.config.ts
├── .github/workflows/     # CI/CD
└── tests/                 # Testes de integração
```

## 🔐 Segurança e Privacidade

### Princípios de Privacidade

1. **Dados Locais**: Nenhum dado é enviado para servidores externos
2. **Criptografia**: Banco de dados sempre criptografado com SQLCipher
3. **Mascaramento PII**: Informações sensíveis são mascaradas antes do armazenamento
4. **Sem Telemetria**: Nenhuma coleta de dados de uso

### Padrões de PII Mascarados

- **CPF**: `123.456.789-01` → `***.***.***-01`
- **E-mail**: `joao@exemplo.com` → `j***@exemplo.com`
- **Telefone**: `(11) 99999-1234` → `(***) ***-1234`
- **Cartão de Crédito**: `1234 5678 9012 3456` → `**** **** **** 3456`
- **RG**: `12.345.678-9` → `**.***.**-*`
- **CNPJ**: `12.345.678/0001-90` → `**.***.***/****-**`

### Configuração de Segurança

O aplicativo usa as seguintes configurações de segurança:

```sql
-- Criptografia do banco
PRAGMA key = 'keyai-desktop-secret-key';

-- Configurações de performance segura
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;
PRAGMA synchronous = NORMAL;
```

## 🔍 Funcionalidades de Busca

### Tipos de Busca

1. **Busca Textual (FTS5)**
   - Busca rápida em texto completo
   - Suporte a operadores: `AND`, `OR`, `NOT`, `*`
   - Exemplo: `email AND login`

2. **Busca Semântica (Embeddings)**
   - Busca baseada em significado
   - Encontra conteúdo relacionado mesmo sem palavras exatas
   - Exemplo: "senha" encontra "password", "login", etc.

3. **Busca Híbrida (RRF)**
   - Combina busca textual e semântica
   - Usa Reciprocal Rank Fusion (RRF)
   - Pesos configuráveis (padrão: 70% textual, 30% semântica)

### Exemplos de Uso

```typescript
// Busca textual
await invoke('search_text', { 
  query: 'email AND password', 
  limit: 50 
});

// Busca semântica
await invoke('search_semantic', { 
  query: 'authentication credentials', 
  limit: 50 
});

// Busca híbrida
await invoke('search_hybrid', { 
  query: 'login information',
  limit: 50,
  text_weight: 0.7,
  semantic_weight: 0.3
});
```

## 📊 Performance

### Metas de Performance (v1.0)

- **Latência de Busca (p95)**: ≤150ms (com 1M palavras)
- **Uso de CPU (idle)**: <3%
- **Cobertura de Testes**: >80% no core Rust

### Benchmarks

Execute benchmarks localmente:

```bash
cargo bench
```

Os resultados são salvos em `target/criterion/` e incluem:
- Latência de busca por tipo
- Throughput do agente de captura
- Performance do mascaramento PII

## 🧪 Testes

### Executar Testes

```bash
# Testes unitários
cargo test

# Testes com output detalhado
cargo test -- --nocapture

# Testes específicos
cargo test masker::tests
```

### Tipos de Teste

1. **Unitários**: Testam funções isoladas
2. **Integração**: Testam interação entre componentes
3. **E2E**: Simulam entrada do usuário via `rdev::simulate`

## 🚦 CI/CD

O projeto possui um sistema completo de CI/CD automatizado com GitHub Actions.

### 🔄 Workflows Implementados

- **CI Completo** - Linting, testes, análise de código
- **Auto Review** - Revisão automática de PRs com sugestões
- **Auto Fix** - Correções automáticas via comando `/autofix`
- **Security** - Análise contínua de vulnerabilidades
- **Release** - Build e deployment automatizados
- **Dependabot** - Atualização automática de dependências

### �️ Executar CI Localmente

```bash
# Linux/macOS
./scripts/ci-local.sh

# Windows PowerShell
.\scripts\ci-local.ps1
```

### 🤖 Comandos em PRs

```bash
# Em qualquer PR, comente:
/autofix          # Aplica todas as correções automáticas
/autofix format   # Apenas formatação
/autofix clippy   # Apenas correções do Clippy
```

### 📊 Matriz de Build

- **Windows**: `windows-latest`
- **macOS**: `macos-latest` (Intel + Apple Silicon)  
- **Linux**: `ubuntu-22.04`

### 📚 Documentação Completa

Veja [docs/CI_CD_DOCUMENTATION.md](docs/CI_CD_DOCUMENTATION.md) para:
- Configuração detalhada de cada workflow
- Troubleshooting e debug
- Métricas e monitoramento
- Melhores práticas

## 🐛 Solução de Problemas

### Problemas Comuns

#### macOS: "KeyAI Desktop não pode ser aberto"
```bash
# Remover quarentena
xattr -cr /Applications/KeyAI\ Desktop.app
```

#### Linux: Permissões de acesso
```bash
# Adicionar usuário ao grupo input
sudo usermod -a -G input $USER
# Reiniciar sessão
```

#### Windows: Antivírus bloqueia
- Adicione exceção para a pasta de instalação
- Use certificado EV para reduzir falsos positivos

### Logs e Debug

Logs são salvos em:
- **Windows**: `%APPDATA%/keyai-desktop/logs/`
- **macOS**: `~/Library/Application Support/keyai-desktop/logs/`
- **Linux**: `~/.local/share/keyai-desktop/logs/`

Para debug detalhado:
```bash
RUST_LOG=debug npm run tauri dev
```

## 📚 Documentação

Para informações detalhadas sobre o projeto, consulte nossa documentação completa:

- **[📖 Documentação Completa](docs/README.md)** - Índice de toda a documentação
- **[🤝 Guia de Contribuição](CONTRIBUTING.md)** - Como contribuir com o projeto
- **[🔒 Política de Segurança](SECURITY.md)** - Segurança e privacidade
- **[🏗️ Arquitetura](ARCHITECTURE.md)** - Arquitetura detalhada do sistema
- **[📡 API](docs/API.md)** - Documentação da API interna
- **[🚀 Deployment](DEPLOYMENT.md)** - Guia de deployment e distribuição

## 🤝 Contribuição

Agradecemos todas as contribuições! Por favor, leia nosso [Guia de Contribuição](CONTRIBUTING.md) detalhado para informações sobre:

- Configuração do ambiente de desenvolvimento
- Padrões de código e convenções
- Processo de review e merge
- Como reportar bugs e sugerir features

### Contribuição Rápida

1. Fork o repositório
2. Crie uma branch para sua feature (`git checkout -b feature/amazing-feature`)
3. Commit suas mudanças (`git commit -m 'Add amazing feature'`)
4. Push para a branch (`git push origin feature/amazing-feature`)
5. Abra um Pull Request

## 📝 Roadmap

### v1.1 (Próxima Release)
- [ ] Suporte para Wayland
- [ ] Customização de padrões PII
- [ ] Exportação de dados
- [ ] Melhorias na busca semântica

### v1.2 (Futuro)
- [ ] OCR para captura de texto
- [ ] Integração com extensões de navegador
- [ ] API para integrações
- [ ] Funcionalidades de colaboração

## 📄 Licença

Este projeto está licenciado sob a Licença MIT - veja o arquivo [LICENSE](LICENSE) para detalhes.

## 🙏 Agradecimentos

- [Tauri](https://tauri.app/) - Framework para aplicações desktop
- [rdev](https://github.com/Narsil/rdev) - Captura de eventos de input
- [SQLite](https://sqlite.org/) - Banco de dados embarcado
- [rust-bert](https://github.com/guillaume-be/rust-bert) - Embeddings em Rust

---

**⚠️ Aviso Legal**: Este software é destinado apenas para uso legítimo e autorizado. O usuário é responsável por garantir conformidade com leis locais de privacidade e uso de keyloggers.

**🔒 Privacidade**: KeyAI Desktop não coleta, armazena ou transmite dados pessoais para terceiros. Todos os dados permanecem locais no dispositivo do usuário. 