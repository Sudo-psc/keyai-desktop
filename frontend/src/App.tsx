import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import Header from './components/Header';
import SearchInterface from './components/SearchInterface';
import SearchResults from './components/SearchResults';
import StatusPanel from './components/StatusPanel';
import { useSearch } from './hooks/useSearch';
import { useGlassTheme } from './hooks/useGlassTheme';
import { AppStats } from './types';
import './styles/liquid-glass.css';

function App() {
  const { results, isLoading, error, searchType } = useSearch();
  const { theme } = useGlassTheme();
  const [stats, setStats] = useState<AppStats | null>(null);

  const fetchStats = async () => {
    try {
      const appStats = await invoke<AppStats>('get_stats');
      setStats(appStats);
    } catch (err) {
      console.error('Erro ao obter estatísticas:', err);
    }
  };

  useEffect(() => {
    fetchStats();
    const interval = setInterval(fetchStats, 30000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="min-h-screen bg-black text-white">
      {/* Background Gradient */}
      <div className="fixed inset-0 bg-gradient-to-br from-black via-charcoal to-black pointer-events-none" />

      {/* Main Layout */}
      <div className="relative z-10 flex h-screen">
        {/* Sidebar */}
        <aside className="w-64 sidebar-glass flex-shrink-0">
          <div className="p-6">
            <div className="mb-8">
              <h1 className="text-2xl font-bold gradient-text">key.ai</h1>
              <p className="text-caption text-secondary mt-1">Desktop Search</p>
            </div>

            <nav className="space-y-2">
              <a href="#" className="sidebar-nav-item active">
                <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                </svg>
                <span>Buscar</span>
              </a>
              <a href="#" className="sidebar-nav-item">
                <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <span>Histórico</span>
              </a>
              <a href="#" className="sidebar-nav-item">
                <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                </svg>
                <span>Configurações</span>
              </a>
            </nav>
          </div>
        </aside>

        {/* Main Content */}
        <main className="flex-1 flex flex-col overflow-hidden">
          {/* Header */}
          <Header stats={stats} onRefresh={fetchStats} />

          {/* Content Area */}
          <div className="flex-1 overflow-auto">
            <div className="p-8 space-y-8">
              {/* Search Interface */}
              <SearchInterface />

              {/* Results */}
              {results.length > 0 && (
                <div className="fade-in">
                  <SearchResults results={results} searchType={searchType} />
                </div>
              )}

              {/* Error State */}
              {error && (
                <div className="floating-card border-red-500/20 bg-red-500/5">
                  <div className="flex items-center gap-3">
                    <div className="p-2 rounded-lg bg-red-500/20">
                      <svg className="w-5 h-5 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z" />
                      </svg>
                    </div>
                    <div>
                      <h3 className="text-heading text-red-400 font-medium">Erro na busca</h3>
                      <p className="text-caption text-red-300 mt-1">{error}</p>
                    </div>
                  </div>
                </div>
              )}
            </div>
          </div>

          {/* Status Panel */}
          <StatusPanel stats={stats} onRefresh={fetchStats} />
        </main>
      </div>
    </div>
  );
}

export default App;
