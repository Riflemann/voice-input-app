// @ts-ignore
import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useRecognitionStore } from '../../stores/recognitionStore'
import { useAudioStore } from '../../stores/audioStore'
import { useRecognition } from '../../hooks/useRecognition'

interface UseRecordReturn {
  isRecording: boolean
  startRecord: () => Promise<void>
  stopRecord: () => Promise<void>
}

interface RecognitionResult {
  text: string
  audio_path: string
}

let listenersRefCount = 0
let unlistenProcessingRef: Promise<() => void> | null = null
let unlistenRecognitionRef: Promise<() => void> | null = null

export function useRecord(): UseRecordReturn {
  const [isRecording, setIsRecording] = useState(false)
  const { recognize } = useRecognition()
  const { setWavPaths, setIsProcessing, setText, setLastResultEmpty } = useRecognitionStore()
  const { selectedDevice } = useAudioStore()

  // Проверяем статус записи каждую секунду для отслеживания авто-стопа
  useEffect(() => {
    if (!isRecording) return

    const checkStatus = setInterval(async () => {
      try {
        const status = await invoke<boolean>('get_recording_status')
        if (!status && isRecording) {
          console.log('[useRecord] Auto-stop detected, updating UI')
          setIsRecording(false)
        }
      } catch (err) {
        console.error('Failed to check recording status:', err)
      }
    }, 1000)

    return () => clearInterval(checkStatus)
  }, [isRecording])

  // Подписываемся на события один раз глобально
  useEffect(() => {
    listenersRefCount += 1

    if (listenersRefCount === 1) {
      console.log('[useRecord] Setting up event listeners')
      let isProcessing = false

      unlistenProcessingRef = listen<[string, string]>('processing-finished', async (event) => {
        if (isProcessing) return

        isProcessing = true
        console.log('[useRecord] processing-finished event received:', event.payload)
        const [prePath, postPath] = event.payload
        console.log('Processing finished:', { prePath, postPath })

        // Сохраняем пути в store
        setWavPaths(prePath, postPath)
        setIsProcessing(false)

        // Автоматически запускаем распознавание на post-обработанном файле
        try {
          console.log('[useRecord] Starting recognition for:', postPath)
          await recognize(postPath)
        } catch (err) {
          console.error('Auto-recognition failed:', err)
        } finally {
          isProcessing = false
        }
      })

      // Подписываемся на событие recognition-completed для получения результата
      unlistenRecognitionRef = listen<RecognitionResult>('recognition-completed', (event) => {
        console.log('[useRecord] recognition-completed event received:', event.payload)
        const { text, audio_path } = event.payload
        console.log('Recognition completed:', { text, audio_path })

        // Обновляем текст в store
        const trimmed = text.trim()
        if (trimmed.length === 0) {
          setText('')
          setLastResultEmpty(true)
        } else {
          setText(text)
          setLastResultEmpty(false)
        }
      })
    }

    return () => {
      listenersRefCount = Math.max(0, listenersRefCount - 1)
      if (listenersRefCount === 0) {
        console.log('[useRecord] Cleaning up event listeners')
        unlistenProcessingRef?.then((fn) => fn()).catch(console.error)
        unlistenRecognitionRef?.then((fn) => fn()).catch(console.error)
        unlistenProcessingRef = null
        unlistenRecognitionRef = null
      }
    }
  }, [])

  const startRecord = async () => {
    setIsRecording(true)
    setIsProcessing(true)
    try {
      // Используем выбранное устройство или получаем дефолтное
      let deviceName: string
      if (selectedDevice) {
        deviceName = selectedDevice.name
      } else {
        const deviceInfo = await invoke<{ name: string }>('get_default_input_device_name')
        deviceName = deviceInfo.name
      }
      
      await invoke('start_recording', { device: deviceName })
      console.log('Recording started with device:', deviceName)
    } catch (err) {
      console.error('Failed to start recording:', err)
      setIsRecording(false)
      setIsProcessing(false)
    }
  }

  const stopRecord = async () => {
    setIsRecording(false)
    try {
      await invoke('stop_recording')
      console.log('Recording stopped')
    } catch (err) {
      console.error('Failed to stop recording:', err)
      setIsRecording(true)
    }
  }

  return { isRecording, startRecord, stopRecord }
}
