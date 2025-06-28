import { Clock, Search } from 'lucide-react'

interface SearchHistoryProps {
  history: string[]
  onHistoryClick: (query: string) => void
}

export default function SearchHistory({ history, onHistoryClick }: SearchHistoryProps) {
  if (history.length === 0) {
    return null
  }

  return (
    <div className="card p-6">
      <h3 className="text-lg font-semibold mb-4 flex items-center">
        <Clock className="w-5 h-5 mr-2 text-primary-500" />
        Histórico de Busca
      </h3>
      
      <div className="space-y-2">
        {history.map((query, index) => (
          <button
            key={index}
            onClick={() => onHistoryClick(query)}
            className="w-full text-left p-3 rounded-lg bg-slate-800 hover:bg-slate-700 transition-colors group"
          >
            <div className="flex items-center justify-between">
              <div className="flex items-center space-x-3">
                <Search className="w-4 h-4 text-slate-400 group-hover:text-primary-400 transition-colors" />
                <span className="text-sm text-slate-200 group-hover:text-white transition-colors">
                  {query}
                </span>
              </div>
              <span className="text-xs text-slate-500 group-hover:text-slate-400 transition-colors">
                Buscar novamente
              </span>
            </div>
          </button>
        ))}
      </div>
      
      <div className="mt-4 pt-4 border-t border-slate-700">
        <p className="text-xs text-slate-500 text-center">
          Mostrando as últimas {history.length} buscas
        </p>
      </div>
    </div>
  )
} 