import { useState, useEffect, useCallback } from 'react';

interface GlassTheme {
  isDark: boolean;
  colors: {
    background: string;
    surface: string;
    primary: string;
    secondary: string;
    accent: string;
    success: string;
    text: {
      primary: string;
      secondary: string;
      accent: string;
      success: string;
    };
  };
  opacity: {
    low: number;
    medium: number;
    high: number;
  };
  blur: {
    sm: string;
    md: string;
    lg: string;
    xl: string;
  };
}

const lightTheme: GlassTheme = {
  isDark: false,
  colors: {
    background: '#FFFFFF',
    surface: '#F5F5F7',
    primary: '#007AFF',
    secondary: '#8E8E93',
    accent: '#007AFF',
    success: '#00C781',
    text: {
      primary: '#000000',
      secondary: '#8E8E93',
      accent: '#007AFF',
      success: '#00C781',
    },
  },
  opacity: {
    low: 0.03,
    medium: 0.08,
    high: 0.15,
  },
  blur: {
    sm: '10px',
    md: '20px',
    lg: '30px',
    xl: '40px',
  },
};

const darkTheme: GlassTheme = {
  isDark: true,
  colors: {
    background: '#000000',
    surface: '#1C1C1E',
    primary: '#007AFF',
    secondary: '#8E8E93',
    accent: '#007AFF',
    success: '#00C781',
    text: {
      primary: '#FFFFFF',
      secondary: '#8E8E93',
      accent: '#007AFF',
      success: '#00C781',
    },
  },
  opacity: {
    low: 0.05,
    medium: 0.15,
    high: 0.25,
  },
  blur: {
    sm: '10px',
    md: '20px',
    lg: '30px',
    xl: '40px',
  },
};

export const useGlassTheme = () => {
  const [isDark, setIsDark] = useState(true); // Default to dark theme

  const theme = isDark ? darkTheme : lightTheme;

  // Load theme preference from localStorage
  useEffect(() => {
    const savedTheme = localStorage.getItem('keyai-theme');
    if (savedTheme) {
      setIsDark(savedTheme === 'dark');
    } else {
      // Check system preference
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      setIsDark(prefersDark);
    }
  }, []);

  // Save theme preference and apply to document
  useEffect(() => {
    localStorage.setItem('keyai-theme', isDark ? 'dark' : 'light');
    document.documentElement.classList.toggle('dark', isDark);
    document.documentElement.style.colorScheme = isDark ? 'dark' : 'light';
  }, [isDark]);

  const toggleTheme = useCallback(() => {
    setIsDark(prev => !prev);
  }, []);

  const setTheme = useCallback((theme: 'light' | 'dark') => {
    setIsDark(theme === 'dark');
  }, []);

  // CSS custom properties for dynamic theming
  const cssVariables = {
    '--color-background': theme.colors.background,
    '--color-surface': theme.colors.surface,
    '--color-primary': theme.colors.primary,
    '--color-secondary': theme.colors.secondary,
    '--color-accent': theme.colors.accent,
    '--color-success': theme.colors.success,
    '--color-text-primary': theme.colors.text.primary,
    '--color-text-secondary': theme.colors.text.secondary,
    '--color-text-accent': theme.colors.text.accent,
    '--color-text-success': theme.colors.text.success,
    '--glass-opacity-low': theme.opacity.low,
    '--glass-opacity-medium': theme.opacity.medium,
    '--glass-opacity-high': theme.opacity.high,
    '--backdrop-blur-sm': theme.blur.sm,
    '--backdrop-blur-md': theme.blur.md,
    '--backdrop-blur-lg': theme.blur.lg,
    '--backdrop-blur-xl': theme.blur.xl,
  };

  // Apply CSS variables to document
  useEffect(() => {
    const root = document.documentElement;
    Object.entries(cssVariables).forEach(([property, value]) => {
      root.style.setProperty(property, String(value));
    });
  }, [theme]);

  // Utility functions for glass effects
  const getGlassStyle = (variant: 'low' | 'medium' | 'high' = 'medium') => ({
    backgroundColor: isDark
      ? `rgba(28, 28, 30, ${theme.opacity[variant]})`
      : `rgba(255, 255, 255, ${theme.opacity[variant]})`,
    backdropFilter: `blur(${theme.blur.md})`,
    WebkitBackdropFilter: `blur(${theme.blur.md})`,
    border: isDark
      ? '1px solid rgba(255, 255, 255, 0.1)'
      : '1px solid rgba(0, 0, 0, 0.1)',
  });

  const getShadowStyle = (variant: 'soft' | 'glow' = 'soft') => {
    if (variant === 'glow') {
      return {
        boxShadow: isDark
          ? '0 8px 32px rgba(0, 0, 0, 0.4), 0 0 40px rgba(0, 122, 255, 0.3)'
          : '0 8px 32px rgba(0, 0, 0, 0.1), 0 0 40px rgba(0, 122, 255, 0.2)',
      };
    }

    return {
      boxShadow: isDark
        ? '0 8px 32px rgba(0, 0, 0, 0.4)'
        : '0 8px 32px rgba(0, 0, 0, 0.1)',
    };
  };

  return {
    theme,
    isDark,
    toggleTheme,
    setTheme,
    cssVariables,
    getGlassStyle,
    getShadowStyle,
  };
};
