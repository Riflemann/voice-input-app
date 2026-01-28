import { create } from 'zustand'

export interface InputDevice {
  name: string
}

interface AudioStore {
  isRecording: boolean
  setIsRecording: (recording: boolean) => void
  audioData: unknown
  setAudioData: (data: unknown) => void
  selectedDevice: InputDevice | null
  setSelectedDevice: (device: InputDevice | null) => void
  availableDevices: InputDevice[]
  setAvailableDevices: (devices: InputDevice[]) => void
}

export const useAudioStore = create<AudioStore>((set) => ({
  isRecording: false,
  setIsRecording: (recording) => set({ isRecording: recording }),
  audioData: null,
  setAudioData: (data) => set({ audioData: data }),
  selectedDevice: null,
  setSelectedDevice: (device) => set({ selectedDevice: device }),
  availableDevices: [],
  setAvailableDevices: (devices) => set({ availableDevices: devices }),
}))
