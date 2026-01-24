import React, { useState } from 'react'

interface UseRecognitionReturn {
  text: string
  setText: (text: string) => void
  isLoading: boolean
  error: Error | null
  recognize: (audioData: unknown) => Promise<void>
}

export function useRecognition(): UseRecognitionReturn {
  const [text, setText] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<Error | null>(null)

  const recognize = async (audioData: unknown) => {
    setIsLoading(true)
    try {
      // TODO: Отправить на распознавание
    } catch (err) {
      if (err instanceof Error) {
        setError(err)
      }
    } finally {
      setIsLoading(false)
    }
  }

  return { text, setText, isLoading, error, recognize }
}
