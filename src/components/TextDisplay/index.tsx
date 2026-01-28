import React from 'react'
import { useRecognitionStore } from '../../stores/recognitionStore'

export function TextDisplay(): React.ReactElement {
  const { text, isProcessing, isRecognizing } = useRecognitionStore()

  const isLoading = isProcessing || isRecognizing

  return (
    <div className="p-6 bg-white rounded-lg shadow">
      {isLoading && (
        <p className="text-gray-500 animate-pulse">Обработка аудио...</p>
      )}
      {text && !isLoading && (
        <p className="text-lg text-gray-800 font-medium">{text}</p>
      )}
      {!text && !isLoading && (
        <p className="text-gray-400">Распознанный текст появится здесь</p>
      )}
    </div>
  )
}
