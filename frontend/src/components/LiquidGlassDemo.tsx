import React from 'react';
import { Search, History, Settings, BarChart3, Clock, Shield } from 'lucide-react';
import '../styles/liquid-glass.css';

const LiquidGlassDemo: React.FC = () => {
  return (
    <div className="min-h-screen bg-black flex">
      {/* Sidebar */}
      <aside className="w-64 sidebar-glass p-6">
        <div className="mb-8">
          <h1 className="text-2xl font-bold gradient-text">key.ai</h1>
          <p className="text-gray-light text-sm mt-1">Desktop Search</p>
        </div>
        
        <nav className="space-y-2">
          <a href="#" className="flex items-center gap-3 p-3 rounded-lg hover:bg-white/5 transition-colors">
            <Search className="w-5 h-5 text-electric-blue" />
            <span className="text-white">Buscar</span>
          </a>
          <a href="#" className="flex items-center gap-3 p-3 rounded-lg hover:bg-white/5 transition-colors">
            <History className="w-5 h-5 text-mint-green" />
            <span className="text-white">Histórico</span>
          </a>
          <a href="#" className="flex items-center gap-3 p-3 rounded-lg hover:bg-white/5 transition-colors">
            <Settings className="w-5 h-5 text-gray-light" />
            <span className="text-white">Configurações</span>
          </a>
        </nav>
      </aside>

      {/* Main Content */}
      <main className="flex-1 p-8">
        {/* Search Card */}
        <div className="floating-card max-w-2xl mx-auto mb-6">
          <div className="flex items-center gap-4 mb-4">
            <Search className="w-6 h-6 text-electric-blue" />
            <h2 className="text-xl font-semibold text-white">Busca Inteligente</h2>
          </div>
          
          <input
            type="text"
            placeholder="Digite para buscar em seu histórico..."
            className="glass-input w-full mb-4"
          />
          
          <div className="flex gap-3">
            <button className="glass-button">
              Busca Textual
            </button>
            <button className="glass-button">
              Busca Semântica
            </button>
          </div>
        </div>

        {/* Stats Cards */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 max-w-4xl mx-auto">
          <div className="floating-card">
            <div className="flex items-center justify-between mb-3">
              <BarChart3 className="w-8 h-8 text-electric-blue" />
              <span className="text-2xl font-bold text-white">1.2M</span>
            </div>
            <p className="text-gray-light">Palavras Registradas</p>
          </div>
          
          <div className="floating-card">
            <div className="flex items-center justify-between mb-3">
              <Clock className="w-8 h-8 text-mint-green" />
              <span className="text-2xl font-bold text-white">24h</span>
            </div>
            <p className="text-gray-light">Histórico Ativo</p>
          </div>
          
          <div className="floating-card">
            <div className="flex items-center justify-between mb-3">
              <Shield className="w-8 h-8 text-electric-blue" />
              <span className="text-2xl font-bold text-white">100%</span>
            </div>
            <p className="text-gray-light">Dados Privados</p>
          </div>
        </div>

        {/* Chart Section */}
        <div className="glass-chart max-w-4xl mx-auto mt-8">
          <h3 className="text-lg font-semibold text-white mb-4">Atividade de Digitação</h3>
          <div className="h-48 flex items-end justify-between gap-2">
            {[40, 65, 45, 80, 55, 90, 70, 85, 60, 75].map((height, i) => (
              <div
                key={i}
                className="flex-1 bg-gradient-to-t from-electric-blue to-mint-green rounded-t"
                style={{ height: `${height}%`, opacity: 0.8 }}
              />
            ))}
          </div>
        </div>
      </main>
    </div>
  );
};

export default LiquidGlassDemo; 