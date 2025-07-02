import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import {
  Database,
  HardDrive,
  Activity,
  Calendar,
  Trash2,
  AlertCircle,
  TrendingUp,
  Clock,
  Settings
} from 'lucide-react'
import { AppStats } from '../types'
import { format, formatDistanceToNow } from 'date-fns'
import { ptBR } from 'date-fns/locale'

interface StatusPanelProps {
  stats: AppStats | null
  onRefresh: () => void
}

export default function StatusPanel({ stats, onRefresh }: StatusPanelProps) {
  const [isClearing, setIsClearing] = useState(false)
  const [showClearConfirm, setShowClearConfirm] = useState(false)
  const [popularSearches, setPopularSearches] = useState<string[]>([])

  useEffect(() => {
    fetchPopularSearches()
  }, [])

  const fetchPopularSearches = async () => {
    try {
      const searches = await invoke<string[]>('get_popular_searches', { limit: 5 })
      setPopularSearches(searches)
    } catch (error) {
      console.error('Erro ao buscar buscas populares:', error)
    }
  }

  const handleClearData = async () => {
    if (!showClearConfirm) {
      setShowClearConfirm(true)
      return
    }

    setIsClearing(true)
    try {
      await invoke('clear_data', { confirm: true })
      await onRefresh()
      setShowClearConfirm(false)
    } catch (error) {
      console.error('Erro ao limpar dados:', error)
    } finally {
      setIsClearing(false)
    }
  }

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 Bytes'
    const k = 1024
    const sizes = ['Bytes', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
  }

  const formatTimestamp = (timestamp?: number): string => {
    if (!timestamp) return 'N/A'
    const date = new Date(timestamp * 1000)
    return format(date, 'dd/MM/yyyy HH:mm', { locale: ptBR })
  }

  const formatRelativeTime = (timestamp?: number): string => {
    if (!timestamp) return 'N/A'
    const date = new Date(timestamp * 1000)
    return formatDistanceToNow(date, { addSuffix: true, locale: ptBR })
  }

  if (!stats) {
    return (
      <div className="card p-6">
        <div className="text-center text-slate-400">
          <AlertCircle className="w-12 h-12 mx-auto mb-2" />
          <p>Estatísticas não disponíveis</p>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Status do Sistema */}
      <div className="card p-6">
        <h3 className="text-lg font-semibold mb-4 flex items-center">
          <Activity className="w-5 h-5 mr-2 text-primary-500" />
          Status do Sistema
        </h3>

        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <span className="text-sm text-slate-400">Agente de Captura</span>
            <span className={`text-sm font-medium ${
              stats.agent.is_running ? 'text-green-400' : 'text-red-400'
            }`}>
              {stats.agent.is_running ? 'Ativo' : 'Inativo'}
            </span>
          </div>

          {stats.agent.is_running && (
            <>
              <div className="flex items-center justify-between">
                <span className="text-sm text-slate-400">Tempo Ativo</span>
                <span className="text-sm font-medium">
                  {Math.floor(stats.agent.uptime_seconds / 3600)}h {Math.floor((stats.agent.uptime_seconds % 3600) / 60)}m
                </span>
              </div>

              <div className="flex items-center justify-between">
                <span className="text-sm text-slate-400">Eventos Capturados</span>
                <span className="text-sm font-medium">
                  {stats.agent.events_captured.toLocaleString('pt-BR')}
                </span>
              </div>
            </>
          )}
        </div>
      </div>

      {/* Estatísticas do Banco de Dados */}
      <div className="card p-6">
        <h3 className="text-lg font-semibold mb-4 flex items-center">
          <Database className="w-5 h-5 mr-2 text-primary-500" />
          Banco de Dados
        </h3>

        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <span className="text-sm text-slate-400">Total de Eventos</span>
            <span className="text-sm font-medium">
              {stats.database.total_events.toLocaleString('pt-BR')}
            </span>
          </div>

          <div className="flex items-center justify-between">
            <span className="text-sm text-slate-400">Tamanho</span>
            <span className="text-sm font-medium">
              {formatBytes(stats.database.total_size_bytes)}
            </span>
          </div>

          {stats.database.oldest_event && (
            <div className="pt-3 border-t border-slate-700">
              <div className="flex items-center mb-2">
                <Calendar className="w-4 h-4 mr-2 text-slate-400" />
                <span className="text-sm text-slate-400">Período de Dados</span>
              </div>

              <div className="text-xs space-y-1">
                <div>
                  <span className="text-slate-500">Mais antigo: </span>
                  <span className="text-slate-300">
                    {formatRelativeTime(stats.database.oldest_event)}
                  </span>
                </div>
                <div>
                  <span className="text-slate-500">Mais recente: </span>
                  <span className="text-slate-300">
                    {formatRelativeTime(stats.database.newest_event)}
                  </span>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Buscas Populares */}
      {popularSearches.length > 0 && (
        <div className="card p-6">
          <h3 className="text-lg font-semibold mb-4 flex items-center">
            <TrendingUp className="w-5 h-5 mr-2 text-primary-500" />
            Buscas Populares
          </h3>

          <div className="space-y-2">
            {popularSearches.map((search, index) => (
              <div key={index} className="flex items-center text-sm">
                <span className="text-slate-500 mr-2">{index + 1}.</span>
                <span className="text-slate-300">{search}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Ações */}
      <div className="card p-6">
        <h3 className="text-lg font-semibold mb-4 flex items-center">
          <Settings className="w-5 h-5 mr-2 text-primary-500" />
          Ações
        </h3>

        <div className="space-y-3">
          <button
            onClick={async () => {
              try {
                await invoke('optimize_search_index')
                await onRefresh()
              } catch (error) {
                console.error('Erro ao otimizar índices:', error)
              }
            }}
            className="w-full btn-secondary py-2 text-sm"
          >
            <HardDrive className="w-4 h-4 mr-2" />
            Otimizar Índices
          </button>

          {showClearConfirm ? (
            <div className="space-y-2">
              <p className="text-sm text-red-400 text-center">
                Tem certeza? Esta ação não pode ser desfeita!
              </p>
              <div className="flex space-x-2">
                <button
                  onClick={handleClearData}
                  disabled={isClearing}
                  className="flex-1 btn bg-red-600 hover:bg-red-700 text-white py-2 text-sm"
                >
                  {isClearing ? 'Limpando...' : 'Confirmar'}
                </button>
                <button
                  onClick={() => setShowClearConfirm(false)}
                  className="flex-1 btn-secondary py-2 text-sm"
                >
                  Cancelar
                </button>
              </div>
            </div>
          ) : (
            <button
              onClick={handleClearData}
              className="w-full btn bg-red-600 hover:bg-red-700 text-white py-2 text-sm"
            >
              <Trash2 className="w-4 h-4 mr-2" />
              Limpar Todos os Dados
            </button>
          )}
        </div>
      </div>

      {/* Última Atualização */}
      <div className="text-center text-xs text-slate-500">
        <Clock className="inline-block w-3 h-3 mr-1" />
        Atualizado {formatDistanceToNow(new Date(), { addSuffix: true, locale: ptBR })}
      </div>
    </div>
  )
}
