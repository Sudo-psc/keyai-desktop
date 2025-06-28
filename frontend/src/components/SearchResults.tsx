import { useState } from 'react'
import { FileText, Calendar, Hash, Brain, FileSearch, ChevronDown, ChevronUp } from 'lucide-react'
import { SearchResult, HybridSearchResult, SearchType } from '../types'
import { format } from 'date-fns'
import { ptBR } from 'date-fns/locale'

interface SearchResultsProps {
  results: (SearchResult | HybridSearchResult)[]
  searchType: SearchType
}

export default function SearchResults({ results, searchType }: SearchResultsProps) {
  const [expandedResults, setExpandedResults] = useState<Set<number>>(new Set())

  const toggleExpanded = (id: number) => {
    const newExpanded = new Set(expandedResults)
    if (newExpanded.has(id)) {
      newExpanded.delete(id)
    } else {
      newExpanded.add(id)
    }
    setExpandedResults(newExpanded)
  }

  const formatTimestamp = (timestamp: number): string => {
    const date = new Date(timestamp * 1000)
    return format(date, "dd/MM/yyyy 'às' HH:mm:ss", { locale: ptBR })
  }

  const formatScore = (score: number): string => {
    return (score * 100).toFixed(1) + '%'
  }

  const isHybridResult = (result: SearchResult | HybridSearchResult): result is HybridSearchResult => {
    return 'text_score' in result && 'semantic_score' in result
  }

  const getRelevanceColor = (score: number): string => {
    if (score >= 0.8) return 'text-green-400'
    if (score >= 0.6) return 'text-yellow-400'
    if (score >= 0.4) return 'text-orange-400'
    return 'text-red-400'
  }

  const highlightSearchTerms = (text: string): string => {
    // Esta é uma implementação simplificada
    // Em produção, você destacaria os termos de busca reais
    return text
  }

  return (
    <div className="card">
      <div className="p-4 border-b border-slate-700">
        <h3 className="text-lg font-semibold flex items-center">
          <FileSearch className="w-5 h-5 mr-2 text-primary-500" />
          Resultados da Busca
          <span className="ml-2 text-sm text-slate-400">
            ({results.length} resultado{results.length !== 1 ? 's' : ''})
          </span>
        </h3>
      </div>

      <div className="divide-y divide-slate-700">
        {results.map((result) => {
          const isExpanded = expandedResults.has(result.id)
          const isHybrid = isHybridResult(result)
          
          return (
            <div key={result.id} className="search-result">
              {/* Cabeçalho do Resultado */}
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <div className="flex items-center space-x-3 mb-2">
                    <FileText className="w-4 h-4 text-slate-400 flex-shrink-0" />
                    <span className="text-sm text-slate-300 font-medium">
                      {result.context || 'Sem contexto'}
                    </span>
                    <span className="text-xs text-slate-500">
                      <Calendar className="inline-block w-3 h-3 mr-1" />
                      {formatTimestamp(result.timestamp)}
                    </span>
                  </div>

                  {/* Conteúdo */}
                  <div className="ml-7">
                    <p className={`text-sm text-slate-200 ${!isExpanded ? 'line-clamp-2' : ''}`}>
                      {highlightSearchTerms(result.content)}
                    </p>
                  </div>

                  {/* Scores */}
                  <div className="ml-7 mt-2 flex items-center space-x-4 text-xs">
                    {searchType === 'text' && (
                      <div className="flex items-center space-x-1">
                        <FileSearch className="w-3 h-3 text-slate-400" />
                        <span className="text-slate-400">Relevância:</span>
                        <span className={getRelevanceColor(isHybrid ? result.text_score : result.relevance_score)}>
                          {formatScore(isHybrid ? result.text_score : result.relevance_score)}
                        </span>
                      </div>
                    )}

                    {searchType === 'semantic' && isHybrid && (
                      <div className="flex items-center space-x-1">
                        <Brain className="w-3 h-3 text-slate-400" />
                        <span className="text-slate-400">Semântica:</span>
                        <span className={getRelevanceColor(result.semantic_score)}>
                          {formatScore(result.semantic_score)}
                        </span>
                      </div>
                    )}

                    {searchType === 'hybrid' && isHybrid && (
                      <>
                        <div className="flex items-center space-x-1">
                          <FileSearch className="w-3 h-3 text-slate-400" />
                          <span className="text-slate-400">Textual:</span>
                          <span className={getRelevanceColor(result.text_score)}>
                            {formatScore(result.text_score)}
                          </span>
                        </div>
                        <div className="flex items-center space-x-1">
                          <Brain className="w-3 h-3 text-slate-400" />
                          <span className="text-slate-400">Semântica:</span>
                          <span className={getRelevanceColor(result.semantic_score)}>
                            {formatScore(result.semantic_score)}
                          </span>
                        </div>
                        <div className="flex items-center space-x-1">
                          <Hash className="w-3 h-3 text-slate-400" />
                          <span className="text-slate-400">Combinado:</span>
                          <span className={`font-medium ${getRelevanceColor(result.combined_score)}`}>
                            {formatScore(result.combined_score)}
                          </span>
                        </div>
                      </>
                    )}
                  </div>
                </div>

                {/* Botão Expandir/Recolher */}
                {result.content.length > 100 && (
                  <button
                    onClick={() => toggleExpanded(result.id)}
                    className="ml-4 p-1 hover:bg-slate-700 rounded transition-colors"
                    title={isExpanded ? 'Recolher' : 'Expandir'}
                  >
                    {isExpanded ? (
                      <ChevronUp className="w-4 h-4 text-slate-400" />
                    ) : (
                      <ChevronDown className="w-4 h-4 text-slate-400" />
                    )}
                  </button>
                )}
              </div>

              {/* Metadados Expandidos */}
              {isExpanded && (
                <div className="mt-4 ml-7 p-3 bg-slate-800 rounded-lg text-xs space-y-1">
                  <div className="flex items-center space-x-2">
                    <span className="text-slate-500">ID:</span>
                    <span className="text-slate-400 font-mono">{result.id}</span>
                  </div>
                  <div className="flex items-center space-x-2">
                    <span className="text-slate-500">Timestamp Unix:</span>
                    <span className="text-slate-400 font-mono">{result.timestamp}</span>
                  </div>
                  {result.context && (
                    <div className="flex items-center space-x-2">
                      <span className="text-slate-500">Aplicação:</span>
                      <span className="text-slate-400">{result.context}</span>
                    </div>
                  )}
                </div>
              )}
            </div>
          )
        })}
      </div>

      {/* Rodapé com Resumo */}
      {results.length > 0 && (
        <div className="p-4 border-t border-slate-700 bg-slate-800/50">
          <div className="flex items-center justify-between text-sm">
            <span className="text-slate-400">
              Mostrando {results.length} resultado{results.length !== 1 ? 's' : ''}
            </span>
            {searchType === 'hybrid' && (
              <span className="text-slate-400">
                Ordenado por relevância combinada (RRF)
              </span>
            )}
          </div>
        </div>
      )}
    </div>
  )
} 