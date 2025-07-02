# 📚 Documentação do Sistema CI/CD - KeyAI Desktop

## 🎯 Visão Geral

O sistema de CI/CD do KeyAI Desktop foi projetado para automatizar completamente o processo de desenvolvimento, teste, revisão e deployment, garantindo alta qualidade e segurança do código.

## 🔄 Workflows Implementados

### 1. **CI - Integração Contínua** (`ci.yml`)

**Trigger**: Push em branches de desenvolvimento, PRs para main/develop

**Funcionalidades**:
- ✅ Linting de código Rust e TypeScript
- ✅ Testes unitários multi-plataforma
- ✅ Testes de integração
- ✅ Análise de código
- ✅ Build de teste em todas as plataformas

**Jobs principais**:
```yaml
- lint-rust      # Formatação e Clippy
- lint-frontend  # ESLint e TypeScript
- test-rust      # Testes em Windows/Linux/macOS
- test-frontend  # Testes React + Coverage
- integration    # Testes E2E
- code-analysis  # Audit, unsafe code, licenses
```

### 2. **Auto Review** (`auto-review.yml`)

**Trigger**: Abertura ou atualização de PRs

**Funcionalidades**:
- 📊 Análise de complexidade de código
- 🔒 Revisão de segurança automática
- 🔍 Detecção de código duplicado
- 🤖 Sugestões automáticas de melhorias
- ⚡ Análise de performance

**Comentários automáticos em PRs**:
- Checklists específicos por linguagem
- Métricas de complexidade
- Sugestões de boas práticas

### 3. **Auto Fix** (`auto-fix.yml`)

**Trigger**: 
- Comentário `/autofix` em PRs
- Abertura de PRs
- Execução manual

**Comandos disponíveis**:
```bash
# Em qualquer PR, comentar:
/autofix          # Executa todas as correções
/autofix format   # Apenas formatação
/autofix clippy   # Correções do Clippy
```

**Correções automáticas**:
- 🎨 Formatação de código (Rust + Frontend)
- 🔧 Sugestões do Clippy
- 📦 Atualização de dependências
- 🔒 Correções de segurança
- 🛠️ Correções comuns (whitespace, imports)

### 4. **Security Analysis** (`security.yml`)

**Trigger**: 
- Push em main/develop
- PRs para main
- Agendado semanalmente (segunda-feira)

**Análises realizadas**:
- 🦀 Auditoria de dependências Rust
- 📦 Auditoria de dependências npm
- 🔍 SAST com Semgrep e CodeQL
- 🔑 Detecção de segredos (Trufflehog + Gitleaks)
- 📋 Verificação de conformidade de licenças

**Alertas**:
- Issues automáticas para vulnerabilidades críticas
- Relatórios semanais de segurança

### 5. **Release** (`release.yml`)

**Trigger**: 
- Push de tags `v*`
- Push em main

**Processo**:
1. Execução de todos os testes
2. Build multi-plataforma (Windows, macOS, Linux)
3. Assinatura de código
4. Criação de release no GitHub
5. Upload de artefatos

### 6. **Dependabot** (`dependabot.yml`)

**Agendamento**:
- Rust/npm: Segundas-feiras às 4h
- GitHub Actions: Domingos às 3h

**Configurações**:
- PRs limitados a 5 por ecossistema
- Agrupamento de atualizações minor/patch
- Ignorar majors de dependências críticas

## 🛠️ Comandos e Uso

### Para Desenvolvedores

#### Executar CI localmente:
```bash
# Linting
cargo fmt --check
cargo clippy -- -D warnings
cd frontend && npm run lint

# Testes
cargo test
cd frontend && npm test

# Build
cargo build --release
cd frontend && npm run build
```

#### Comandos em PRs:
```bash
/autofix          # Correções automáticas
/autofix format   # Apenas formatação
/autofix clippy   # Apenas Clippy
```

### Para Maintainers

#### Trigger manual de workflows:
```bash
# Via GitHub CLI
gh workflow run auto-fix.yml -f fix_type=all
gh workflow run security.yml
```

#### Configurar secrets necessários:
```bash
# Secrets obrigatórios:
GITHUB_TOKEN              # Automático
TAURI_PRIVATE_KEY        # Para updates
TAURI_KEY_PASSWORD       # Para updates

# Secrets opcionais (assinatura):
APPLE_CERTIFICATE
APPLE_CERTIFICATE_PASSWORD
WINDOWS_CERTIFICATE
WINDOWS_CERTIFICATE_PASSWORD
```

## 📊 Métricas e Monitoramento

### Dashboard de CI/CD

Acesse em: `https://github.com/[org]/keyai-desktop/actions`

**Métricas importantes**:
- ⏱️ Tempo médio de build: ~15 min
- ✅ Taxa de sucesso do CI: >95%
- 🔒 Vulnerabilidades detectadas/semana
- 📦 PRs de dependências abertas

### Notificações

**Configurar notificações**:
1. Settings → Notifications
2. Ativar "Actions"
3. Escolher eventos importantes

## 🔧 Troubleshooting

### Problema: CI falhando em testes

**Solução**:
```bash
# Executar localmente com logs
RUST_BACKTRACE=1 cargo test -- --nocapture
```

### Problema: Auto-fix não funcionando

**Verificar**:
1. Permissões do usuário (write/admin)
2. Branch protection rules
3. Token do GitHub válido

### Problema: Build falhando em plataforma específica

**Debug**:
```yaml
# Adicionar ao workflow:
- name: Debug info
  run: |
    echo "OS: ${{ runner.os }}"
    echo "Arch: ${{ runner.arch }}"
    rustc --version
    cargo --version
```

## 🚀 Melhores Práticas

### 1. Commits
- Use Conventional Commits
- Mensagens claras e descritivas
- Referência issues quando aplicável

### 2. Pull Requests
- PRs pequenos e focados (<500 linhas)
- Descrição detalhada
- Screenshots para mudanças visuais
- Aguardar CI passar antes de merge

### 3. Branches
- Seguir Gitflow
- Nomear branches descritivamente
- Deletar após merge

### 4. Segurança
- Nunca commitar secrets
- Revisar dependências antes de adicionar
- Manter dependências atualizadas

## 📈 Evolução Futura

### Planejado para v2.0:
- [ ] Deploy automático para stores
- [ ] Testes de performance automatizados
- [ ] Análise de acessibilidade
- [ ] Integração com Sentry
- [ ] Métricas de qualidade de código

### Melhorias contínuas:
- Otimização de tempo de build
- Mais análises de segurança
- Automação de changelog
- Testes visuais para UI

## 🆘 Suporte

**Problemas com CI/CD?**
1. Verificar [Actions](https://github.com/[org]/keyai-desktop/actions)
2. Consultar logs detalhados
3. Abrir issue com label `ci/cd`

**Contatos**:
- DevOps Team: devops@keyai.app
- Security Team: security@keyai.app

---

*Última atualização: Dezembro 2024*