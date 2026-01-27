import { invoke } from '@tauri-apps/api/core'

export async function startAudioCapture(): Promise<unknown> {
  return invoke('start_audio_capture')
}

export async function stopAudioCapture(): Promise<unknown> {
  return invoke('stop_audio_capture')
}

export async function recognizeAudio(audioPath: string): Promise<unknown> {
  return invoke('recognize_audio', { path: audioPath })
}
