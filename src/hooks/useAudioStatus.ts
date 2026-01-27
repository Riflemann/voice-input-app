// @ts-ignore
import React, { useState } from 'react'

export type AudioStatus = 'idle' | 'recording' | 'processing' | 'error'

interface UseAudioStatusReturn {
  status: AudioStatus
  setStatus: (status: AudioStatus) => void
}

export function useAudioStatus(): UseAudioStatusReturn {
  const [status, setStatus] = useState<AudioStatus>('idle')

  return { status, setStatus }
}
