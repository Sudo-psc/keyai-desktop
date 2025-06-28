import { useState, useCallback, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import { Search, Filter, Clock, FileText, Brain, Zap } from 'lucide-react'
import { SearchType, SearchResponse, HybridSearchResponse, HybridSearchResult, SearchResult } from '../types'
import SearchResults from './SearchResults'
import SearchHistory from './SearchHistory'

export default function SearchInterface() {
  const [query, setQuery] = useState('')
  const [searchType, setSearchType] = useState<SearchType>('hybrid')
  const [isSearching, setIsSearching] = useState(false)
  const [results, setResults] = useState<(SearchResult | HybridSearchResult)[]>([])
  const [searchTime, setSearchTime] = useState<number | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [suggestions, setSuggestions] = useState<string[]>([])
  const [showSuggestions, setShowSuggestions] = useState(false)
  const [searchHistory, setSearchHistory] = useState<string[]>([])

  // Configurações de busca híbrida
  const [textWeight, setTextWeight] = useState(0.7)
  const [semanticWeight, setSemanticWeight] = useState(0.3)
  const [resultLimit, setResultLimit] = useState(50)

  // Buscar sugestões enquanto digita
  useEffect(() => {
    const fetchSuggestions = async () => {
      if (query.length < 2) {
        setSuggestions([])
        return
      }

      try {
        const suggs = await invoke<string[]>('get_search_suggestions', {
          partial_query: query,
          limit: 5
        })
        setSuggestions(suggs)
      } catch (error) {
        console.error('Erro ao buscar sugestões:', error)
      }
    }

    const debounceTimer = setTimeout(fetchSuggestions, 300)
    return () => clearTimeout(debounceTimer)
  }, [query])

  const performSearch = useCallback(async (searchQuery: string, type: SearchType) => {
    if (!searchQuery.trim()) return

    setIsSearching(true)
    setError(null)
    setResults([])
    setSearchTime(null)

    try {
      let response: SearchResponse | HybridSearchResponse

      switch (type) {
        case 'text':
          response = await invoke<SearchResponse>('search_text', {
            query: searchQuery,
            limit: resultLimit
          })
          setResults(response.results)
          break

        case 'semantic':
          response = await invoke<HybridSearchResponse>('search_semantic', {
            query: searchQuery,
            limit: resultLimit
          })
          setResults(response.results)
          break

        case 'hybrid':
          response = await invoke<HybridSearchResponse>('search_hybrid', {
            query: searchQuery,
            limit: resultLimit,
            text_weight: textWeight,
            semantic_weight: semanticWeight
          })
          setResults(response.results)
          break
      }

      setSearchTime(response.search_time_ms)
      
      // Adicionar à história de busca
      setSearchHistory(prev => {
        const newHistory = [searchQuery, ...prev.filter(q => q !== searchQuery)]
        return newHistory.slice(0, 10) // Manter apenas as últimas 10 buscas
      })
    } catch (error) {
      console.error('Erro na busca:', error)
      setError(error as string)
    } finally {
      setIsSearching(false)
      setShowSuggestions(false)
    }
  }, [resultLimit, textWeight, semanticWeight])

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault()
    performSearch(query, searchType)
  }

  const handleSuggestionClick = (suggestion: string) => {
    setQuery(suggestion)
    performSearch(suggestion, searchType)
  }

  const handleHistoryClick = (historicalQuery: string) => {
    setQuery(historicalQuery)
    performSearch(historicalQuery, searchType)
  }

  const clearSearch = () => {
    setQuery('')
    setResults([])
    setSearchTime(null)
    setError(null)
    setSuggestions([])
  }

  return (
    <div className="space-y-6">
      {/* Formulário de Busca */}
      <div className="card p-6">
        <form onSubmit={handleSearch} className="space-y-4">
          {/* Campo de Busca */}
          <div className="relative">
            <div className="relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-slate-400 w-5 h-5" />
              <input
                type="text"
                value={query}
                onChange={(e) => {
                  setQuery(e.target.value)
                  setShowSuggestions(true)
                }}
                onFocus={() => setShowSuggestions(true)}
                onBlur={() => setTimeout(() => setShowSuggestions(false), 200)}
                placeholder="Digite sua busca..."
                className="input pl-10 pr-4 text-lg"
                disabled={isSearching}
              />
              {query && (
                <button
                  type="button"
                  onClick={clearSearch}
                  className="absolute right-3 top-1/2 transform -translate-y-1/2 text-slate-400 hover:text-slate-200"
                >
                  ✕
                </button>
              )}
            </div>

            {/* Sugestões */}
            {showSuggestions && suggestions.length > 0 && (
              <div className="absolute top-full left-0 right-0 mt-1 bg-slate-800 border border-slate-700 rounded-lg shadow-lg z-10">
                {suggestions.map((suggestion, index) => (
                  <button
                    key={index}
                    type="button"
                    onClick={() => handleSuggestionClick(suggestion)}
                    className="block w-full text-left px-4 py-2 hover:bg-slate-700 transition-colors"
                  >
                    <Search className="inline-block w-4 h-4 mr-2 text-slate-400" />
                    {suggestion}
                  </button>
                ))}
              </div>
            )}
          </div>

          {/* Controles de Busca */}
          <div className="flex items-center space-x-4">
            {/* Tipo de Busca */}
            <div className="flex items-center space-x-2">
              <label className="text-sm text-slate-400">Tipo:</label>
              <div className="flex space-x-1">
                <button
                  type="button"
                  onClick={() => setSearchType('text')}
                  className={`px-3 py-1 rounded-lg text-sm transition-colors ${
                    searchType === 'text'
                      ? 'bg-primary-600 text-white'
                      : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
                  }`}
                  title="Busca textual (FTS5)"
                >
                  <FileText className="inline-block w-4 h-4 mr-1" />
                  Textual
                </button>
                <button
                  type="button"
                  onClick={() => setSearchType('semantic')}
                  className={`px-3 py-1 rounded-lg text-sm transition-colors ${
                    searchType === 'semantic'
                      ? 'bg-primary-600 text-white'
                      : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
                  }`}
                  title="Busca semântica (embeddings)"
                >
                  <Brain className="inline-block w-4 h-4 mr-1" />
                  Semântica
                </button>
                <button
                  type="button"
                  onClick={() => setSearchType('hybrid')}
                  className={`px-3 py-1 rounded-lg text-sm transition-colors ${
                    searchType === 'hybrid'
                      ? 'bg-primary-600 text-white'
                      : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
                  }`}
                  title="Busca híbrida (textual + semântica)"
                >
                  <Zap className="inline-block w-4 h-4 mr-1" />
                  Híbrida
                </button>
              </div>
            </div>

            {/* Configurações de Busca Híbrida */}
            {searchType === 'hybrid' && (
              <div className="flex items-center space-x-4 text-sm">
                <div className="flex items-center space-x-2">
                  <label className="text-slate-400">Peso Textual:</label>
                  <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.1"
                    value={textWeight}
                    onChange={(e) => {
                      const value = parseFloat(e.target.value)
                      setTextWeight(value)
                      setSemanticWeight(1 - value)
                    }}
                    className="w-20"
                  />
                  <span className="text-slate-300">{(textWeight * 100).toFixed(0)}%</span>
                </div>
                <div className="flex items-center space-x-2">
                  <label className="text-slate-400">Peso Semântico:</label>
                  <span className="text-slate-300">{(semanticWeight * 100).toFixed(0)}%</span>
                </div>
              </div>
            )}

            {/* Limite de Resultados */}
            <div className="flex items-center space-x-2 text-sm">
              <label className="text-slate-400">Limite:</label>
              <select
                value={resultLimit}
                onChange={(e) => setResultLimit(parseInt(e.target.value))}
                className="bg-slate-700 text-slate-300 rounded px-2 py-1"
              >
                <option value="10">10</option>
                <option value="25">25</option>
                <option value="50">50</option>
                <option value="100">100</option>
              </select>
            </div>

            {/* Botão de Busca */}
            <button
              type="submit"
              disabled={isSearching || !query.trim()}
              className="btn-primary px-6 py-2 ml-auto"
            >
              {isSearching ? (
                <>
                  <div className="loading-spinner w-4 h-4 mr-2"></div>
                  Buscando...
                </>
              ) : (
                <>
                  <Search className="w-4 h-4 mr-2" />
                  Buscar
                </>
              )}
            </button>
          </div>
        </form>

        {/* Estatísticas da Busca */}
        {searchTime !== null && (
          <div className="mt-4 pt-4 border-t border-slate-700 flex items-center justify-between text-sm">
            <div className="flex items-center space-x-4">
              <span className="text-slate-400">
                {results.length} resultado{results.length !== 1 ? 's' : ''} encontrado{results.length !== 1 ? 's' : ''}
              </span>
              <span className="text-slate-400">
                <Clock className="inline-block w-4 h-4 mr-1" />
                {searchTime}ms
              </span>
            </div>
            {searchType === 'hybrid' && (
              <span className="text-slate-400">
                Busca híbrida: {(textWeight * 100).toFixed(0)}% textual + {(semanticWeight * 100).toFixed(0)}% semântica
              </span>
            )}
          </div>
        )}

        {/* Erro */}
        {error && (
          <div className="mt-4 p-4 bg-red-900/20 border border-red-700 rounded-lg text-red-400">
            {error}
          </div>
        )}
      </div>

      {/* Histórico de Busca */}
      {searchHistory.length > 0 && !isSearching && results.length === 0 && (
        <SearchHistory 
          history={searchHistory} 
          onHistoryClick={handleHistoryClick}
        />
      )}

      {/* Resultados da Busca */}
      {results.length > 0 && (
        <SearchResults 
          results={results} 
          searchType={searchType}
        />
      )}
    </div>
  )
} 