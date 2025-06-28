import { useEffect } from 'react'

export type KeyHandler = (event: KeyboardEvent) => void

interface UseKeyboardOptions {
  enabled?: boolean
  bindings: Array<{
    key: string
    ctrl?: boolean
    shift?: boolean
    alt?: boolean
    meta?: boolean
    handler: KeyHandler
  }>
}

export function useKeyboard({ enabled = true, bindings }: UseKeyboardOptions) {
  useEffect(() => {
    if (!enabled) return
    const handle = (event: KeyboardEvent) => {
      for (const binding of bindings) {
        if (
          event.key.toLowerCase() === binding.key.toLowerCase() &&
          (!!binding.ctrl === event.ctrlKey) &&
          (!!binding.shift === event.shiftKey) &&
          (!!binding.alt === event.altKey) &&
          (!!binding.meta === event.metaKey)
        ) {
          binding.handler(event)
        }
      }
    }
    window.addEventListener('keydown', handle)
    return () => window.removeEventListener('keydown', handle)
  }, [enabled, bindings])
}

export default useKeyboard 