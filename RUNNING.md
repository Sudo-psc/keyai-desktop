# 🚀 Como Executar o KeyAI Desktop

## Pré-requisitos

1. **Rust** (1.78+)
   ```bash
   # Instalar via rustup
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Node.js** (16+) e **npm** (7+)
   ```bash
   # macOS (via Homebrew)
   brew install node
   
   # Ou baixar de https://nodejs.org/
   ```

3. **Dependências do Sistema**
   - **macOS**: Xcode Command Line Tools
   - **Linux**: `libgtk-3-dev`, `libwebkit2gtk-4.0-dev`, `libssl-dev`
   - **Windows**: Visual Studio Build Tools

## Instalação

1. **Clone o repositório**
   ```bash
   git clone https://github.com/seu-usuario/keyai-desktop.git
   cd keyai-desktop
   ```

2. **Instale as dependências**
   ```bash
   # Instalar Tauri CLI
   npm install
   
   # Instalar dependências do frontend
   cd frontend && npm install && cd ..
   ```

## Executando o Aplicativo

### Modo Desenvolvimento (Recomendado para testar)

```bash
# Do diretório raiz do projeto
npm run dev

# Ou diretamente com Tauri
npm run tauri dev
```

Isso irá:
- Compilar o código Rust
- Iniciar o servidor de desenvolvimento Vite
- Abrir a janela do aplicativo

### Comandos Úteis

```bash
# Apenas o frontend (sem Tauri)
npm run frontend:dev

# Compilar para produção
npm run build

# Limpar arquivos de build
npm run clean

# Reinstalar todas as dependências
npm run install:all
```

## Solução de Problemas

### Erro: "tauri command not found"
```bash
# Reinstale o Tauri CLI
npm install --save-dev @tauri-apps/cli
```

### Erro de compilação Rust
```bash
# Atualize as dependências Rust
cargo update

# Limpe o cache
cargo clean
```

### Erro de permissões no macOS
O aplicativo precisa de permissões de Acessibilidade:
1. Abra **Preferências do Sistema** > **Segurança e Privacidade**
2. Vá para a aba **Privacidade**
3. Selecione **Acessibilidade**
4. Adicione o aplicativo KeyAI Desktop

### Frontend não carrega
```bash
# Verifique se as dependências estão instaladas
cd frontend
npm install
npm run dev
```

## Estrutura de Comandos

| Comando | Descrição |
|---------|-----------|
| `npm run dev` | Executa em modo desenvolvimento |
| `npm run build` | Compila para produção |
| `npm run tauri dev` | Executa Tauri em desenvolvimento |
| `npm run tauri build` | Compila aplicativo final |
| `npm run frontend:dev` | Apenas frontend (sem backend) |
| `npm run clean` | Limpa arquivos temporários |

## Desenvolvimento

### Hot Reload
- **Frontend**: Alterações em React são aplicadas automaticamente
- **Backend Rust**: Requer reiniciar o aplicativo (Tauri faz isso automaticamente)

### Logs e Debug
- Logs do Rust aparecem no terminal
- Logs do frontend aparecem no DevTools (F12 na janela do app)

### Variáveis de Ambiente
```bash
# Para debug detalhado
RUST_LOG=debug npm run dev

# Para desabilitar GPU (se houver problemas gráficos)
WEBKIT_DISABLE_COMPOSITING_MODE=1 npm run dev
```

## Build para Produção

```bash
# Gera executáveis para a plataforma atual
npm run build

# Os arquivos estarão em:
# - macOS: src-tauri/target/release/bundle/dmg/
# - Windows: src-tauri/target/release/bundle/msi/
# - Linux: src-tauri/target/release/bundle/appimage/
```

## Próximos Passos

Após executar o aplicativo, você verá:
1. **Tela principal** com barra de busca
2. **Painel de status** mostrando estatísticas
3. **Controles** para iniciar/parar captura

⚠️ **Nota**: A captura de teclas ainda não está implementada. O aplicativo mostrará a interface, mas não capturará eventos reais do teclado. 