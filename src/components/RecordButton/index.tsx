import React from 'react'
import { useRecord } from './useRecord'

export function RecordButton(): React.ReactElement {
  const { isRecording, startRecord, stopRecord } = useRecord()

  return (
    <div className="relative inline-block">
      {/* Пульсирующие круги при записи */}
      {isRecording && (
        <>
          <span className="absolute inset-0 rounded-lg bg-red-500 opacity-75 animate-ping" />
          <span className="absolute inset-0 rounded-lg bg-red-500 opacity-50 animate-pulse" />
        </>
      )}
      
      <button
        onClick={isRecording ? stopRecord : startRecord}
        className={`relative px-6 py-3 rounded-lg font-semibold transition-all duration-300 ${
          isRecording
            ? 'bg-red-600 hover:bg-red-700 text-white shadow-lg shadow-red-500/50'
            : 'bg-blue-600 hover:bg-blue-700 text-white'
        }`}
      >
        <span className="flex items-center gap-2">
          {isRecording ? (
            <>
              <span className="inline-block w-3 h-3 bg-white rounded-full animate-pulse" />
              Остановить
            </>
          ) : (
            <>
              🎤 Записать
            </>
          )}
        </span>
      </button>
    </div>
  )
}
