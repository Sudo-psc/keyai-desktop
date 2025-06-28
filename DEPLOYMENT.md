# 🚀 Guia de Deployment - KeyAI Desktop

## 📋 Índice

- [Visão Geral](#visão-geral)
- [Pré-requisitos](#pré-requisitos)
- [Configuração do Ambiente](#configuração-do-ambiente)
- [Build Local](#build-local)
- [CI/CD Pipeline](#cicd-pipeline)
- [Assinatura de Código](#assinatura-de-código)
- [Distribuição](#distribuição)
- [Monitoramento](#monitoramento)
- [Rollback](#rollback)

## 🎯 Visão Geral

O KeyAI Desktop utiliza uma estratégia de deployment automatizada via GitHub Actions, com builds multiplataforma e distribuição através de releases no GitHub. O processo inclui assinatura de código para Windows e macOS, garantindo a autenticidade dos binários.

### Estratégia de Release

- **Continuous Deployment**: Builds automáticos na branch `main`
- **Semantic Versioning**: Versionamento semântico (v1.0.0)
- **Multi-platform**: Windows, macOS (Intel + Apple Silicon), Linux
- **Code Signing**: Assinatura digital para Windows e macOS
- **Automated Testing**: Testes automáticos antes do deploy

## 🛠️ Pré-requisitos

### Ferramentas Necessárias

- **Git**: Para versionamento
- **Rust**: 1.78+ com toolchain stable
- **Node.js**: 20+ com npm
- **Tauri CLI**: Para builds locais

### Certificados de Assinatura

#### Windows
- **Certificado EV** (Extended Validation)
- Formato: `.pfx` ou `.p12`
- Provedor recomendado: DigiCert, Sectigo

#### macOS
- **Apple Developer Account**
- **Developer ID Application Certificate**
- **Developer ID Installer Certificate** (para .pkg)

### Segredos do GitHub

Configure os seguintes segredos no repositório:

```
WINDOWS_CERTIFICATE          # Certificado .pfx em Base64
WINDOWS_CERTIFICATE_PASSWORD # Senha do certificado
APPLE_CERTIFICATE            # Certificado .p12 em Base64  
APPLE_CERTIFICATE_PASSWORD   # Senha do certificado
APPLE_ID                     # Email da conta Apple Developer
APPLE_PASSWORD               # Senha específica do app
APPLE_TEAM_ID                # ID do time Apple Developer
```

## 🔧 Configuração do Ambiente

### 1. Configuração Local

```bash
# Clone o repositório
git clone https://github.com/keyai/keyai-desktop.git
cd keyai-desktop

# Instalar Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Instalar Node.js dependencies
cd frontend
npm install
cd ..

# Instalar Tauri CLI
cargo install tauri-cli
```

### 2. Configuração de Dependências por Plataforma

#### Ubuntu/Debian
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
    libxcb-xfixes0-dev \
    pkg-config \
    build-essential
```

#### macOS
```bash
# Instalar Xcode Command Line Tools
xcode-select --install

# Verificar certificados
security find-identity -v -p codesigning
```

#### Windows
```powershell
# Instalar Visual Studio Build Tools
# Baixar de: https://visualstudio.microsoft.com/downloads/

# Verificar certificados
Get-ChildItem -Path Cert:\CurrentUser\My
```

## 🏗️ Build Local

### Build de Desenvolvimento

```bash
# Build e execução em modo desenvolvimento
npm run tauri dev

# Build apenas do frontend
cd frontend
npm run build
cd ..

# Build apenas do backend
cargo build
```

### Build de Produção

```bash
# Build completo de produção
npm run tauri build

# Build com logs detalhados
RUST_LOG=debug npm run tauri build

# Build para arquitetura específica
cargo build --release --target x86_64-pc-windows-msvc
```

### Configuração de Build

**Cargo.toml** - Otimizações de release:
```toml
[profile.release]
opt-level = 3          # Otimização máxima
lto = true            # Link Time Optimization
codegen-units = 1     # Melhor otimização
panic = "abort"       # Menor tamanho do binário
strip = true          # Remove símbolos de debug
```

**tauri.conf.json** - Configuração do bundle:
```json
{
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devPath": "http://localhost:1420",
    "distDir": "../dist"
  },
  "bundle": {
    "active": true,
    "targets": ["msi", "dmg", "appimage", "deb"],
    "identifier": "com.keyai.desktop",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "windows": {
      "certificateThumbprint": null,
      "digestAlgorithm": "sha256",
      "timestampUrl": "http://timestamp.sectigo.com"
    },
    "macOS": {
      "entitlements": null,
      "exceptionDomain": "",
      "frameworks": [],
      "providerShortName": null,
      "signingIdentity": null
    }
  }
}
```

## 🔄 CI/CD Pipeline

### Workflow Principal (.github/workflows/release.yml)

```yaml
name: Release

on:
  push:
    branches: [main]
    tags: ['v*']
  workflow_dispatch:

jobs:
  build-and-release:
    strategy:
      fail-fast: false
      matrix:
        platform:
          - name: Windows
            os: windows-latest
            target: x86_64-pc-windows-msvc
            bundle: msi
          - name: macOS Intel
            os: macos-latest
            target: x86_64-apple-darwin
            bundle: dmg
          - name: macOS Apple Silicon
            os: macos-latest
            target: aarch64-apple-darwin
            bundle: dmg
          - name: Linux
            os: ubuntu-22.04
            target: x86_64-unknown-linux-gnu
            bundle: appimage,deb

    runs-on: ${{ matrix.platform.os }}
    
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: npm
          cache-dependency-path: frontend/package-lock.json

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.platform.target }}

      - name: Install Linux dependencies
        if: matrix.platform.os == 'ubuntu-22.04'
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.0-dev \
            libappindicator3-dev librsvg2-dev patchelf

      - name: Cache Rust dependencies
        uses: swatinem/rust-cache@v2
        with:
          key: ${{ matrix.platform.target }}

      - name: Install frontend dependencies
        run: |
          cd frontend
          npm ci

      - name: Build frontend
        run: |
          cd frontend
          npm run build

      - name: Setup code signing (Windows)
        if: matrix.platform.os == 'windows-latest'
        run: |
          echo "${{ secrets.WINDOWS_CERTIFICATE }}" | base64 -d > certificate.pfx
          echo "WINDOWS_CERTIFICATE_FILE=certificate.pfx" >> $GITHUB_ENV

      - name: Setup code signing (macOS)
        if: matrix.platform.os == 'macos-latest'
        run: |
          echo "${{ secrets.APPLE_CERTIFICATE }}" | base64 -d > certificate.p12
          security create-keychain -p keychain-password build.keychain
          security default-keychain -s build.keychain
          security unlock-keychain -p keychain-password build.keychain
          security import certificate.p12 -k build.keychain -P "${{ secrets.APPLE_CERTIFICATE_PASSWORD }}" -T /usr/bin/codesign
          security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k keychain-password build.keychain

      - name: Build and sign application
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          WINDOWS_CERTIFICATE_PASSWORD: ${{ secrets.WINDOWS_CERTIFICATE_PASSWORD }}
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}
          APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
        with:
          tagName: v__VERSION__
          releaseName: KeyAI Desktop v__VERSION__
          releaseBody: |
            ## Novidades nesta versão
            
            ### ✨ Novas Funcionalidades
            - Busca híbrida melhorada
            - Interface mais responsiva
            - Melhor performance na captura
            
            ### 🐛 Correções
            - Correção de vazamento de memória
            - Melhoria na estabilidade
            
            ### 📦 Downloads
            - **Windows**: KeyAI-Desktop_${{ github.ref_name }}_x64.msi
            - **macOS**: KeyAI-Desktop_${{ github.ref_name }}.dmg
            - **Linux**: KeyAI-Desktop_${{ github.ref_name }}.AppImage
          releaseDraft: false
          prerelease: false
          includeUpdaterJson: true
```

### Workflow de Testes

```yaml
name: Tests

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    
    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        
      - name: Cache dependencies
        uses: swatinem/rust-cache@v2
        
      - name: Run tests
        run: cargo test --verbose
        
      - name: Run clippy
        run: cargo clippy -- -D warnings
        
      - name: Check formatting
        run: cargo fmt --all -- --check
        
      - name: Security audit
        run: |
          cargo install cargo-audit
          cargo audit
```

## 🔐 Assinatura de Código

### Windows (Authenticode)

```bash
# Assinar manualmente (para debug)
signtool sign /f certificate.pfx /p password /t http://timestamp.sectigo.com /fd sha256 keyai-desktop.exe

# Verificar assinatura
signtool verify /pa keyai-desktop.exe
```

### macOS (Developer ID)

```bash
# Assinar aplicação
codesign --sign "Developer ID Application: Your Name" --verbose KeyAI\ Desktop.app

# Verificar assinatura
codesign --verify --verbose KeyAI\ Desktop.app

# Notarização (automática via tauri-action)
xcrun notarytool submit KeyAI-Desktop.dmg --apple-id $APPLE_ID --password $APPLE_PASSWORD --team-id $APPLE_TEAM_ID
```

### Linux (GPG)

```bash
# Gerar chave GPG
gpg --full-generate-key

# Assinar arquivo
gpg --armor --detach-sign KeyAI-Desktop.AppImage

# Verificar assinatura
gpg --verify KeyAI-Desktop.AppImage.asc KeyAI-Desktop.AppImage
```

## 📦 Distribuição

### GitHub Releases

Os binários são automaticamente publicados no GitHub Releases com:

- **Assets**: Instaladores para todas as plataformas
- **Checksums**: SHA256 para verificação de integridade
- **Release Notes**: Changelog automático
- **Updater JSON**: Para atualizações automáticas

### Estrutura de Release

```
KeyAI Desktop v1.0.0/
├── KeyAI-Desktop_1.0.0_x64.msi           # Windows Installer
├── KeyAI-Desktop_1.0.0_x64.msi.sig       # Assinatura Windows
├── KeyAI-Desktop_1.0.0.dmg               # macOS Disk Image
├── KeyAI-Desktop_1.0.0_aarch64.dmg       # macOS Apple Silicon
├── KeyAI-Desktop_1.0.0.AppImage          # Linux AppImage
├── keyai-desktop_1.0.0_amd64.deb         # Debian Package
├── KeyAI-Desktop_1.0.0.AppImage.sig      # Assinatura Linux
├── latest.json                           # Updater manifest
└── CHECKSUMS.txt                         # SHA256 checksums
```

### Canais de Distribuição

1. **GitHub Releases** (Principal)
   - Download direto
   - Atualizações automáticas
   - Verificação de integridade

2. **Microsoft Store** (Futuro)
   - Distribuição Windows
   - Atualizações automáticas
   - Sandboxing adicional

3. **Mac App Store** (Futuro)
   - Distribuição macOS
   - Atualizações automáticas
   - Notarização obrigatória

4. **Linux Package Repositories** (Futuro)
   - APT (Ubuntu/Debian)
   - RPM (Fedora/RHEL)
   - AUR (Arch Linux)

## 📊 Monitoramento

### Métricas de Release

```yaml
# .github/workflows/metrics.yml
name: Release Metrics

on:
  release:
    types: [published]

jobs:
  metrics:
    runs-on: ubuntu-latest
    steps:
      - name: Collect download stats
        run: |
          curl -H "Authorization: token ${{ secrets.GITHUB_TOKEN }}" \
            "https://api.github.com/repos/keyai/keyai-desktop/releases/latest" \
            | jq '.assets[].download_count'
            
      - name: Update metrics dashboard
        run: |
          # Script para atualizar dashboard de métricas
```

### Health Checks

```rust
// src/health.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthStatus {
    pub version: String,
    pub build_time: String,
    pub git_commit: String,
    pub database_ok: bool,
    pub agent_running: bool,
}

impl HealthStatus {
    pub fn check() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            build_time: env!("BUILD_TIME").to_string(),
            git_commit: env!("GIT_COMMIT").to_string(),
            database_ok: Database::health_check().is_ok(),
            agent_running: Agent::is_running(),
        }
    }
}
```

### Telemetria (Opcional)

```rust
// Telemetria anônima para melhorar o produto
#[derive(Debug, Serialize)]
pub struct TelemetryData {
    pub version: String,
    pub platform: String,
    pub arch: String,
    pub usage_stats: UsageStats,
    pub performance_metrics: PerformanceMetrics,
    pub error_counts: ErrorCounts,
}

// Enviado apenas se o usuário optar por participar
```

## 🔄 Rollback

### Estratégia de Rollback

1. **Detecção de Problemas**
   - Monitoramento automático
   - Relatórios de usuários
   - Métricas de erro

2. **Processo de Rollback**
   ```bash
   # Reverter release no GitHub
   gh release delete v1.0.1 --yes
   
   # Recriar release com versão anterior
   gh release create v1.0.0-hotfix --title "Hotfix Release"
   ```

3. **Comunicação**
   - Notificação aos usuários
   - Atualização da documentação
   - Post-mortem interno

### Rollback Automático

```yaml
# .github/workflows/rollback.yml
name: Automatic Rollback

on:
  workflow_run:
    workflows: ["Release"]
    types: [completed]

jobs:
  check-release:
    if: ${{ github.event.workflow_run.conclusion == 'failure' }}
    runs-on: ubuntu-latest
    steps:
      - name: Rollback failed release
        run: |
          # Script para rollback automático
          echo "Rolling back failed release..."
```

## 🚨 Troubleshooting

### Problemas Comuns

#### 1. Falha na Assinatura de Código

```bash
# Windows: Verificar certificado
certutil -dump certificate.pfx

# macOS: Verificar keychain
security find-identity -v -p codesigning
```

#### 2. Falha no Build

```bash
# Limpar cache
cargo clean
rm -rf target/
rm -rf frontend/node_modules/

# Rebuild
npm run tauri build
```

#### 3. Dependências Faltando

```bash
# Ubuntu: Instalar dependências
sudo apt-get install -y $(cat .github/workflows/release.yml | grep "apt-get install" -A 5 | grep -o "lib[a-z0-9-]*")

# macOS: Reinstalar Xcode tools
sudo xcode-select --reset
xcode-select --install
```

### Logs de Debug

```bash
# Build com logs detalhados
RUST_LOG=debug npm run tauri build

# Verificar logs do GitHub Actions
gh run list --workflow=release.yml
gh run view <run-id> --log
```

## 📋 Checklist de Release

### Pré-Release
- [ ] Todos os testes passando
- [ ] Documentação atualizada
- [ ] Changelog preparado
- [ ] Certificados válidos
- [ ] Secrets configurados

### Release
- [ ] Tag criada no Git
- [ ] Pipeline executado com sucesso
- [ ] Binários assinados
- [ ] Release notes publicadas
- [ ] Checksums verificados

### Pós-Release
- [ ] Downloads funcionando
- [ ] Atualizações automáticas testadas
- [ ] Métricas coletadas
- [ ] Feedback dos usuários monitorado
- [ ] Hotfixes aplicados se necessário

---

Este guia cobre todos os aspectos do deployment do KeyAI Desktop. Para dúvidas específicas sobre o processo de release, consulte a documentação do Tauri ou abra uma issue no repositório. 