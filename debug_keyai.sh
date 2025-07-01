#!/bin/bash
# Script de Debug para KeyAI Desktop no macOS
# Executa com máximo logging e captura diagnósticos

set -e

# Cores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Timestamp para logs únicos
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
LOG_DIR="$HOME/keyai_debug_$TIMESTAMP"

echo -e "${BLUE}=== KeyAI Desktop Debug Script ===${NC}"
echo -e "${YELLOW}Timestamp: $TIMESTAMP${NC}"
echo -e "${YELLOW}Log Directory: $LOG_DIR${NC}"

# Criar diretório de logs
mkdir -p "$LOG_DIR"

# 1. Verificar permissões TCC
echo -e "\n${BLUE}[1/7] Verificando permissões TCC...${NC}"
sqlite3 ~/Library/Application\ Support/com.apple.TCC/TCC.db \
  "SELECT service, client, auth_value FROM access WHERE service='kTCCServiceAccessibility';" \
  > "$LOG_DIR/tcc_permissions.txt" 2>&1 || echo "Erro ao acessar TCC.db (normal se não for root)"

# 2. Coletar informações do sistema
echo -e "${BLUE}[2/7] Coletando informações do sistema...${NC}"
{
    echo "=== System Info ==="
    sw_vers
    echo -e "\n=== Process Info ==="
    ps aux | grep -E "keyai|Terminal" | grep -v grep
    echo -e "\n=== Environment ==="
    env | grep -E "TERM|RUST" | sort
} > "$LOG_DIR/system_info.txt"

# 3. Limpar e compilar em debug
echo -e "${BLUE}[3/7] Compilando em modo debug...${NC}"
cd "$(dirname "$0")"
cargo clean
RUSTFLAGS="-C debuginfo=2 -C strip=none" cargo build 2>&1 | tee "$LOG_DIR/build.log"

# 4. Configurar ambiente de debug
echo -e "${BLUE}[4/7] Configurando ambiente...${NC}"
export RUST_BACKTRACE=full
export RUST_LOG=keyai=trace,rdev=trace,tauri=debug
export RUST_LIB_BACKTRACE=1

# 5. Iniciar captura de logs do sistema
echo -e "${BLUE}[5/7] Iniciando captura de logs do sistema...${NC}"
# Log stream em background
log stream --predicate 'process == "keyai-desktop" OR eventMessage CONTAINS "keyai" OR subsystem == "com.apple.TCC"' \
    --style compact --info --debug > "$LOG_DIR/system_stream.log" 2>&1 &
STREAM_PID=$!
echo "Log stream PID: $STREAM_PID"

# 6. Executar com LLDB
echo -e "${BLUE}[6/7] Executando KeyAI com LLDB...${NC}"
cat > "$LOG_DIR/lldb_commands.txt" << EOF
settings set target.process.stop-on-exec true
breakpoint set --name panic_unwind
breakpoint set --name rust_panic
breakpoint set --file agent/mod.rs --line 495
run
EOF

echo -e "${YELLOW}Instruções LLDB:${NC}"
echo "1. Quando o LLDB iniciar, digite: run"
echo "2. Na interface, clique em 'Ativar Captura'"
echo "3. Digite algumas teclas de teste"
echo "4. Se houver crash:"
echo "   - Digite: bt all"
echo "   - Digite: thread info"
echo "   - Digite: process save-core $LOG_DIR/keyai.core"
echo "5. Digite: quit"

# Executar com timeout para evitar travamento
timeout 300 lldb -s "$LOG_DIR/lldb_commands.txt" target/debug/keyai-desktop 2>&1 | tee "$LOG_DIR/lldb_session.log" || {
    echo -e "${RED}LLDB terminou ou timeout alcançado${NC}"
}

# 7. Coletar logs finais
echo -e "${BLUE}[7/7] Coletando logs finais...${NC}"

# Parar log stream
kill $STREAM_PID 2>/dev/null || true

# Coletar logs do Console.app
echo "Exportando logs do Console.app..."
log show --predicate 'process == "keyai-desktop"' --last 10m --style compact > "$LOG_DIR/console_export.log" 2>&1

# Coletar crash reports se existirem
if ls ~/Library/Logs/DiagnosticReports/keyai-desktop*.crash 2>/dev/null; then
    echo "Copiando crash reports..."
    cp ~/Library/Logs/DiagnosticReports/keyai-desktop*.crash "$LOG_DIR/"
fi

# Criar resumo
echo -e "\n${GREEN}=== Resumo de Debug ===${NC}" | tee "$LOG_DIR/SUMMARY.txt"
echo "Logs salvos em: $LOG_DIR" | tee -a "$LOG_DIR/SUMMARY.txt"
echo "" | tee -a "$LOG_DIR/SUMMARY.txt"
echo "Arquivos gerados:" | tee -a "$LOG_DIR/SUMMARY.txt"
ls -la "$LOG_DIR/" | tee -a "$LOG_DIR/SUMMARY.txt"

# Verificar por erros óbvios
echo -e "\n${YELLOW}=== Análise Rápida ===${NC}" | tee -a "$LOG_DIR/SUMMARY.txt"
if grep -q "permission\|accessibility\|TCC" "$LOG_DIR"/*.log 2>/dev/null; then
    echo -e "${RED}⚠️  Possíveis problemas de permissão detectados${NC}" | tee -a "$LOG_DIR/SUMMARY.txt"
fi

if grep -q "panic\|PANIC" "$LOG_DIR"/*.log 2>/dev/null; then
    echo -e "${RED}⚠️  Panic detectado nos logs${NC}" | tee -a "$LOG_DIR/SUMMARY.txt"
fi

echo -e "\n${GREEN}Debug concluído!${NC}"
echo -e "${YELLOW}Para analisar: cd $LOG_DIR${NC}" 