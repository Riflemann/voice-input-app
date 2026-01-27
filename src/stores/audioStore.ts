import { create } from 'zustand'

interface AudioStore {
  isRecording: boolean
  setIsRecording: (recording: boolean) => void
  audioData: unknown
  setAudioData: (data: unknown) => void
}

export const useAudioStore = create<AudioStore>((set) => ({
  isRecording: false,
  setIsRecording: (recording) => set({ isRecording: recording }),
  audioData: null,
  setAudioData: (data) => set({ audioData: data }),
}))
