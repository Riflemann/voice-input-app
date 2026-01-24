import React from 'react'
import { useRecord } from './useRecord'

export function RecordButton(): React.ReactElement {
  const { isRecording, startRecord, stopRecord } = useRecord()

  return (
    <button
      onClick={isRecording ? stopRecord : startRecord}
      className={`px-6 py-3 rounded-lg font-semibold transition-all duration-300 ${
        isRecording
          ? 'bg-red-600 hover:bg-red-700 text-white animate-pulse'
          : 'bg-blue-600 hover:bg-blue-700 text-white'
      }`}
    >
      {isRecording ? '⏹ Остановить' : '🎤 Записать'}
    </button>
  )
}
