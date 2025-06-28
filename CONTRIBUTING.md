# 🤝 Guia de Contribuição - KeyAI Desktop

Obrigado por considerar contribuir com o KeyAI Desktop! Este documento fornece diretrizes para contribuir com o projeto.

## 📋 Índice

- [Código de Conduta](#código-de-conduta)
- [Como Posso Contribuir?](#como-posso-contribuir)
- [Configuração do Ambiente](#configuração-do-ambiente)
- [Fluxo de Desenvolvimento](#fluxo-de-desenvolvimento)
- [Padrões de Código](#padrões-de-código)
- [Testes](#testes)
- [Documentação](#documentação)
- [Processo de Review](#processo-de-review)

## 📜 Código de Conduta

Este projeto segue o [Código de Conduta do Contributor Covenant](https://www.contributor-covenant.org/). Ao participar, você concorda em manter um ambiente respeitoso e inclusivo.

## 🚀 Como Posso Contribuir?

### Reportando Bugs

Antes de reportar um bug:
- ✅ Verifique se já existe uma issue similar
- ✅ Teste na versão mais recente
- ✅ Colete logs relevantes

**Template para Bug Report:**
```markdown
**Descrição do Bug**
Descrição clara e concisa do problema.

**Passos para Reproduzir**
1. Vá para '...'
2. Clique em '...'
3. Veja o erro

**Comportamento Esperado**
O que deveria acontecer.

**Comportamento Atual**
O que realmente acontece.

**Ambiente**
- OS: [Windows 11, macOS 14, Ubuntu 22.04]
- Versão: [1.0.0]
- Arquitetura: [x64, arm64]

**Logs**
```
Anexe logs relevantes aqui
```
```

### Sugerindo Melhorias

**Template para Feature Request:**
```markdown
**Problema/Necessidade**
Qual problema esta feature resolveria?

**Solução Proposta**
Descrição detalhada da solução.

**Alternativas Consideradas**
Outras abordagens que você considerou.

**Contexto Adicional**
Screenshots, mockups, etc.
```

### Contribuindo com Código

1. **Fork** o repositório
2. **Clone** seu fork
3. **Crie** uma branch para sua feature
4. **Desenvolva** seguindo os padrões
5. **Teste** suas mudanças
6. **Commit** com mensagens descritivas
7. **Push** para seu fork
8. **Abra** um Pull Request

## 🛠️ Configuração do Ambiente

### Pré-requisitos

**Rust (1.78+)**
```bash
# Instalar via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Verificar instalação
rustc --version
cargo --version
```

**Node.js (20+)**
```bash
# Via nvm (recomendado)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 20
nvm use 20
```

**Dependências do Sistema**

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
    libxcb-xfixes0-dev \
    pkg-config \
    build-essential \
    curl \
    wget \
    file
```

**macOS:**
```bash
# Xcode Command Line Tools
xcode-select --install

# Homebrew (se necessário)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

**Windows:**
- Visual Studio 2019+ com C++ Build Tools
- WebView2 Runtime
- Git for Windows

### Configuração Inicial

```bash
# Clone o repositório
git clone https://github.com/keyai/keyai-desktop.git
cd keyai-desktop

# Instalar dependências do frontend
cd frontend
npm install
cd ..

# Verificar configuração
cargo check

# Executar testes
cargo test

# Executar em modo desenvolvimento
npm run tauri dev
```

## 🔄 Fluxo de Desenvolvimento

### Estrutura de Branches

- `main`: Branch principal (protegida)
- `develop`: Branch de desenvolvimento
- `feature/nome-da-feature`: Features
- `bugfix/nome-do-bug`: Correções
- `hotfix/nome-do-hotfix`: Correções urgentes

### Convenções de Commit

Usamos [Conventional Commits](https://www.conventionalcommits.org/):

```
<tipo>[escopo opcional]: <descrição>

[corpo opcional]

[rodapé opcional]
```

**Tipos:**
- `feat`: Nova funcionalidade
- `fix`: Correção de bug
- `docs`: Mudanças na documentação
- `style`: Formatação, ponto e vírgula, etc.
- `refactor`: Refatoração de código
- `test`: Adição ou correção de testes
- `chore`: Tarefas de manutenção

**Exemplos:**
```bash
feat(search): adicionar busca semântica com embeddings

fix(agent): corrigir vazamento de memória na captura de teclas

docs(readme): atualizar instruções de instalação

test(masker): adicionar testes para mascaramento de CPF
```

## 📝 Padrões de Código

### Rust

**Formatação:**
```bash
# Formatar código
cargo fmt

# Verificar estilo
cargo clippy
```

**Convenções:**
- Use `snake_case` para funções e variáveis
- Use `PascalCase` para structs e enums
- Use `SCREAMING_SNAKE_CASE` para constantes
- Documente funções públicas com `///`
- Use `Result<T, E>` para tratamento de erros
- Prefira `&str` sobre `String` quando possível

**Exemplo:**
```rust
/// Mascara informações pessoais em texto.
/// 
/// # Argumentos
/// 
/// * `text` - O texto a ser mascarado
/// * `patterns` - Padrões de PII a serem aplicados
/// 
/// # Retorna
/// 
/// Texto com PII mascarado
/// 
/// # Exemplos
/// 
/// ```
/// let masked = mask_pii("João Silva - CPF: 123.456.789-01", &pii_patterns);
/// assert_eq!(masked, "João Silva - CPF: ***.***.***-01");
/// ```
pub fn mask_pii(text: &str, patterns: &[PiiPattern]) -> Result<String, MaskError> {
    // implementação
}
```

### TypeScript/React

**Configuração ESLint:**
```bash
# Verificar código
npm run lint

# Corrigir automaticamente
npm run lint:fix
```

**Convenções:**
- Use `camelCase` para variáveis e funções
- Use `PascalCase` para componentes
- Use `UPPER_CASE` para constantes
- Prefira componentes funcionais
- Use hooks customizados para lógica reutilizável
- Tipagem forte (evite `any`)

**Exemplo:**
```typescript
interface SearchResult {
  id: string;
  content: string;
  timestamp: Date;
  relevance: number;
}

interface SearchComponentProps {
  onSearch: (query: string) => void;
  results: SearchResult[];
  isLoading: boolean;
}

export const SearchComponent: React.FC<SearchComponentProps> = ({
  onSearch,
  results,
  isLoading
}) => {
  const [query, setQuery] = useState('');
  
  const handleSubmit = useCallback((e: React.FormEvent) => {
    e.preventDefault();
    onSearch(query);
  }, [query, onSearch]);

  return (
    <form onSubmit={handleSubmit}>
      {/* implementação */}
    </form>
  );
};
```

## 🧪 Testes

### Executando Testes

```bash
# Testes unitários Rust
cargo test

# Testes com output detalhado
cargo test -- --nocapture

# Testes específicos
cargo test masker::tests

# Benchmarks
cargo bench

# Testes do frontend
cd frontend
npm test
```

### Escrevendo Testes

**Testes Unitários (Rust):**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_cpf() {
        let input = "CPF: 123.456.789-01";
        let expected = "CPF: ***.***.***-01";
        
        let result = mask_cpf(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_mask_cpf_invalid() {
        let input = "CPF inválido";
        let result = mask_cpf(input);
        
        assert!(result.is_err());
    }
}
```

**Testes de Integração:**
```rust
// tests/integration_test.rs
use keyai_desktop::agent::Agent;
use keyai_desktop::db::Database;

#[tokio::test]
async fn test_agent_to_db_flow() {
    let db = Database::new_temp().await.unwrap();
    let agent = Agent::new().unwrap();
    
    // Simular eventos de teclado
    agent.simulate_keypress("Hello World").await;
    
    // Verificar se foi salvo no banco
    let results = db.search("Hello").await.unwrap();
    assert!(!results.is_empty());
}
```

### Cobertura de Testes

```bash
# Instalar tarpaulin
cargo install cargo-tarpaulin

# Gerar relatório de cobertura
cargo tarpaulin --out Html

# Ver relatório
open tarpaulin-report.html
```

## 📚 Documentação

### Documentação de Código

- **Rust**: Use `///` para documentação pública
- **TypeScript**: Use JSDoc para funções complexas
- **README**: Mantenha atualizado com mudanças

### Documentação de API

```rust
/// Comando Tauri para buscar texto no banco de dados.
/// 
/// # Argumentos
/// 
/// * `query` - Termo de busca
/// * `limit` - Número máximo de resultados (padrão: 50)
/// * `offset` - Offset para paginação (padrão: 0)
/// 
/// # Retorna
/// 
/// Lista de resultados ordenados por relevância
/// 
/// # Erros
/// 
/// Retorna erro se:
/// - Query estiver vazia
/// - Banco de dados estiver inacessível
/// - Limite exceder 1000
#[tauri::command]
pub async fn search_text(
    query: String,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<SearchResult>, String> {
    // implementação
}
```

## 🔍 Processo de Review

### Checklist para Pull Requests

**Antes de Submeter:**
- [ ] Código segue os padrões estabelecidos
- [ ] Testes passam localmente
- [ ] Documentação foi atualizada
- [ ] Commits seguem convenções
- [ ] Branch está atualizada com `main`

**Template de PR:**
```markdown
## Descrição
Breve descrição das mudanças.

## Tipo de Mudança
- [ ] Bug fix
- [ ] Nova funcionalidade
- [ ] Breaking change
- [ ] Documentação

## Como Testar
1. Passo 1
2. Passo 2
3. Passo 3

## Checklist
- [ ] Testes passam
- [ ] Código formatado
- [ ] Documentação atualizada
- [ ] Self-review realizado

## Screenshots (se aplicável)
```

### Processo de Review

1. **Automated Checks**: CI/CD executa testes e linting
2. **Code Review**: Pelo menos 1 aprovação necessária
3. **Testing**: Revisor testa funcionalidade
4. **Merge**: Squash and merge para `main`

### Critérios de Aprovação

- ✅ Funcionalidade funciona conforme especificado
- ✅ Código é legível e bem documentado
- ✅ Testes cobrem cenários importantes
- ✅ Performance não foi degradada
- ✅ Segurança foi considerada

## 🎯 Dicas para Contribuidores

### Primeiras Contribuições

**Issues para Iniciantes:**
- Procure por labels `good first issue`
- Comece com documentação ou testes
- Faça perguntas no Discord ou GitHub

### Desenvolvimento Eficiente

```bash
# Desenvolvimento com hot-reload
npm run tauri dev

# Executar apenas o backend
cargo run

# Watch mode para testes
cargo watch -x test

# Formatação automática
cargo watch -x fmt
```

### Debugging

**Rust:**
```bash
# Debug com logs
RUST_LOG=debug cargo run

# Debug com GDB
cargo build
gdb target/debug/keyai-desktop
```

**Frontend:**
```bash
# Debug no browser
npm run tauri dev
# Abrir DevTools: F12
```

### Performance

```bash
# Profile com perf (Linux)
cargo build --release
sudo perf record -g ./target/release/keyai-desktop
sudo perf report

# Benchmark específico
cargo bench -- search_benchmark
```

## 📞 Suporte

- **GitHub Issues**: Para bugs e features
- **GitHub Discussions**: Para perguntas gerais
- **Discord**: Chat em tempo real (link no README)
- **Email**: keyai-dev@exemplo.com

## 🙏 Reconhecimento

Todos os contribuidores são listados no arquivo `CONTRIBUTORS.md` e nos releases.

---

**Obrigado por contribuir com o KeyAI Desktop! 🚀** 