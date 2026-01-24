// @ts-ignore
import React, { useState } from 'react'

interface UseRecordReturn {
  isRecording: boolean
  startRecord: () => Promise<void>
  stopRecord: () => Promise<void>
}

export function useRecord(): UseRecordReturn {
  const [isRecording, setIsRecording] = useState(false)

  const startRecord = async () => {
    setIsRecording(true)
    // TODO: Реализовать захват аудио
  }

  const stopRecord = async () => {
    setIsRecording(false)
    // TODO: Отправить аудио на обработку
  }

  return { isRecording, startRecord, stopRecord }
}
