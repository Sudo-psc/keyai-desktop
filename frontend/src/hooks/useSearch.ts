import { useState, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import { SearchType, SearchResult, HybridSearchResult } from '../types'

interface UseSearchOptions {
  defaultType?: SearchType
  defaultLimit?: number
  defaultTextWeight?: number
  defaultSemanticWeight?: number
}

export function useSearch(options?: UseSearchOptions) {
  const [query, setQuery] = useState('')
  const [searchType, setSearchType] = useState<SearchType>(options?.defaultType || 'hybrid')
  const [results, setResults] = useState<(SearchResult | HybridSearchResult)[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [searchTime, setSearchTime] = useState<number | null>(null)
  const [suggestions, setSuggestions] = useState<string[]>([])
  const [textWeight, setTextWeight] = useState(options?.defaultTextWeight ?? 0.7)
  const [semanticWeight, setSemanticWeight] = useState(options?.defaultSemanticWeight ?? 0.3)
  const [limit, setLimit] = useState(options?.defaultLimit ?? 50)

  // Executa a busca conforme o tipo
  const search = useCallback(async (q?: string, type?: SearchType) => {
    const searchQuery = typeof q === 'string' ? q : query
    const searchMode = type || searchType
    setIsLoading(true)
    setError(null)
    setResults([])
    setSearchTime(null)
    try {
      let response: any
      if (searchMode === 'text') {
        response = await invoke('search_text', { query: searchQuery, limit })
        setResults(response.results)
      } else if (searchMode === 'semantic') {
        response = await invoke('search_semantic', { query: searchQuery, limit })
        setResults(response.results)
      } else {
        response = await invoke('search_hybrid', {
          query: searchQuery,
          limit,
          text_weight: textWeight,
          semantic_weight: semanticWeight,
        })
        setResults(response.results)
      }
      setSearchTime(response.search_time_ms)
    } catch (err: any) {
      setError(err?.message || String(err))
    } finally {
      setIsLoading(false)
    }
  }, [query, searchType, limit, textWeight, semanticWeight])

  // Busca sugestões
  const fetchSuggestions = useCallback(async (partial: string) => {
    if (!partial || partial.length < 2) {
      setSuggestions([])
      return
    }
    try {
      const suggs = await invoke<string[]>('get_search_suggestions', { partial_query: partial, limit: 5 })
      setSuggestions(suggs)
    } catch (err) {
      setSuggestions([])
    }
  }, [])

  return {
    query,
    setQuery,
    searchType,
    setSearchType,
    results,
    isLoading,
    error,
    searchTime,
    suggestions,
    fetchSuggestions,
    search,
    textWeight,
    setTextWeight,
    semanticWeight,
    setSemanticWeight,
    limit,
    setLimit,
  }
}

export default useSearch 