import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import SearchInterface from './components/SearchInterface'
import Header from './components/Header'
import StatusPanel from './components/StatusPanel'
import { AppStats } from './types'

function App() {
  const [stats, setStats] = useState<AppStats | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const fetchStats = async () => {
    try {
      const appStats = await invoke<AppStats>('get_stats')
      setStats(appStats)
      setError(null)
    } catch (err) {
      console.error('Erro ao obter estatísticas:', err)
      setError('Erro ao carregar estatísticas da aplicação')
    } finally {
      setIsLoading(false)
    }
  }

  useEffect(() => {
    fetchStats()
    
    // Atualizar estatísticas a cada 30 segundos
    const interval = setInterval(fetchStats, 30000)
    
    return () => clearInterval(interval)
  }, [])

  if (isLoading) {
    return (
      <div className="min-h-screen bg-slate-900 flex items-center justify-center">
        <div className="text-center">
          <div className="loading-spinner w-12 h-12 mx-auto mb-4"></div>
          <p className="text-slate-400">Carregando KeyAI Desktop...</p>
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="min-h-screen bg-slate-900 flex items-center justify-center">
        <div className="text-center">
          <div className="text-red-500 text-6xl mb-4">⚠️</div>
          <h1 className="text-xl font-semibold text-white mb-2">Erro na Aplicação</h1>
          <p className="text-slate-400 mb-4">{error}</p>
          <button 
            onClick={fetchStats}
            className="btn-primary px-4 py-2"
          >
            Tentar Novamente
          </button>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-slate-900">
      <Header stats={stats} onRefresh={fetchStats} />
      
      <main className="container mx-auto px-4 py-6">
        <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
          {/* Interface de Busca - Coluna Principal */}
          <div className="lg:col-span-3">
            <SearchInterface />
          </div>
          
          {/* Painel de Status - Sidebar */}
          <div className="lg:col-span-1">
            <StatusPanel stats={stats} onRefresh={fetchStats} />
          </div>
        </div>
      </main>
    </div>
  )
}

export default App 