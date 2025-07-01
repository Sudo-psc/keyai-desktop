# 🔍 APIs Alternativas para Captura de Teclado no macOS

## 📋 Resumo Executivo

Após investigação detalhada das opções disponíveis para captura de teclado no macOS, identificamos que o **problema do KeyAI não é o `rdev` em si**, mas sim **conflitos específicos com atalhos de sistema do macOS** (principalmente Cmd+Shift+3/4 para screenshots).

## 🎯 Recomendação Principal

**MANTER `rdev` com melhorias incrementais** ao invés de migrar para uma API completamente diferente.

### ✅ Razões para Manter `rdev`:
1. **Captura mouse funciona perfeitamente** (centenas de eventos capturados sem problemas)
2. **Infraestrutura robusta** já implementada
3. **Multiplataforma** (Windows, Linux, macOS)
4. **Comunidade ativa** (621 stars, 169 forks)
5. **Bem documentado** e maduro

### 🔧 Melhorias Implementadas:
- Filtros inteligentes para atalhos de sistema
- Modo seguro temporário para macOS
- Diagnósticos detalhados de TCC
- Tratamento robusto de panics

## 📊 Análise das Alternativas

### 1. **CGEventTap (Core Graphics)**

#### ✅ Vantagens:
- API nativa do macOS
- Controle total sobre eventos
- Pode interceptar e modificar eventos
- Usado por apps comerciais como Keystroke Pro

#### ❌ Desvantagens:
- **Requer implementação do zero** em Rust
- Mais complexo que `rdev`
- Necessita binding manual para Core Foundation
- Mesmo problema de conflitos com atalhos de sistema

#### 📝 Implementação:
```rust
// Exemplo de binding necessário
let event_tap = CGEventTapCreate(
    kCGSessionEventTap,
    kCGHeadInsertEventTap,
    kCGEventTapOptionDefault,
    event_mask,
    event_callback,
    std::ptr::null_mut()
);
```

### 2. **Accessibility API (AXUIElement)**

#### ✅ Vantagens:
- Pode obter contexto de UI
- Identifica elementos específicos
- Usado em ferramentas como try! Swift 2024 demos

#### ❌ Desvantagens:
- **Não é para captura global de teclas**
- Focado em automação de UI
- Requer que elementos tenham acessibilidade habilitada
- Não resolve o problema principal

#### 📝 Uso Típico:
```swift
// Exemplo em Swift
let element = AXUIElementCopyElementAtPosition(systemWide, x, y)
let pid = AXUIElementGetPid(element)
```

### 3. **Hammerspoon (Lua)**

#### ✅ Vantagens:
- Framework maduro para automação
- Comunidade ativa
- Exemplos disponíveis (hammerspoon-activity-logger)

#### ❌ Desvantagens:
- **Não é Rust** (requer integração complexa)
- Adiciona dependência externa
- Não resolve conflitos de sistema
- Arquitetura diferente do KeyAI

### 4. **Claves (Rust)**

#### ✅ Vantagens:
- Biblioteca Rust nativa
- Suporte macOS + Windows
- Interface simples

#### ❌ Desvantagens:
- **Projeto pequeno** (1 star, 1 fork)
- Pouca documentação
- Não há evidência de resolver conflitos de sistema
- Menos maduro que `rdev`

#### 📝 API:
```rust
let receiver = claves::init();
dbg!(receiver.recv().unwrap());
```

### 5. **CGEventSupervisor (Swift)**

#### ✅ Vantagens:
- Wrapper moderno para CGEventTap
- Interface Swift elegante
- Suporte a cancelamento de eventos

#### ❌ Desvantagens:
- **Requer Swift/Objective-C**
- Não integra diretamente com Rust
- Ainda usa CGEventTap por baixo
- Mesmo problema de conflitos

## 🔄 Estratégia de Migração (Se Necessário)

### Fase 1: Validação
1. **Implementar PoC com CGEventTap** diretamente
2. **Testar se resolve** os conflitos com Cmd+Shift+3
3. **Comparar performance** com `rdev`

### Fase 2: Implementação Híbrida
```rust
#[cfg(target_os = "macos")]
mod macos_native {
    // Implementação CGEventTap
}

#[cfg(not(target_os = "macos"))]
mod cross_platform {
    // Continuar usando rdev
}
```

### Fase 3: Migração Gradual
1. Manter `rdev` para mouse
2. Usar CGEventTap apenas para teclado
3. Avaliar resultados

## 🛠️ Implementação Recomendada

### Opção A: Melhoria do `rdev` (RECOMENDADA)
```rust
// Continuar com filtros inteligentes
fn should_filter_system_shortcut(event: &Event) -> bool {
    match event.event_type {
        EventType::KeyPress(Key::Num3) | EventType::KeyPress(Key::Num4) => {
            // Detectar se Cmd+Shift está pressionado
            true
        }
        _ => false
    }
}
```

### Opção B: CGEventTap Nativo
```rust
// Implementação direta (mais trabalho)
use core_foundation::*;
use core_graphics::*;

fn create_event_tap() -> Result<CFMachPortRef, Error> {
    let event_mask = CGEventMaskBit(kCGEventKeyDown) | 
                     CGEventMaskBit(kCGEventKeyUp);
    
    CGEventTapCreate(
        kCGSessionEventTap,
        kCGHeadInsertEventTap,
        kCGEventTapOptionDefault,
        event_mask,
        event_callback,
        std::ptr::null_mut()
    )
}
```

## 📈 Matriz de Decisão

| Critério | rdev + Melhorias | CGEventTap | Accessibility API | Claves |
|----------|------------------|------------|------------------|--------|
| **Facilidade** | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐ | ⭐⭐⭐ |
| **Maturidade** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ |
| **Performance** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ |
| **Controle** | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ |
| **Multiplataforma** | ⭐⭐⭐⭐⭐ | ⭐ | ⭐ | ⭐⭐⭐⭐ |
| **Resolve Problema** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐ | ⭐⭐ |

## 🎯 Conclusão

**Recomendação: Continuar com `rdev` + melhorias incrementais**

### Próximos Passos:
1. ✅ **Implementar filtros mais inteligentes** (já feito)
2. 🔄 **Testar com usuários reais**
3. 📊 **Monitorar logs de crash**
4. 🔧 **Refinar filtros conforme necessário**

### Se os filtros não resolverem:
1. Implementar PoC com CGEventTap
2. Comparar estabilidade e performance
3. Migração gradual se comprovada a melhoria

O `rdev` continua sendo a melhor opção para o KeyAI, especialmente considerando que:
- **Mouse funciona perfeitamente**
- **Problema é específico e solucionável**
- **Migração seria custosa sem garantia de melhoria**

---

*Documento criado em: 30/06/2025*  
*Status: Investigação Completa*  
*Decisão: Manter rdev com melhorias* 