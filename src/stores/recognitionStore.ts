import { create } from 'zustand'

interface RecognitionStore {
  text: string
  setText: (text: string) => void
  isProcessing: boolean
  setIsProcessing: (processing: boolean) => void
  history: string[]
  addToHistory: (item: string) => void
}

export const useRecognitionStore = create<RecognitionStore>((set) => ({
  text: '',
  setText: (text) => set({ text }),
  isProcessing: false,
  setIsProcessing: (processing) => set({ isProcessing: processing }),
  history: [],
  addToHistory: (item) =>
    set((state) => ({
      history: [item, ...state.history],
    })),
}))
