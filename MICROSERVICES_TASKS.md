# KeyAI Microservices Implementation Tasks

Este arquivo acompanha o progresso da refatoração para arquitetura de microsserviços API-first.

## Completed Tasks

### Infrastructure
- [x] Criar Docker Compose completo com todos os serviços
- [x] Configurar Prometheus para observabilidade
- [x] Configurar Kong API Gateway (arquivo base)
- [x] Criar Makefile para automação de builds e deploy
- [x] Criar documentação API_FIRST_REFACTOR.md

### Auth Service
- [x] Criar estrutura base do Auth Service
- [x] Implementar main.rs com Axum e OpenAPI
- [x] Configurar JWT e autenticação
- [x] Criar Dockerfile otimizado
- [x] Adicionar métricas Prometheus

### Capture Service
- [x] Atualizar Cargo.toml com dependências REST
- [x] Implementar main.rs com API REST e RabbitMQ
- [x] Criar config.rs para configurações
- [x] Criar error.rs para tratamento de erros
- [x] Criar metrics.rs para observabilidade
- [x] Criar Dockerfile otimizado

### Masker Service
- [x] Atualizar Cargo.toml com dependências
- [x] Verificar main.rs (já implementado com API REST)
- [x] Criar Dockerfile otimizado

### Documentation
- [x] Criar OpenAPI consolidada em docs/openapi/keyai-api-v1.yaml
- [x] Criar docker-compose simplificado para desenvolvimento

## In Progress Tasks

### Storage Service
- [ ] Criar Cargo.toml com dependências
- [ ] Implementar main.rs com API REST
- [ ] Criar schemas do banco de dados
- [ ] Implementar consumer RabbitMQ
- [ ] Criar Dockerfile

### Search Service
- [ ] Atualizar Cargo.toml com dependências
- [ ] Implementar busca híbrida (FTS5 + vetorial)
- [ ] Integrar com Redis para cache
- [ ] Criar API REST com paginação
- [ ] Criar Dockerfile

## Future Tasks

### Kong API Gateway
- [ ] Implementar rate limiting por usuário
- [ ] Configurar cache de respostas
- [ ] Adicionar circuit breakers
- [ ] Configurar load balancing

### Security
- [ ] Implementar RBAC (Role-Based Access Control)
- [ ] Adicionar API keys para serviços
- [ ] Configurar TLS/SSL em todos os serviços
- [ ] Implementar audit logging

### Testing
- [ ] Criar testes unitários para cada serviço
- [ ] Implementar testes de integração
- [ ] Criar testes de carga com k6
- [ ] Implementar testes de contrato

### CI/CD
- [ ] Criar GitHub Actions para build de cada serviço
- [ ] Configurar deploy automático para staging
- [ ] Implementar blue-green deployment
- [ ] Criar health checks automatizados

### Monitoring & Observability
- [ ] Criar dashboards Grafana para cada serviço
- [ ] Configurar alertas no Prometheus
- [ ] Implementar distributed tracing com Jaeger
- [ ] Adicionar logging centralizado com Loki

### Performance
- [ ] Implementar connection pooling otimizado
- [ ] Adicionar cache distribuído com Redis
- [ ] Otimizar queries do banco de dados
- [ ] Implementar batch processing

### Frontend Integration
- [ ] Atualizar frontend para usar nova API REST
- [ ] Implementar retry logic no cliente
- [ ] Adicionar cache local
- [ ] Criar SDK TypeScript para API

## Implementation Plan

### Sprint 1 (Current)
- Complete Storage and Search services
- Basic Kong configuration
- Integration tests

### Sprint 2
- Security implementation
- Performance optimizations
- Frontend integration

### Sprint 3
- Full CI/CD pipeline
- Production deployment configuration
- Load testing and optimization

### Sprint 4
- Advanced features (analytics, ML)
- Documentation and training
- Migration tools

## Relevant Files

### Infrastructure
- `docker-compose.yml` - Orquestração completa ✅
- `docker/docker-compose.microservices.yml` - Versão simplificada ✅
- `docker/prometheus.yml` - Configuração de métricas ✅
- `docker/kong.yml` - API Gateway config ✅
- `Makefile` - Automação de builds ✅

### Services
- `services/auth-service/` - Serviço de autenticação ✅
- `services/capture-service/` - Captura de teclas ✅
- `services/masker-service/` - Mascaramento PII ✅
- `services/storage-service/` - Persistência (WIP)
- `services/search-service/` - Busca híbrida (WIP)

### Documentation
- `API_FIRST_REFACTOR.md` - Guia de refatoração ✅
- `docs/openapi/keyai-api-v1.yaml` - API specification ✅

## Notes

- Todos os serviços devem seguir o padrão REST com OpenAPI
- Usar RabbitMQ para comunicação assíncrona
- PostgreSQL como banco principal, Redis para cache
- Métricas em todos os endpoints com Prometheus
- Health checks padronizados em `/health` 