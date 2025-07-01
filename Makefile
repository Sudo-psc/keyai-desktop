.PHONY: help build test up down logs clean migrate lint format check

# Default target
help:
	@echo "KeyAI Microservices - Available Commands:"
	@echo "  make build         - Build all services"
	@echo "  make test          - Run all tests"
	@echo "  make up            - Start all services with docker-compose"
	@echo "  make down          - Stop all services"
	@echo "  make logs          - Show logs from all services"
	@echo "  make clean         - Clean build artifacts"
	@echo "  make migrate       - Run database migrations"
	@echo "  make lint          - Run linters"
	@echo "  make format        - Format code"
	@echo "  make check         - Run pre-commit checks"
	@echo "  make dev           - Run in development mode"
	@echo "  make prod          - Run in production mode"

# Build all services
build:
	@echo "🔨 Building all services..."
	docker-compose build --parallel

# Run tests
test:
	@echo "🧪 Running tests..."
	cd services/auth-service && cargo test
	cd services/agent-service && cargo test
	cd services/masker-service && cargo test
	cd services/storage-service && cargo test
	cd services/search-service && cargo test
	cd frontend && npm test

# Start services
up:
	@echo "🚀 Starting all services..."
	docker-compose up -d
	@echo "✅ Services started! Access:"
	@echo "  - API Gateway: http://localhost:8000"
	@echo "  - Kong Admin: http://localhost:8001"
	@echo "  - RabbitMQ: http://localhost:15672"
	@echo "  - Grafana: http://localhost:3000"
	@echo "  - Jaeger: http://localhost:16686"
	@echo "  - Prometheus: http://localhost:9090"

# Stop services
down:
	@echo "🛑 Stopping all services..."
	docker-compose down

# View logs
logs:
	docker-compose logs -f

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	docker-compose down -v
	find . -name "target" -type d -exec rm -rf {} +
	find . -name "node_modules" -type d -exec rm -rf {} +
	find . -name "dist" -type d -exec rm -rf {} +

# Run database migrations
migrate:
	@echo "🗄️  Running database migrations..."
	docker-compose run --rm storage-service sqlx migrate run
	docker-compose run --rm auth-service sqlx migrate run

# Lint code
lint:
	@echo "🔍 Running linters..."
	cd services/auth-service && cargo clippy -- -D warnings
	cd services/agent-service && cargo clippy -- -D warnings
	cd services/masker-service && cargo clippy -- -D warnings
	cd services/storage-service && cargo clippy -- -D warnings
	cd services/search-service && cargo clippy -- -D warnings
	cd frontend && npm run lint

# Format code
format:
	@echo "✨ Formatting code..."
	cd services/auth-service && cargo fmt
	cd services/agent-service && cargo fmt
	cd services/masker-service && cargo fmt
	cd services/storage-service && cargo fmt
	cd services/search-service && cargo fmt
	cd frontend && npm run format

# Pre-commit checks
check: lint test
	@echo "✅ All checks passed!"

# Development mode
dev:
	@echo "🔧 Starting in development mode..."
	docker-compose -f docker-compose.yml -f docker-compose.dev.yml up

# Production mode
prod:
	@echo "🚀 Starting in production mode..."
	docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d

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