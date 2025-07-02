import React, { useState } from 'react'
import { Search, Sparkles, Type, History } from 'lucide-react'
import GlassButton from './ui/GlassButton'
import GlassInput from './ui/GlassInput'
import GlassCard from './ui/GlassCard'
import { useSearch } from '../hooks/useSearch'
import '../styles/liquid-glass.css'

const SearchInterface: React.FC = () => {
  const [query, setQuery] = useState('')
  const [searchType, setSearchType] = useState<'text' | 'semantic' | 'hybrid'>('hybrid')
  const { search, isLoading, results } = useSearch()

  const handleSearch = async () => {
    if (!query.trim()) return

    try {
      await search(query, searchType)
    } catch (error) {
      console.error('Erro na busca:', error)
    }
  }

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSearch()
    }
  }

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      {/* Card Principal de Busca */}
      <GlassCard variant="primary" size="lg" hoverable={false}>
        <div className="space-y-6">
          {/* Header */}
          <div className="flex items-center gap-4">
            <div className="p-3 rounded-xl bg-electric-blue/20">
              <Search className="w-6 h-6 text-electric-blue" />
            </div>
            <div>
              <h2 className="text-title font-semibold gradient-text">
                Busca Inteligente
              </h2>
              <p className="text-caption text-secondary mt-1">
                Encontre qualquer coisa em seu histórico de digitação
              </p>
            </div>
          </div>

          {/* Input de Busca */}
          <div className="space-y-4">
            <GlassInput
              icon={Search}
              placeholder="Digite para buscar em seu histórico..."
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyPress={handleKeyPress}
              className="text-lg"
              containerClassName="w-full"
            />

            {/* Tipos de Busca */}
            <div className="flex flex-wrap gap-3">
              <GlassButton
                variant={searchType === 'text' ? 'primary' : 'secondary'}
                size="sm"
                icon={Type}
                onClick={() => setSearchType('text')}
              >
                Busca Textual
              </GlassButton>

              <GlassButton
                variant={searchType === 'semantic' ? 'primary' : 'secondary'}
                size="sm"
                icon={Sparkles}
                onClick={() => setSearchType('semantic')}
              >
                Busca Semântica
              </GlassButton>

              <GlassButton
                variant={searchType === 'hybrid' ? 'success' : 'secondary'}
                size="sm"
                icon={Search}
                onClick={() => setSearchType('hybrid')}
              >
                Busca Híbrida
              </GlassButton>
            </div>

            {/* Botão de Busca */}
            <div className="flex justify-end">
              <GlassButton
                onClick={handleSearch}
                loading={isLoading}
                disabled={!query.trim()}
                size="lg"
                className="min-w-32"
              >
                {isLoading ? 'Buscando...' : 'Buscar'}
              </GlassButton>
            </div>
          </div>
        </div>
      </GlassCard>

      {/* Dicas de Busca */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <GlassCard size="sm" hoverable={false}>
          <div className="text-center space-y-2">
            <Type className="w-8 h-8 text-electric-blue mx-auto" />
            <h3 className="text-body font-medium">Busca Textual</h3>
            <p className="text-small text-secondary">
              Encontra correspondências exatas de palavras e frases
            </p>
          </div>
        </GlassCard>

        <GlassCard size="sm" hoverable={false}>
          <div className="text-center space-y-2">
            <Sparkles className="w-8 h-8 text-mint-green mx-auto" />
            <h3 className="text-body font-medium">Busca Semântica</h3>
            <p className="text-small text-secondary">
              Entende o significado e contexto da sua busca
            </p>
          </div>
        </GlassCard>

        <GlassCard size="sm" hoverable={false}>
          <div className="text-center space-y-2">
            <Search className="w-8 h-8 text-electric-blue mx-auto" />
            <h3 className="text-body font-medium">Busca Híbrida</h3>
            <p className="text-small text-secondary">
              Combina ambas as técnicas para melhores resultados
            </p>
          </div>
        </GlassCard>
      </div>

      {/* Busca Rápida */}
      <GlassCard hoverable={false}>
        <div className="flex items-center gap-3 mb-4">
          <History className="w-5 h-5 text-secondary" />
          <h3 className="text-heading font-medium">Buscas Rápidas</h3>
        </div>

        <div className="flex flex-wrap gap-2">
          {[
            'senhas salvas',
            'emails importantes',
            'código Python',
            'reunião ontem',
            'projeto keyai',
            'documentos PDF'
          ].map((suggestion) => (
            <button
              key={suggestion}
              onClick={() => setQuery(suggestion)}
              className="px-3 py-1 text-small rounded-full bg-white/5 hover:bg-white/10
                         text-secondary hover:text-primary transition-all duration-200
                         border border-white/10 hover:border-white/20"
            >
              {suggestion}
            </button>
          ))}
        </div>
      </GlassCard>

      {/* Estatísticas Rápidas */}
      {!isLoading && !results.length && query && (
        <GlassCard variant="default" hoverable={false}>
          <div className="text-center space-y-2">
            <Search className="w-12 h-12 text-secondary mx-auto opacity-50" />
            <h3 className="text-heading">Nenhum resultado encontrado</h3>
            <p className="text-caption text-secondary">
              Tente usar palavras-chave diferentes ou verifique a ortografia
            </p>
          </div>
        </GlassCard>
      )}
    </div>
  )
}

export default SearchInterface
