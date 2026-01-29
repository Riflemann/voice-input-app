/**
 * Хук для инициализации и использования Whisper
 */
import { useEffect, useState, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

interface RecognitionResult {
  text: string
  audio_path: string
}

export const useWhisper = () => {
  const [isInitialized, setIsInitialized] = useState(false)
  const [isInitializing, setIsInitializing] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const initAttemptedRef = useRef(false)

  // Инициализация Whisper при монтировании
  useEffect(() => {
    const initWhisper = async () => {
      if (initAttemptedRef.current) return

      initAttemptedRef.current = true
      setIsInitializing(true)
      setError(null)

      try {
        console.log('🎤 Initializing Whisper...')
        await invoke('init_whisper', { 
          modelSize: 'base' // или 'tiny', 'small', 'medium', 'large'
        })
        console.log('✅ Whisper initialized successfully')
        setIsInitialized(true)
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : String(err)
        console.error('❌ Failed to initialize Whisper:', errorMsg)
        setError(errorMsg)
      } finally {
        setIsInitializing(false)
      }
    }

    initWhisper()
  }, [])

  // Слушаем события распознавания
  useEffect(() => {
    const unlisten = listen<RecognitionResult>('recognition-completed', (event) => {
      console.log('🎯 Recognition completed:', event.payload.text)
      // Здесь можно обновить store или state
    })

    return () => {
      unlisten.then(fn => fn())
    }
  }, [])

  return {
    isInitialized,
    isInitializing,
    error
  }
}
