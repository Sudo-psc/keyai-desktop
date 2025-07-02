import '@testing-library/jest-dom';

// Mock do Tauri API para testes
global.window.__TAURI__ = {
  tauri: {
    invoke: async (cmd: string, args?: any) => {
      console.log(`Mock invoke: ${cmd}`, args);
      // Retornar dados mockados baseados no comando
      switch (cmd) {
        case 'search':
          return {
            results: [],
            total: 0,
            execution_time_ms: 0,
          };
        case 'get_status':
          return {
            is_recording: false,
            total_words: 0,
            last_capture: null,
          };
        default:
          return null;
      }
    },
  },
  event: {
    emit: async (event: string, payload?: any) => {
      console.log(`Mock emit: ${event}`, payload);
    },
    listen: async (event: string, handler: (event: any) => void) => {
      console.log(`Mock listen: ${event}`);
      return () => {};
    },
  },
} as any;