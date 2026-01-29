import React from 'react'
import { RecordButton } from './components/RecordButton'
import { TextDisplay } from './components/TextDisplay'
import { HistoryList } from './components/HistoryList'
import { SettingsPanel } from './components/SettingsPanel'
import { FloatingWidget } from './components/FloatingWidget'
import { useRecognitionStore } from './stores/recognitionStore'
import { useWhisper } from './hooks/useWhisper'
import './index.css'

interface AppProps {}

const App: React.FC<AppProps> = () => {
  const { text, isProcessing, history } = useRecognitionStore()
  const { isInitialized, isInitializing, error } = useWhisper()

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100">
      {/* Плавающий виджет с кнопкой записи */}
      <FloatingWidget />
      
      <header className="bg-white shadow sticky top-0 z-10">
        <div className="max-w-7xl mx-auto py-6 px-4 sm:px-6 lg:px-8 flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold text-gray-900">🎙️ Voice Input App</h1>
            <p className="text-gray-600 text-sm mt-1">Распознавание речи в реальном времени</p>
          </div>
          
          {/* Статус инициализации Whisper */}
          {isInitializing && (
            <div className="flex items-center text-yellow-600">
              <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-yellow-600 mr-2"></div>
              <span className="text-sm">Загрузка модели...</span>
            </div>
          )}
          {isInitialized && !error && (
            <div className="flex items-center text-green-600">
              <svg className="w-5 h-5 mr-1" fill="currentColor" viewBox="0 0 20 20">
                <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd"/>
              </svg>
              <span className="text-sm font-medium">Whisper готов</span>
            </div>
          )}
          {error && (
            <div className="flex items-center text-red-600 cursor-help" title={error}>
              <svg className="w-5 h-5 mr-1" fill="currentColor" viewBox="0 0 20 20">
                <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd"/>
              </svg>
              <span className="text-sm">Ошибка модели</span>
            </div>
          )}
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
            <TextDisplay />

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
