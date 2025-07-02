# Script para executar CI localmente no Windows
# Simula os mesmos checks que rodam no GitHub Actions

$ErrorActionPreference = "Stop"

Write-Host "🚀 Iniciando CI Local para KeyAI Desktop" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# Função para printar status
function Print-Status {
    param(
        [bool]$Success,
        [string]$Message
    )
    
    if ($Success) {
        Write-Host "✓ $Message" -ForegroundColor Green
    } else {
        Write-Host "✗ $Message" -ForegroundColor Red
        exit 1
    }
}

# 1. Verificar ferramentas necessárias
Write-Host "`n📋 Verificando ferramentas..." -ForegroundColor Yellow

$cargoExists = Get-Command cargo -ErrorAction SilentlyContinue
Print-Status ($null -ne $cargoExists) "Rust/Cargo instalado"

$nodeExists = Get-Command node -ErrorAction SilentlyContinue
Print-Status ($null -ne $nodeExists) "Node.js instalado"

$npmExists = Get-Command npm -ErrorAction SilentlyContinue
Print-Status ($null -ne $npmExists) "npm instalado"

# 2. Rust Checks
Write-Host "`n🦀 Verificando código Rust..." -ForegroundColor Yellow

Write-Host "  → Formatação..."
$formatResult = & cargo fmt -- --check 2>&1
Print-Status ($LASTEXITCODE -eq 0) "Formatação Rust OK"

Write-Host "  → Clippy..."
$clippyResult = & cargo clippy --all-targets --all-features -- -D warnings 2>&1
Print-Status ($LASTEXITCODE -eq 0) "Clippy sem warnings"

Write-Host "  → Testes..."
$testResult = & cargo test --verbose 2>&1
Print-Status ($LASTEXITCODE -eq 0) "Testes Rust passando"

Write-Host "  → Documentação..."
$docResult = & cargo doc --no-deps --all-features 2>&1
Print-Status ($LASTEXITCODE -eq 0) "Documentação Rust OK"

# 3. Frontend Checks
Write-Host "`n⚛️  Verificando código Frontend..." -ForegroundColor Yellow

Push-Location frontend

Write-Host "  → Instalando dependências..."
$npmCiResult = & npm ci 2>&1
Print-Status ($LASTEXITCODE -eq 0) "Dependências instaladas"

Write-Host "  → Linting..."
$lintResult = & npm run lint 2>&1
Print-Status ($LASTEXITCODE -eq 0) "ESLint sem erros"

Write-Host "  → Type checking..."
$typeResult = & npm run typecheck 2>&1
Print-Status ($LASTEXITCODE -eq 0) "TypeScript sem erros"

Write-Host "  → Formatação..."
$formatCheckResult = & npm run format:check 2>&1
Print-Status ($LASTEXITCODE -eq 0) "Formatação Frontend OK"

Write-Host "  → Testes..."
$testFrontendResult = & npm test -- --run 2>&1
Print-Status ($LASTEXITCODE -eq 0) "Testes Frontend passando"

Write-Host "  → Build..."
$buildResult = & npm run build 2>&1
Print-Status ($LASTEXITCODE -eq 0) "Build Frontend OK"

Pop-Location

# 4. Build completo
Write-Host "`n🔨 Build completo..." -ForegroundColor Yellow
$releaseBuild = & cargo build --release 2>&1
Print-Status ($LASTEXITCODE -eq 0) "Build Release OK"

# 5. Análise de segurança (opcional)
$cargoAuditExists = Get-Command cargo-audit -ErrorAction SilentlyContinue
if ($null -ne $cargoAuditExists) {
    Write-Host "`n🔒 Análise de segurança..." -ForegroundColor Yellow
    $auditResult = & cargo audit 2>&1
    Print-Status ($LASTEXITCODE -eq 0) "Sem vulnerabilidades conhecidas"
} else {
    Write-Host "`n⚠️  cargo-audit não instalado. Pulando análise de segurança." -ForegroundColor DarkYellow
    Write-Host "   Instale com: cargo install cargo-audit" -ForegroundColor DarkYellow
}

# 6. Resumo
Write-Host "`n========================================" -ForegroundColor Green
Write-Host "✅ CI Local concluído com sucesso!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "Próximos passos:"
Write-Host "  1. Commit suas mudanças"
Write-Host "  2. Push para o GitHub"
Write-Host "  3. Abra um Pull Request"
Write-Host ""
Write-Host "Dica: Execute '/autofix' no PR para correções automáticas"