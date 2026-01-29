import React, { useState, useEffect } from 'react'
import { useRecord } from '../RecordButton/useRecord'
import { useRecognitionStore } from '../../stores/recognitionStore'

export function FloatingWidget(): React.ReactElement {
  const { isRecording, startRecord, stopRecord } = useRecord()
  const { text } = useRecognitionStore()
  const [isExpanded, setIsExpanded] = useState(false)

  // Закрываем виджет при завершении записи
  useEffect(() => {
    if (!isRecording && isExpanded) {
      const timer = setTimeout(() => {
        setIsExpanded(false)
      }, 2000)
      return () => clearTimeout(timer)
    }
  }, [isRecording, isExpanded])

  const handleToggleRecording = async () => {
    if (isRecording) {
      await stopRecord()
    } else {
      setIsExpanded(true)
      await startRecord()
    }
  }

  const handleCopyText = async () => {
    if (text) {
      try {
        await navigator.clipboard.writeText(text)
        alert('✓ Текст скопирован в буфер обмена')
      } catch (err) {
        console.error('Failed to copy:', err)
      }
    }
  }

  return (
    <div className="fixed bottom-4 right-4 z-50">
      {/* Основная кнопка виджета */}
      <div
        onClick={handleToggleRecording}
        className={`rounded-full p-4 shadow-2xl transition-all duration-300 cursor-pointer ${
          isRecording
            ? 'bg-red-600 shadow-red-500/50 animate-pulse w-20 h-20'
            : 'bg-blue-600 shadow-blue-500/50 hover:bg-blue-700 w-16 h-16'
        }`}
      >
        <div className="flex flex-col items-center justify-center h-full gap-2">
          {isRecording ? (
            <>
              <div className="text-white text-xs font-semibold">Запись</div>
              <div className="flex gap-1">
                <span className="w-1.5 h-1.5 bg-white rounded-full animate-pulse" />
                <span
                  className="w-1.5 h-1.5 bg-white rounded-full animate-pulse"
                  style={{ animationDelay: '0.2s' }}
                />
                <span
                  className="w-1.5 h-1.5 bg-white rounded-full animate-pulse"
                  style={{ animationDelay: '0.4s' }}
                />
              </div>
            </>
          ) : (
            <div className="text-white text-xl">🎤</div>
          )}
        </div>
      </div>

      {/* Развернутая панель с дополнительными действиями */}
      {isExpanded && text && (
        <div className="absolute bottom-24 right-0 bg-white rounded-lg shadow-2xl p-4 w-64">
          <div className="space-y-3">
            <div>
              <p className="text-xs text-gray-500 font-semibold mb-1">Распознанный текст:</p>
              <p className="text-sm text-gray-800 p-2 bg-gray-100 rounded line-clamp-4">
                {text}
              </p>
            </div>
            <button
              onClick={handleCopyText}
              className="w-full px-3 py-2 bg-green-600 text-white text-sm font-medium rounded hover:bg-green-700 transition-colors"
            >
              📋 Копировать текст
            </button>
          </div>
        </div>
      )}
    </div>
  )
}
