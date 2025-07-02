#!/bin/bash

# 🚀 KeyAI Desktop - Microservices Development Helper
# Script para gerenciar os microsserviços localmente

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
COMPOSE_FILE="docker/docker-compose.microservices.yml"
SERVICES=("capture-service" "masker-service" "storage-service" "search-service" "auth-service")
INFRASTRUCTURE=("postgres" "redis" "rabbitmq")

print_header() {
    echo -e "${BLUE}"
    echo "=================================================="
    echo "🔑 KeyAI Desktop - Microservices Manager"
    echo "=================================================="
    echo -e "${NC}"
}

print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Function to check if Docker is running
check_docker() {
    if ! docker info > /dev/null 2>&1; then
        print_error "Docker não está rodando. Por favor, inicie o Docker primeiro."
        exit 1
    fi
}

# Function to check if Docker Compose file exists
check_compose_file() {
    if [ ! -f "$COMPOSE_FILE" ]; then
        print_error "Arquivo Docker Compose não encontrado: $COMPOSE_FILE"
        exit 1
    fi
}

# Function to build all services
build_services() {
    print_info "Construindo todos os microsserviços..."
    
    for service in "${SERVICES[@]}"; do
        print_info "Construindo $service..."
        if docker-compose -f "$COMPOSE_FILE" build "$service"; then
            print_status "$service construído com sucesso"
        else
            print_error "Falha ao construir $service"
            exit 1
        fi
    done
    
    print_status "Todos os serviços foram construídos com sucesso!"
}

# Function to start infrastructure
start_infrastructure() {
    print_info "Iniciando infraestrutura (PostgreSQL, Redis, RabbitMQ)..."
    
    docker-compose -f "$COMPOSE_FILE" up -d "${INFRASTRUCTURE[@]}"
    
    print_info "Aguardando infraestrutura ficar pronta..."
    sleep 10
    
    # Check if services are healthy
    for infra in "${INFRASTRUCTURE[@]}"; do
        if docker-compose -f "$COMPOSE_FILE" ps "$infra" | grep -q "healthy\|Up"; then
            print_status "$infra está rodando"
        else
            print_warning "$infra pode não estar pronto ainda"
        fi
    done
}

# Function to start specific service
start_service() {
    local service=$1
    print_info "Iniciando $service..."
    
    docker-compose -f "$COMPOSE_FILE" up -d "$service"
    
    sleep 5
    
    if docker-compose -f "$COMPOSE_FILE" ps "$service" | grep -q "Up"; then
        print_status "$service iniciado com sucesso"
        show_service_info "$service"
    else
        print_error "Falha ao iniciar $service"
        docker-compose -f "$COMPOSE_FILE" logs "$service" --tail=20
    fi
}

# Function to show service information
show_service_info() {
    local service=$1
    case $service in
        "capture-service")
            echo "  📍 URL: http://localhost:3001"
            echo "  🏥 Health: http://localhost:3001/health"
            echo "  📊 Metrics: http://localhost:3001/metrics"
            ;;
        "masker-service")
            echo "  📍 URL: http://localhost:3002"
            echo "  🏥 Health: http://localhost:3002/health"
            echo "  📊 Metrics: http://localhost:3002/metrics"
            ;;
        "storage-service")
            echo "  📍 URL: http://localhost:3003"
            echo "  🏥 Health: http://localhost:3003/health"
            echo "  📊 Metrics: http://localhost:3003/metrics"
            ;;
        "search-service")
            echo "  📍 URL: http://localhost:3004"
            echo "  🏥 Health: http://localhost:3004/health"
            ;;
        "auth-service")
            echo "  📍 URL: http://localhost:3005"
            echo "  🏥 Health: http://localhost:3005/health"
            ;;
    esac
}

# Function to check service health
check_health() {
    local service=$1
    local port=""
    
    case $service in
        "capture-service") port="3001" ;;
        "masker-service") port="3002" ;;
        "storage-service") port="3003" ;;
        "search-service") port="3004" ;;
        "auth-service") port="3005" ;;
    esac
    
    if [ -n "$port" ]; then
        print_info "Verificando health de $service na porta $port..."
        if curl -f -s "http://localhost:$port/health" > /dev/null; then
            print_status "$service está saudável"
        else
            print_error "$service não está respondendo ou não está saudável"
        fi
    fi
}

# Function to show logs
show_logs() {
    local service=$1
    local lines=${2:-50}
    
    print_info "Mostrando logs de $service (últimas $lines linhas)..."
    docker-compose -f "$COMPOSE_FILE" logs "$service" --tail="$lines" -f
}

# Function to stop services
stop_services() {
    print_info "Parando todos os serviços..."
    docker-compose -f "$COMPOSE_FILE" down
    print_status "Todos os serviços foram parados"
}

# Function to clean up
cleanup() {
    print_info "Limpando containers, volumes e redes..."
    docker-compose -f "$COMPOSE_FILE" down -v --remove-orphans
    docker system prune -f
    print_status "Limpeza concluída"
}

# Function to show status
show_status() {
    print_info "Status dos serviços:"
    echo ""
    
    # Infrastructure status
    echo -e "${YELLOW}📦 Infraestrutura:${NC}"
    for infra in "${INFRASTRUCTURE[@]}"; do
        if docker-compose -f "$COMPOSE_FILE" ps "$infra" | grep -q "Up"; then
            echo -e "  ${GREEN}✅ $infra${NC}"
        else
            echo -e "  ${RED}❌ $infra${NC}"
        fi
    done
    
    echo ""
    
    # Services status
    echo -e "${YELLOW}🛠️  Microsserviços:${NC}"
    for service in "${SERVICES[@]}"; do
        if docker-compose -f "$COMPOSE_FILE" ps "$service" | grep -q "Up"; then
            echo -e "  ${GREEN}✅ $service${NC}"
            check_health "$service"
        else
            echo -e "  ${RED}❌ $service${NC}"
        fi
    done
}

# Function to run tests
run_tests() {
    print_info "Executando testes dos microsserviços..."
    
    # Test capture-service
    if docker-compose -f "$COMPOSE_FILE" ps "capture-service" | grep -q "Up"; then
        print_info "Testando capture-service..."
        cargo test --manifest-path services/capture-service/Cargo.toml
    fi
    
    # Test masker-service
    if docker-compose -f "$COMPOSE_FILE" ps "masker-service" | grep -q "Up"; then
        print_info "Testando masker-service..."
        cargo test --manifest-path services/masker-service/Cargo.toml
    fi
    
    # Test storage-service
    if docker-compose -f "$COMPOSE_FILE" ps "storage-service" | grep -q "Up"; then
        print_info "Testando storage-service..."
        cargo test --manifest-path services/storage-service/Cargo.toml
    fi
    
    print_status "Testes concluídos"
}

# Function to show help
show_help() {
    echo -e "${BLUE}KeyAI Desktop - Microservices Development Helper${NC}"
    echo ""
    echo "Uso: $0 [COMANDO] [OPÇÕES]"
    echo ""
    echo "Comandos disponíveis:"
    echo "  build               Constrói todos os microsserviços"
    echo "  start               Inicia infraestrutura e todos os serviços"
    echo "  start <service>     Inicia um serviço específico"
    echo "  stop                Para todos os serviços"
    echo "  restart             Para e inicia todos os serviços"
    echo "  status              Mostra status de todos os serviços"
    echo "  logs <service>      Mostra logs de um serviço"
    echo "  health              Verifica health de todos os serviços"
    echo "  test                Executa testes dos serviços"
    echo "  clean               Para serviços e limpa containers/volumes"
    echo "  help                Mostra esta ajuda"
    echo ""
    echo "Serviços disponíveis:"
    for service in "${SERVICES[@]}"; do
        echo "  - $service"
    done
    echo ""
    echo "Exemplos:"
    echo "  $0 start                    # Inicia tudo"
    echo "  $0 start capture-service    # Inicia apenas capture-service"
    echo "  $0 logs masker-service      # Mostra logs do masker-service"
    echo "  $0 health                   # Verifica health de todos"
}

# Main script logic
main() {
    print_header
    
    # Check prerequisites
    check_docker
    check_compose_file
    
    # Parse command
    case ${1:-help} in
        "build")
            build_services
            ;;
        "start")
            if [ -n "$2" ]; then
                start_infrastructure
                start_service "$2"
            else
                start_infrastructure
                for service in "${SERVICES[@]}"; do
                    start_service "$service"
                done
                echo ""
                show_status
            fi
            ;;
        "stop")
            stop_services
            ;;
        "restart")
            stop_services
            sleep 2
            start_infrastructure
            for service in "${SERVICES[@]}"; do
                start_service "$service"
            done
            ;;
        "status")
            show_status
            ;;
        "logs")
            if [ -n "$2" ]; then
                show_logs "$2" "${3:-50}"
            else
                print_error "Por favor, especifique um serviço para ver os logs"
                exit 1
            fi
            ;;
        "health")
            for service in "${SERVICES[@]}"; do
                if docker-compose -f "$COMPOSE_FILE" ps "$service" | grep -q "Up"; then
                    check_health "$service"
                fi
            done
            ;;
        "test")
            run_tests
            ;;
        "clean")
            cleanup
            ;;
        "help"|*)
            show_help
            ;;
    esac
}

# Run main function with all arguments
main "$@" 