import React from 'react'
import { RecordButton } from './components/RecordButton'
import { TextDisplay } from './components/TextDisplay'
import { HistoryList } from './components/HistoryList'
import { SettingsPanel } from './components/SettingsPanel'
import { useRecognitionStore } from './stores/recognitionStore'
import './index.css'

interface AppProps {}

const App: React.FC<AppProps> = () => {
  const { text, isProcessing, history } = useRecognitionStore()

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100">
      <header className="bg-white shadow sticky top-0 z-10">
        <div className="max-w-7xl mx-auto py-6 px-4 sm:px-6 lg:px-8">
          <h1 className="text-3xl font-bold text-gray-900">🎙️ Voice Input App</h1>
          <p className="text-gray-600 text-sm mt-1">Распознавание речи в реальном времени</p>
        </div>
      </header>

      <main className="max-w-6xl mx-auto py-8 px-4 sm:px-6 lg:px-8">
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
          {/* Main Content */}
          <div className="lg:col-span-2 space-y-6">
            {/* Record Button */}
            <div className="flex justify-center">
              <RecordButton />
            </div>

            {/* Text Display */}
            <TextDisplay text={text} isLoading={isProcessing} />

            {/* Settings */}
            <SettingsPanel />
          </div>

          {/* Sidebar */}
          <div className="lg:col-span-1">
            <HistoryList history={history} />
          </div>
        </div>
      </main>

      <footer className="bg-white shadow mt-12 border-t">
        <div className="max-w-7xl mx-auto py-6 px-4 sm:px-6 lg:px-8">
          <p className="text-gray-600 text-sm text-center">
            © 2024 Voice Input App. Built with React + TypeScript + Vite + Tailwind
          </p>
        </div>
      </footer>
    </div>
  )
}

export default App
