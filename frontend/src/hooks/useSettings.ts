import { useState, useEffect } from 'react'

export interface AppSettings {
  textWeight: number
  semanticWeight: number
  resultLimit: number
  darkMode: boolean
}

const SETTINGS_KEY = 'keyai_settings_v1'

const defaultSettings: AppSettings = {
  textWeight: 0.7,
  semanticWeight: 0.3,
  resultLimit: 50,
  darkMode: true,
}

export function useSettings() {
  const [settings, setSettings] = useState<AppSettings>(() => {
    try {
      const saved = localStorage.getItem(SETTINGS_KEY)
      if (saved) return { ...defaultSettings, ...JSON.parse(saved) }
    } catch {}
    return defaultSettings
  })

  useEffect(() => {
    try {
      localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings))
    } catch {}
  }, [settings])

  const updateSetting = <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => {
    setSettings((prev) => ({ ...prev, [key]: value }))
  }

  const resetSettings = () => setSettings(defaultSettings)

  return {
    settings,
    setSettings,
    updateSetting,
    resetSettings,
  }
}

export default useSettings 