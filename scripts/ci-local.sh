#!/bin/bash

# Script para executar CI localmente
# Simula os mesmos checks que rodam no GitHub Actions

set -e

echo "🚀 Iniciando CI Local para KeyAI Desktop"
echo "========================================"

# Cores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Função para printar status
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✓${NC} $2"
    else
        echo -e "${RED}✗${NC} $2"
        exit 1
    fi
}

# 1. Verificar ferramentas necessárias
echo -e "\n📋 Verificando ferramentas..."
command -v cargo >/dev/null 2>&1
print_status $? "Rust/Cargo instalado"

command -v node >/dev/null 2>&1
print_status $? "Node.js instalado"

command -v npm >/dev/null 2>&1
print_status $? "npm instalado"

# 2. Rust Checks
echo -e "\n🦀 Verificando código Rust..."

echo "  → Formatação..."
cargo fmt -- --check
print_status $? "Formatação Rust OK"

echo "  → Clippy..."
cargo clippy --all-targets --all-features -- -D warnings
print_status $? "Clippy sem warnings"

echo "  → Testes..."
cargo test --verbose
print_status $? "Testes Rust passando"

echo "  → Documentação..."
cargo doc --no-deps --all-features
print_status $? "Documentação Rust OK"

# 3. Frontend Checks
echo -e "\n⚛️  Verificando código Frontend..."

cd frontend

echo "  → Instalando dependências..."
npm ci
print_status $? "Dependências instaladas"

echo "  → Linting..."
npm run lint
print_status $? "ESLint sem erros"

echo "  → Type checking..."
npm run typecheck
print_status $? "TypeScript sem erros"

echo "  → Formatação..."
npm run format:check
print_status $? "Formatação Frontend OK"

echo "  → Testes..."
npm test -- --run
print_status $? "Testes Frontend passando"

echo "  → Build..."
npm run build
print_status $? "Build Frontend OK"

cd ..

# 4. Build completo
echo -e "\n🔨 Build completo..."
cargo build --release
print_status $? "Build Release OK"

# 5. Análise de segurança (opcional)
if command -v cargo-audit >/dev/null 2>&1; then
    echo -e "\n🔒 Análise de segurança..."
    cargo audit
    print_status $? "Sem vulnerabilidades conhecidas"
else
    echo -e "\n${YELLOW}⚠️  cargo-audit não instalado. Pulando análise de segurança.${NC}"
    echo "   Instale com: cargo install cargo-audit"
fi

# 6. Resumo
echo -e "\n========================================"
echo -e "${GREEN}✅ CI Local concluído com sucesso!${NC}"
echo -e "========================================"
echo ""
echo "Próximos passos:"
echo "  1. Commit suas mudanças"
echo "  2. Push para o GitHub"
echo "  3. Abra um Pull Request"
echo ""
echo "Dica: Execute '/autofix' no PR para correções automáticas"