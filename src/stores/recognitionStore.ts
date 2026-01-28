import { create } from 'zustand'

interface RecognitionStore {
  text: string
  setText: (text: string) => void
  isProcessing: boolean
  setIsProcessing: (processing: boolean) => void
  isRecognizing: boolean
  setIsRecognizing: (recognizing: boolean) => void
  preWavPath: string | null
  postWavPath: string | null
  setWavPaths: (pre: string | null, post: string | null) => void
  history: string[]
  addToHistory: (item: string) => void
}

export const useRecognitionStore = create<RecognitionStore>((set) => ({
  text: '',
  setText: (text) => set({ text }),
  isProcessing: false,
  setIsProcessing: (processing) => set({ isProcessing: processing }),
  isRecognizing: false,
  setIsRecognizing: (recognizing) => set({ isRecognizing: recognizing }),
  preWavPath: null,
  postWavPath: null,
  setWavPaths: (pre, post) => set({ preWavPath: pre, postWavPath: post }),
  history: [],
  addToHistory: (item) =>
    set((state) => ({
      history: [item, ...state.history],
    })),
}))
