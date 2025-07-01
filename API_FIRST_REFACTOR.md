# KeyAI API-First Microservices Refactor

## 🎯 Objetivo

Transformar a aplicação desktop monolítica KeyAI em uma arquitetura de microsserviços API-first, mantendo compatibilidade com o frontend existente.

## 🏗️ Arquitetura Implementada

### Componentes Principais

1. **API Gateway (Kong)**
   - Roteamento centralizado
   - Autenticação/Autorização
   - Rate limiting
   - Métricas e observabilidade

2. **Microsserviços**
   - **Auth Service** (Port 3005): Autenticação JWT, gestão de usuários
   - **Agent Service** (Port 3001): Captura de eventos de teclado
   - **Masker Service** (Port 3002): Mascaramento de PII
   - **Storage Service** (Port 3003): Persistência de dados
   - **Search Service** (Port 3004): Busca híbrida (textual + semântica)

3. **Infraestrutura**
   - **PostgreSQL**: Banco de dados principal
   - **Redis**: Cache e sessões
   - **RabbitMQ**: Mensageria assíncrona
   - **Prometheus + Grafana**: Monitoramento
   - **Jaeger**: Distributed tracing

## 🚀 Quick Start

### Pré-requisitos
- Docker & Docker Compose
- Make (opcional)
- 8GB RAM mínimo

### Iniciando os Serviços

```bash
# Clone o repositório
git clone https://github.com/Sudo-psc/keyai-desktop.git
cd keyai-desktop

# Checkout da branch de refatoração
git checkout feature/api-first-refactor

# Copie as variáveis de ambiente
cp .env.example .env

# Inicie todos os serviços
make up

# Ou sem Make:
docker-compose up -d
```

### Verificando a Saúde dos Serviços

```bash
# Verificar status de todos os serviços
make health-check

# Ver logs
make logs

# Logs de serviço específico
make auth-logs
make search-logs
```

## 📡 Endpoints da API

### API Gateway (Kong)
- **Base URL**: `http://localhost:8000`
- **Admin API**: `http://localhost:8001`

### Autenticação
```bash
# Registrar usuário
curl -X POST http://localhost:8000/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "securePassword123!",
    "name": "John Doe"
  }'

# Login
curl -X POST http://localhost:8000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "securePassword123!"
  }'
```

### Captura de Eventos
```bash
# Enviar evento (requer autenticação)
curl -X POST http://localhost:8000/api/v1/agent/events \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "type": "keypress",
    "data": "Hello World",
    "timestamp": "2024-01-15T10:00:00Z"
  }'
```

### Busca
```bash
# Buscar eventos
curl -X GET "http://localhost:8000/api/v1/search?query=hello&type=hybrid" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

## 📚 Documentação da API

### OpenAPI/Swagger
- Swagger UI: `http://localhost:8000/swagger-ui`
- OpenAPI JSON: `http://localhost:8000/api-docs/openapi.json`

### Postman Collection
Importe o arquivo `postman/KeyAI_API.postman_collection.json`

## 🔧 Desenvolvimento

### Estrutura de Código

```
services/
├── auth-service/      # Autenticação e autorização
├── agent-service/     # Captura de eventos
├── masker-service/    # Mascaramento de PII
├── storage-service/   # Persistência
└── search-service/    # Motor de busca

docker/
├── kong.yml          # Configuração do API Gateway
├── prometheus.yml    # Configuração de métricas
└── grafana/         # Dashboards
```

### Adicionando um Novo Serviço

1. Crie a estrutura em `services/seu-servico/`
2. Adicione ao `docker-compose.yml`
3. Configure rotas no Kong (`docker/kong.yml`)
4. Adicione métricas ao Prometheus
5. Crie testes de integração

### Convenções de API

- Versionamento: `/api/v1/`
- Autenticação: Bearer JWT
- Respostas: JSON com envelope padrão
- Erros: RFC 7807 (Problem Details)

## 🧪 Testes

```bash
# Testes unitários
make test

# Testes de integração
cd tests/integration && cargo test

# Testes de carga
k6 run tests/load/search.js
```

## 📊 Monitoramento

### Dashboards
- **Grafana**: http://localhost:3000 (admin/admin)
- **Prometheus**: http://localhost:9090
- **Jaeger**: http://localhost:16686
- **RabbitMQ**: http://localhost:15672 (keyai/keyaipass)

### Métricas Principais
- Latência p95 por endpoint
- Taxa de erro por serviço
- Throughput de mensagens
- Uso de CPU/Memória
- Cache hit rate

## 🔒 Segurança

### Implementado
- ✅ Autenticação JWT
- ✅ Rate limiting por IP/usuário
- ✅ CORS configurado
- ✅ Validação de entrada
- ✅ Secrets em variáveis de ambiente
- ✅ TLS entre serviços (dev: self-signed)

### Próximos Passos
- [ ] OAuth2/OIDC support
- [ ] API keys para serviços
- [ ] Audit logging
- [ ] WAF rules

## 🚢 Deploy em Produção

### Kubernetes
```bash
# Aplicar manifestos
kubectl apply -f k8s/

# Verificar pods
kubectl get pods -n keyai

# Escalar serviço
kubectl scale deployment search-service --replicas=5 -n keyai
```

### Docker Swarm
```bash
# Inicializar swarm
docker swarm init

# Deploy stack
docker stack deploy -c docker-compose.prod.yml keyai
```

## 🐛 Troubleshooting

### Serviço não inicia
```bash
# Verificar logs
docker-compose logs nome-do-servico

# Verificar conectividade
docker-compose exec nome-do-servico ping outro-servico
```

### Problemas de autenticação
```bash
# Verificar JWT secret
echo $JWT_SECRET

# Testar token
curl http://localhost:8000/api/v1/auth/verify \
  -H "Authorization: Bearer YOUR_TOKEN"
```

### Performance
```bash
# Verificar métricas
curl http://localhost:8000/metrics

# Profiling
go tool pprof http://localhost:6060/debug/pprof/profile
```

## 📝 Roadmap

### Fase 1 (Concluída) ✅
- [x] Docker Compose setup
- [x] Kong API Gateway
- [x] Auth Service com JWT
- [x] Observabilidade básica

### Fase 2 (Em Progresso) 🚧
- [ ] Migração completa dos endpoints
- [ ] Testes de integração
- [ ] CI/CD pipeline
- [ ] Documentação completa

### Fase 3 (Planejada) 📋
- [ ] Kubernetes manifests
- [ ] Helm charts
- [ ] Service mesh (Istio)
- [ ] GitOps (ArgoCD)

## 🤝 Contribuindo

1. Crie uma feature branch
2. Implemente com testes
3. Atualize documentação
4. Abra PR com descrição detalhada

## 📄 Licença

MIT License - veja LICENSE para detalhes. 