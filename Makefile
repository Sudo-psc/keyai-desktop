# ==============================================================================
# Makefile para KeyAI Desktop - Arquitetura de Microsserviços
# ==============================================================================

# Definições de Variáveis
# ------------------------------------------------------------------------------
SHELL := /bin/bash
DOCKER_COMPOSE_DEV = docker-compose -f docker/docker-compose.dev.yml
DOCKER_COMPOSE_PROD = docker-compose -f docker/docker-compose.prod.yml

SERVICES = capture-service masking-service storage-service search-service analytics-service
LIBS = common-lib proto-lib

# Ajuda
# ------------------------------------------------------------------------------
.PHONY: help
help:
	@echo "Usage: make <target>"
	@echo ""
	@echo "Targets:"
	@echo "  setup         - Instala todas as dependências do projeto"
	@echo "  dev           - Inicia o ambiente de desenvolvimento com hot-reload"
	@echo "  build         - Compila todos os microsserviços em modo release"
	@echo "  build-local   - Compila o binário Tauri para a plataforma local"
	@echo "  test          - Roda todos os testes (unitários e de integração)"
	@echo "  lint          - Roda linters (clippy e fmt)"
	@echo "  clean         - Limpa artefatos de build"
	@echo "  proto-gen     - Gera código a partir dos arquivos .proto"
	@echo "  docker-up     - Inicia todos os serviços com Docker Compose"
	@echo "  docker-down   - Para e remove todos os containers Docker"
	@echo "  docker-build  - Builda as imagens Docker para todos os serviços"
	@echo "  docker-logs   - Mostra os logs de todos os serviços"
	@echo "  deploy-k8s    - Aplica as configurações do Kubernetes"

# Setup Inicial
# ------------------------------------------------------------------------------
.PHONY: setup
setup:
	@echo "📦 Instalando dependências..."
	@if ! command -v rustc &> /dev/null; then \
		echo "Rust não encontrado. Por favor, instale via https://rustup.rs/"; \
		exit 1; \
	fi
	@if ! command -v node &> /dev/null; then \
		echo "Node.js não encontrado. Por favor, instale via nvm ou site oficial."; \
		exit 1; \
	fi
	@if ! command -v docker &> /dev/null; then \
		echo "Docker não encontrado. Por favor, instale."; \
		exit 1; \
	fi
	rustup update
	(cd frontend && npm install)
	@echo "✅ Dependências instaladas."

# Desenvolvimento Local (Monolito)
# ------------------------------------------------------------------------------
.PHONY: dev
dev:
	@echo "🚀 Iniciando ambiente de desenvolvimento (Tauri)..."
	npm run tauri dev

# Build
# ------------------------------------------------------------------------------
.PHONY: build
build:
	@echo "🏗️  Compilando todos os microsserviços em modo release..."
	cargo build --workspace --release
	@echo "✅ Build concluído."

.PHONY: build-local
build-local:
	@echo "📦 Compilando aplicação Tauri para a plataforma local..."
	npm run tauri build
	@echo "✅ Build local concluído."

# Testes e Qualidade
# ------------------------------------------------------------------------------
.PHONY: test
test:
	@echo "🧪 Rodando todos os testes..."
	cargo test --workspace
	(cd frontend && npm test)
	@echo "✅ Testes concluídos."

.PHONY: lint
lint:
	@echo "🔍 Rodando linters..."
	cargo fmt --all -- --check
	cargo clippy --workspace -- -D warnings
	(cd frontend && npm run lint)
	@echo "✅ Linting concluído."

# Limpeza
# ------------------------------------------------------------------------------
.PHONY: clean
clean:
	@echo "🧹 Limpando artefatos de build..."
	cargo clean
	(cd frontend && rm -rf node_modules build dist)
	@echo "✅ Limpeza concluída."

# Microsserviços
# ------------------------------------------------------------------------------
.PHONY: proto-gen
proto-gen:
	@echo "🤖 Gerando código a partir dos arquivos .proto..."
	(cd services/proto && buf generate)
	@echo "✅ Código gerado."

# Docker
# ------------------------------------------------------------------------------
.PHONY: docker-up
docker-up:
	@echo "🐳 Iniciando todos os serviços com Docker Compose..."
	$(DOCKER_COMPOSE_DEV) up -d
	@echo "✅ Serviços iniciados. Use 'make docker-logs' para ver os logs."

.PHONY: docker-down
docker-down:
	@echo "🛑 Parando e removendo todos os containers..."
	$(DOCKER_COMPOSE_DEV) down --remove-orphans
	@echo "✅ Containers parados."

.PHONY: docker-build
docker-build:
	@echo "🛠️  Buildando imagens Docker para todos os serviços..."
	$(DOCKER_COMPOSE_DEV) build --parallel
	@echo "✅ Imagens construídas."

.PHONY: docker-logs
docker-logs:
	@echo "📜 Mostrando logs de todos os serviços..."
	$(DOCKER_COMPOSE_DEV) logs -f

# Kubernetes (Exemplo)
# ------------------------------------------------------------------------------
.PHONY: deploy-k8s
deploy-k8s:
	@echo "🚀 Aplicando configurações do Kubernetes..."
	kubectl apply -f infrastructure/k8s/
	@echo "✅ Deploy concluído."

# Utilitários
# ------------------------------------------------------------------------------
.PHONY: list-services
list-services:
	@echo "Serviços disponíveis:"
	@for service in $(SERVICES); do \
		echo "  - $$service"; \
	done

.PHONY: check-env
check-env:
	@echo "Verificando variáveis de ambiente necessárias..."
	@if [ -z "$(NATS_URL)" ]; then \
		echo "Aviso: NATS_URL não definida."; \
	fi
	@if [ -z "$(DATABASE_URL)" ]; then \
		echo "Aviso: DATABASE_URL não definida."; \
	fi
	@echo "✅ Verificação concluída."

# Service-specific commands
auth-logs:
	docker-compose logs -f auth-service

masker-logs:
	docker-compose logs -f masker-service

storage-logs:
	docker-compose logs -f storage-service

search-logs:
	docker-compose logs -f search-service

agent-logs:
	docker-compose logs -f agent-service

# Database commands
db-shell:
	docker-compose exec postgres psql -U keyai -d keyai

redis-cli:
	docker-compose exec redis redis-cli

# Kong API Gateway commands
kong-reload:
	docker-compose exec kong kong reload

kong-status:
	curl -s http://localhost:8001/status | jq

# Health checks
health-check:
	@echo "🏥 Checking service health..."
	@curl -s http://localhost:8000/api/v1/auth/health | jq
	@curl -s http://localhost:8000/api/v1/agent/health | jq
	@curl -s http://localhost:8000/api/v1/masker/health | jq
	@curl -s http://localhost:8000/api/v1/storage/health | jq
	@curl -s http://localhost:8000/api/v1/search/health | jq

# Generate OpenAPI documentation
docs:
	@echo "📚 Generating API documentation..."
	@echo "Access Swagger UI at: http://localhost:8000/swagger-ui"

# Benchmarks
bench:
	@echo "📊 Running benchmarks..."
	cd services/search-service && cargo bench
	cd services/agent-service && cargo bench 