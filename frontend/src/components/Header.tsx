import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import { Shield, Activity, RefreshCw, Power, Settings } from 'lucide-react'
import { AppStats, AgentStatus } from '../types'

interface HeaderProps {
  stats: AppStats | null
  onRefresh: () => void
}

export default function Header({ stats, onRefresh }: HeaderProps) {
  const [isRefreshing, setIsRefreshing] = useState(false)
  const [agentStatus, setAgentStatus] = useState<AgentStatus | null>(stats?.agent || null)
  const [isTogglingAgent, setIsTogglingAgent] = useState(false)

  useEffect(() => {
    if (stats?.agent) {
      setAgentStatus(stats.agent)
    }
  }, [stats])

  const handleRefresh = async () => {
    setIsRefreshing(true)
    await onRefresh()
    setTimeout(() => setIsRefreshing(false), 500)
  }

  const toggleAgent = async () => {
    if (!agentStatus || isTogglingAgent) return
    
    setIsTogglingAgent(true)
    try {
      const newStatus = await invoke<AgentStatus>('toggle_agent', {
        enable: !agentStatus.is_running
      })
      setAgentStatus(newStatus)
    } catch (error) {
      console.error('Erro ao alternar agente:', error)
    } finally {
      setIsTogglingAgent(false)
    }
  }

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 Bytes'
    const k = 1024
    const sizes = ['Bytes', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
  }

  return (
    <header className="bg-slate-800 border-b border-slate-700 shadow-lg">
      <div className="container mx-auto px-4 py-4">
        <div className="flex items-center justify-between">
          {/* Logo e Título */}
          <div className="flex items-center space-x-3">
            <div className="flex items-center justify-center w-10 h-10 bg-primary-600 rounded-lg">
              <Shield className="w-6 h-6 text-white" />
            </div>
            <div>
              <h1 className="text-xl font-bold text-white">KeyAI Desktop</h1>
              <p className="text-xs text-slate-400">Sistema de Registro e Busca com Privacidade</p>
            </div>
          </div>

          {/* Estatísticas Rápidas */}
          <div className="flex items-center space-x-6">
            {stats && (
              <>
                <div className="text-center">
                  <p className="text-xs text-slate-400">Total de Eventos</p>
                  <p className="text-lg font-semibold text-white">
                    {stats.database.total_events.toLocaleString('pt-BR')}
                  </p>
                </div>
                <div className="text-center">
                  <p className="text-xs text-slate-400">Tamanho do Banco</p>
                  <p className="text-lg font-semibold text-white">
                    {formatBytes(stats.database.total_size_bytes)}
                  </p>
                </div>
              </>
            )}
          </div>

          {/* Controles */}
          <div className="flex items-center space-x-3">
            {/* Botão de Status do Agente */}
            <button
              onClick={toggleAgent}
              disabled={isTogglingAgent}
              className={`flex items-center space-x-2 px-4 py-2 rounded-lg transition-all ${
                agentStatus?.is_running
                  ? 'bg-green-600 hover:bg-green-700 text-white'
                  : 'bg-slate-600 hover:bg-slate-700 text-white'
              } ${isTogglingAgent ? 'opacity-50 cursor-not-allowed' : ''}`}
            >
              {agentStatus?.is_running ? (
                <>
                  <Activity className="w-4 h-4" />
                  <span className="text-sm font-medium">Capturando</span>
                </>
              ) : (
                <>
                  <Power className="w-4 h-4" />
                  <span className="text-sm font-medium">Parado</span>
                </>
              )}
            </button>

            {/* Botão de Atualizar */}
            <button
              onClick={handleRefresh}
              disabled={isRefreshing}
              className="p-2 rounded-lg bg-slate-700 hover:bg-slate-600 text-white transition-colors"
              title="Atualizar estatísticas"
            >
              <RefreshCw className={`w-4 h-4 ${isRefreshing ? 'animate-spin' : ''}`} />
            </button>

            {/* Botão de Configurações */}
            <button
              className="p-2 rounded-lg bg-slate-700 hover:bg-slate-600 text-white transition-colors"
              title="Configurações"
            >
              <Settings className="w-4 h-4" />
            </button>
          </div>
        </div>
      </div>
    </header>
  )
} 