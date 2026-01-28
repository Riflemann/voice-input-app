// @ts-ignore
import React, { useState, useEffect } from 'react'
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

export function useRecord(): UseRecordReturn {
  const [isRecording, setIsRecording] = useState(false)
  const { recognize } = useRecognition()
  const { setWavPaths, setIsProcessing, setText } = useRecognitionStore()
  const { selectedDevice } = useAudioStore()

  // Подписываемся на событие processing-finished при монтировании компонента
  useEffect(() => {
    console.log('[useRecord] Setting up event listeners')
    
    const unlistenProcessing = listen<[string, string]>('processing-finished', async (event) => {
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
      }
    })

    // Подписываемся на событие recognition-completed для получения результата
    const unlistenRecognition = listen<RecognitionResult>('recognition-completed', (event) => {
      console.log('[useRecord] recognition-completed event received:', event.payload)
      const { text, audio_path } = event.payload
      console.log('Recognition completed:', { text, audio_path })
      
      // Обновляем текст в store (это дублирует результат из recognize(), но обеспечивает консистентность)
      setText(text)
    })

    return () => {
      console.log('[useRecord] Cleaning up event listeners')
      unlistenProcessing.then((fn) => fn()).catch(console.error)
      unlistenRecognition.then((fn) => fn()).catch(console.error)
    }
  }, [recognize, setWavPaths, setIsProcessing, setText])

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
