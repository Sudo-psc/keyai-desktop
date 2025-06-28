Plano de Desenvolvimento de Software — KeyAI Desktop (Rust)


1. Objetivo do Documento

Servir como o plano técnico e operacional para o desenvolvimento do KeyAI Desktop v1.0.
Alinhar as equipes de Engenharia, Produto e QA sobre escopo, arquitetura e cronograma.

2. Escopo do Projeto


Funcionalidades Incluídas (v1.0)

Agente de registro de teclas local para Windows, macOS (X11) e Linux (X11).
Máscara automática de PII (CPF, e-mail, telefone) em tempo real, no dispositivo.
Banco de dados local único e criptografado com SQLCipher.
Busca híbrida: full-text (FTS5) e semântica (vetorial com sqlite-vec).
GUI minimalista em Tauri para busca e configurações básicas.
Instaladores nativos (.msi, .dmg, .AppImage, .deb).

Funcionalidades Excluídas (Fora do Escopo v1.0)

Sincronização ou backup em nuvem.
Versões para plataformas móveis (iOS/Android).
Captura de texto via OCR.
Customização de padrões de PII pelo usuário.
Funcionalidades de colaboração ou multiusuário.
Integração com extensões de navegador.

3. Arquitetura de Alto Nível

A arquitetura é desacoplada para garantir performance e resiliência. A captura de teclas, sendo a operação mais crítica, é isolada de processos mais lentos como escrita em disco ou processamento de PII. Isso previne a perda de eventos de teclado caso o banco de dados esteja bloqueado ou uma regra de PII seja computacionalmente intensiva.

Snippet de código


@startuml
!theme plain
skinparam monochrome true
skinparam packageStyle rectangle
skinparam shadowing false

package "KeyAI Desktop Application" {
  <<WebView>>
 
}

package "Processo em Background" {
  <<Thread Rust>>
  [Masker (Filtro PII)] <<Thread Rust>>
  <<Thread Rust>>
}

database "Banco de Dados Criptografado" as DB {
 
 
  [Índice Vetorial (sqlite-vec)]
}

[Input do Usuário] -->
 --> [Masker (Filtro PII)] : Eventos brutos (canal MPSC)
[Masker (Filtro PII)] --> : Dados mascarados
 --> DB : Escritas em lote
 <--> : Comandos Tauri
 <--> DB : Queries de busca

@enduml


Agente (Rust Core): Processo em background persistente.
Usa a crate rdev para captura de eventos de teclado globais.
Opera em uma thread dedicada de alta prioridade para não perder eventos.
Envia eventos brutos para o Masker via um canal em memória (MPSC).
Masker (Rust Core): Módulo que consome eventos do Agente.
Aplica regras de regex para identificar e ofuscar PII.
Agrupa dados processados para escrita eficiente no banco.
DB (SQLite Engine): Camada de persistência.
Arquivo único keyai.db criptografado via rusqlite com a feature bundled-sqlcipher.
Contém tabelas para eventos e tabelas virtuais para FTS5 e sqlite-vec.
GUI (Tauri Frontend): Interface do usuário.
Construída com tecnologias web (React + TypeScript).
Comunica-se com o backend Rust via comandos Tauri para executar buscas e alterar configurações.

4. Pilha Tecnológica (Tech-Stack)

A seleção de tecnologias prioriza a performance, segurança e a simplificação do processo de build multiplataforma. A opção por bundled-sqlcipher elimina a dependência de bibliotecas OpenSSL no ambiente de CI/CD, um ponto comum de falhas. A adoção de sqlite-vec em vez de sqlite-vss alinha o projeto com a solução mais moderna e recomendada pelo autor original para busca vetorial.

Categoria
Tecnologia
Versão Mínima
Justificativa
Linguagem
Rust
1.78+
Performance, segurança de memória e ecossistema robusto.
Framework App
Tauri
1.7+
Binários pequenos, performance e segurança por padrão.
UI Frontend
React + TypeScript
18.x + 5.x
Ecossistema maduro e tipagem forte para a UI.
Input Capture
rdev
0.5.3+
Suporte multiplataforma (Win/macOS/X11) para eventos globais.
Banco de Dados
rusqlite
0.31+
Wrapper idiomático e performático para SQLite em Rust.
Criptografia DB
rusqlite feature
bundled-sqlcipher
Empacota SQLCipher, simplificando a compilação cruzada.
Busca Full-Text
SQLite Extension
FTS5
Nativo do SQLite, rápido e eficiente para busca de texto.
Busca Vetorial
SQLite Extension
sqlite-vec
Solução moderna para busca semântica em SQLite, substitui sqlite-vss.
Embeddings
rust-bert
0.21+
Geração de embeddings localmente, mantendo a privacidade.
CI/CD
GitHub Actions
- Integração nativa com tauri-action para builds multiplataforma.
Build Action
tauri-action
v0
Ferramenta oficial para build e release de apps Tauri.
Testes
criterion
0.5+
Benchmarking de performance do agente e da busca.


5. Plano de Iterações / Sprints

O plano de iterações é desenhado para mitigar os maiores riscos no início do projeto. A validação da captura de teclas multiplataforma com rdev é a prioridade máxima (Sprint 1), pois sua inviabilidade comprometeria todo o projeto. Apenas após a validação do core técnico, o trabalho na interface do usuário é iniciado.

Sprint
Objetivo-Chave
Entregáveis Principais
Sprint 0
Configuração do Projeto
Repositório, CI/CD básico, setup Tauri, WBS detalhado.
Sprint 1
PoC do Agente de Captura
Agente funcional em Windows, macOS (X11) e Linux (X11).
Sprint 2
Persistência e Criptografia
Schema do DB, integração com rusqlite + SQLCipher, PII Masker v1.
Sprint 3
Implementação da Busca
Integração de FTS5 e sqlite-vec, API de busca no backend.
Sprint 4
UI/UX Básico e Integração
Tela de busca, visualização de resultados, comandos Tauri.
Sprint 5
Refinamento e Testes E2E
Benchmarks de performance, testes de integração, ciclo de feedback.
Sprint 6
Empacotamento e Assinatura
Configuração de tauri-action para release, assinatura de código.
Sprint 7
Alpha Interna & Correções
Distribuição interna, coleta de feedback, correção de bugs críticos.
Sprint 8
Preparação para Beta
Finalização de features da v1.0, documentação do usuário.


6. WBS (Work Breakdown Structure)

Epic 1: Core Engine & Data Capture
Implementar listener rdev para Windows.
Implementar listener rdev para macOS (incluindo permissão de acessibilidade).
Implementar listener rdev para Linux (X11).
Criar canal assíncrono entre Agente e Masker.
Desenvolver módulo de PII masking com padrões regex iniciais.
Epic 2: Persistência e Busca
Definir schema final do SQLite.
Integrar rusqlite com a feature bundled-sqlcipher.
Implementar serviço de escrita em lote no DB.
Configurar tabela virtual FTS5.
Integrar sqlite-vec e rust-bert para busca vetorial.
Implementar lógica de busca híbrida (ex: Reciprocal Rank Fusion).
Dependência Crítica: Epic 1 deve estar funcional.
Epic 3: Interface do Usuário (GUI)
Setup do projeto Tauri com React+TS.
Implementar a janela principal de busca.
Criar painel de configurações.
Implementar comandos Tauri para search, pause, resume.
Exibir resultados de busca dinamicamente.
Dependência Crítica: API de busca do Epic 2 deve estar disponível.
Epic 4: Engenharia de Release (CI/CD)
Criar workflow GitHub Actions (release.yml).
Implementar matriz de build para Win, macOS, Linux.
Configurar segredos para assinatura de código.
Adicionar passo de assinatura de código para Windows (signtool).
Adicionar passo de assinatura e notarização para macOS.
Configurar tauri-action para publicar artefatos de release.
Epic 5: Qualidade e Testes
Escrever testes unitários para o PII Masker.
Escrever testes de integração para a camada de DB.
Desenvolver suíte de testes E2E usando rdev::simulate para input.
Criar benchmarks de performance para latência de busca e uso de CPU.

7. Cronograma e Recursos (FTE)


Linha do Tempo (Gantt Simplificado)

Mês 1: Sprints 0-1. Foco em setup e PoC do Agente de captura.
Mês 2: Sprints 2-3. Foco em backend, DB e lógica de busca.
Mês 3: Sprints 4-5. Foco em UI, integração e testes iniciais.
Mês 4: Sprints 6-7. Foco em engenharia de release e Alpha interna.
Mês 5: Sprint 8. Finalização para Beta e correções de bugs.
Mês 6: Fase de Beta (Fechado/Aberto) e preparação para o lançamento (GA).

Equipe e Alocação (FTE - Full-Time Equivalent)


Papel
FTE
Justificativa
Rust Developer (Sênior)
2.0
Responsáveis pelo core (Agente, Masker, DB). Dois para paralelismo.
Frontend Developer (Pleno/Sênior)
1.0
Focado na UI com Tauri/React, com experiência em integração Rust.
QA Engineer
0.5
Meio período para automação de testes E2E e testes manuais.


8. CI/CD Detalhado


Workflow (GitHub Actions)

Arquivo: .github/workflows/release.yml.
Gatilho: push na branch main ou workflow_dispatch manual.
Job: Um único job build-and-release com uma matriz de estratégia.
strategy.matrix.os: [windows-latest, macos-latest, ubuntu-latest].
Passos por OS:
Checkout do código (actions/checkout@v4).
Setup do toolchain Rust (dtolnay/rust-toolchain@stable).
Setup do Node.js (actions/setup-node@v4).
Cache de dependências Rust e Node (swatinem/rust-cache@v2).
Setup de Assinatura de Código (Condicional):
Para windows-latest: Decodificar segredo com certificado PFX e importar no keystore.
Para macos-latest: Usar segredos APPLE_CERTIFICATE e APPLE_CERTIFICATE_PASSWORD que o Tauri Action consome para criar um keychain temporário e realizar a notarização.
Instalar dependências do frontend (npm install).
Build do frontend (npm run build).
Executar tauri-apps/tauri-action@v0:
Executa tauri build.
Cria um Release no GitHub e faz o upload dos artefatos (.msi, .dmg, .AppImage) e do manifesto de atualização (latest.json).

Segredos Necessários (GitHub Secrets)

WINDOWS_CERTIFICATE: Certificado .pfx para Windows, codificado em Base64.
WINDOWS_CERTIFICATE_PASSWORD: Senha do certificado .pfx.
APPLE_CERTIFICATE: Certificado de desenvolvedor .p12 da Apple, em Base64.
APPLE_CERTIFICATE_PASSWORD: Senha do certificado .p12.
APPLE_ID: E-mail da conta de desenvolvedor Apple (para notarização).
APPLE_PASSWORD: Senha de aplicativo Apple (para notarização).
APPLE_TEAM_ID: ID do time de desenvolvedor Apple (para notarização).

9. Qualidade e Testes


Estratégia de Testes

Unitários (cargo test): Foco em lógica pura e isolada (ex: funções do PII Masker).
Integração: Testar a interação entre componentes (ex: Agent -> Masker -> DB).
End-to-End (E2E): Simular entrada do usuário via rdev::simulate e validar o estado do DB e os resultados da busca.
Benchmarks (criterion.rs): Medir latência da busca e uso de recursos do Agente.

Metas de Qualidade (v1.0)

Cobertura de Código (Core Rust): >80%.
Latência de Busca (p95): ≤150 ms (base com 1M de palavras).
Uso de CPU (Agente Ocioso): <3% em média.
Bugs Críticos no Lançamento (GA): Zero.

10. Gestão de Riscos

A gestão de riscos foca nas ameaças existenciais do projeto: viabilidade técnica em ambientes hostis (Wayland, antivírus) e performance. A decisão de não suportar Wayland na v1.0 é estratégica, transformando um risco técnico incontrolável em uma limitação de escopo gerenciável e comunicável aos usuários.

Risco
Probabilidade
Impacto
Ações de Mitigação
Incompatibilidade com Wayland
Alta
Alto
Focar em X11 para v1.0 e documentar Wayland como não suportado. Alocar P&D para monitorar libei para futuras versões.
Falsos Positivos de Antivírus
Alta
Médio
Usar assinatura de código EV (Extended Validation). Submeter binários para análise e whitelisting pelos principais fornecedores de AV.
"Sherlocking" por SOs
Baixa
Alto
Focar em diferenciais (busca híbrida, privacidade total). Construir uma base de usuários leal e iterar rapidamente.
Bugs de Plataforma em rdev
Média
Alto
Realizar PoC no Sprint 1 para validar rdev em todas as plataformas alvo. Manter um fork privado se for necessário para correções.
Performance da Busca Vetorial
Média
Médio
Benchmarking contínuo com criterion.rs a cada PR. Otimizar geração de embeddings e parâmetros do índice vetorial.


11. Plano de Lançamento & Suporte


Fases de Lançamento

Alpha (Mês 4): Distribuição interna para a equipe.
Beta Fechado (Mês 5): Lançamento para usuários técnicos selecionados.
Beta Aberto (Mês 6): Lançamento público via site para testes em larga escala.
General Availability (GA): Lançamento nas lojas de aplicativos (MS Store, etc.).

Canais de Feedback

GitHub Issues para bugs e requisição de funcionalidades.
Servidor Discord dedicado para a comunidade de usuários.

Política de Atualização

Utilizar o updater nativo do Tauri, que verifica o latest.json no GitHub Releases.
Atualizações automáticas e silenciosas para patches de segurança.
Notificação no app para atualizações maiores de funcionalidades.

12. Orçamento Detalhado

O orçamento é baseado em salários para o mercado de Toronto, ON, Canadá, e custos operacionais anuais para distribuição de software. A inclusão de um custo opcional para a API da OpenAI serve como uma ferramenta de planejamento estratégico para futuras versões que possam oferecer funcionalidades baseadas em nuvem.

CAPEX (Custo de Desenvolvimento - 6 meses)

Premissas: Salários baseados no 75º percentil para Toronto, ON.

Item
Cálculo
Custo (6 meses, CAD)
2.0 FTE Rust Dev
2×$121,500×(6/12)
$121,500
1.0 FTE Frontend Dev
1×$118,000×(6/12)
$59,000
0.5 FTE QA Engineer
0.5×$95,434×(6/12)
$23,859
Total CAPEX


$204,359


OPEX (Custo Operacional - Anual)


Item
Custo Anual (CAD)
Fonte
EV Code Signing Certificate
~$864


Apple Developer Program
~$135 ($99 USD)


Total OPEX
~$999




Custo Opcional: API OpenAI (Estimativa para v1.x)

Premissas: 10.000 usuários, 10 buscas/dia, 500 tokens input/100 tokens output por busca.

Item
Cálculo Mensal
Custo Mensal (USD)
Fonte
Input (GPT-4o)
1.5B tokens×($2.50/1M tokens)
$3,750


Output (GPT-4o)
300M tokens×($10.00/1M tokens)
$3,000


Custo Total Mensal


~$6,750