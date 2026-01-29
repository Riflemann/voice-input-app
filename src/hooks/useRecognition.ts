// @ts-ignore
import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useRecognitionStore } from '../stores/recognitionStore'

interface UseRecognitionReturn {
  text: string
  setText: (text: string) => void
  isLoading: boolean
  error: Error | null
  recognize: (audioPath: string) => Promise<void>
}

export function useRecognition(): UseRecognitionReturn {
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<Error | null>(null)
  const { text, setText, setIsRecognizing, setLastResultEmpty } = useRecognitionStore()

  // @ts-ignore
  const recognize = async (audioPath: string) => {
    setIsLoading(true)
    setIsRecognizing(true)
    setError(null)
    try {
      const result = await invoke<string>('recognize_audio', { audioPath })
      const trimmed = result.trim()
      if (trimmed.length === 0) {
        setText('')
        setLastResultEmpty(true)
      } else {
        setText(result)
        setLastResultEmpty(false)
        useRecognitionStore.getState().addToHistory(result)
      }
    } catch (err) {
      const newError = err instanceof Error ? err : new Error(String(err))
      setError(newError)
      console.error('Recognition error:', newError)
    } finally {
      setIsLoading(false)
      setIsRecognizing(false)
    }
  }

  return { text, setText, isLoading, error, recognize }
}
