import React, { useState } from 'react'
import { useRecognitionStore } from '../../stores/recognitionStore'

export function TextDisplay(): React.ReactElement {
  const { text, isProcessing, isRecognizing, lastResultEmpty } = useRecognitionStore()
  const [copied, setCopied] = useState(false)

  const isLoading = isProcessing || isRecognizing

  const handleCopy = async () => {
    if (text) {
      try {
        // Используем встроенный Browser Clipboard API
        await navigator.clipboard.writeText(text)
        setCopied(true)
        setTimeout(() => setCopied(false), 2000)
      } catch (err) {
        console.error('Failed to copy:', err)
      }
    }
  }

  return (
    <div className="p-6 bg-white rounded-lg shadow">
      {isLoading && (
        <p className="text-gray-500 animate-pulse">Обработка аудио...</p>
      )}
      {text && !isLoading && (
        <div className="space-y-3">
          <p className="text-lg text-gray-800 font-medium">{text}</p>
          <button
            onClick={handleCopy}
            className={`px-4 py-2 rounded text-sm font-medium transition-all ${
              copied
                ? 'bg-green-600 text-white'
                : 'bg-gray-200 text-gray-800 hover:bg-gray-300'
            }`}
          >
            {copied ? '✓ Скопировано' : '📋 Копировать'}
          </button>
        </div>
      )}
      {!text && !isLoading && lastResultEmpty && (
        <p className="text-gray-500">Речь не обнаружена</p>
      )}
      {!text && !isLoading && !lastResultEmpty && (
        <p className="text-gray-400">Распознанный текст появится здесь</p>
      )}
    </div>
  )
}
