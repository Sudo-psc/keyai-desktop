export interface SearchResult {
  id: number
  content: string
  timestamp: number
  relevance_score: number
  context?: string
}

export interface HybridSearchResult {
  id: number
  content: string
  timestamp: number
  text_score: number
  semantic_score: number
  combined_score: number
  context?: string
}

export interface SearchResponse {
  results: SearchResult[]
  total_count: number
  search_time_ms: number
}

export interface HybridSearchResponse {
  results: HybridSearchResult[]
  total_count: number
  search_time_ms: number
}

export interface AgentStatus {
  is_running: boolean
  uptime_seconds: number
  events_captured: number
}

export interface DatabaseStats {
  total_events: number
  total_size_bytes: number
  oldest_event?: number
  newest_event?: number
}

export interface AppStats {
  database: DatabaseStats
  agent: AgentStatus
}

export interface SearchOptions {
  limit?: number
  text_weight?: number
  semantic_weight?: number
}

export type SearchType = 'text' | 'semantic' | 'hybrid'

export interface SearchHistoryItem {
  query: string
  type: SearchType
  timestamp: number
  results_count: number
} 