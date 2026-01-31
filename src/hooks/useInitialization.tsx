import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'

interface SetupStatus {
  models_initialized: boolean
  default_model_installed: boolean
  available_models: string[]
  installed_models: string[]
}

export const useInitialization = () => {
  const [isInitializing, setIsInitializing] = useState(true)
  const [initError, setInitError] = useState<string | null>(null)
  const [setupStatus, setSetupStatus] = useState<SetupStatus | null>(null)

  useEffect(() => {
    const initialize = async () => {
      try {
        // Get initial setup status
        const status = await invoke<SetupStatus>('get_setup_status')
        setSetupStatus(status)

        // If not initialized, run full initialization
        if (!status.models_initialized || !status.default_model_installed) {
          console.log('Running initialization...')
          const initStatus = await invoke<SetupStatus>('initialize_app')
          setSetupStatus(initStatus)
          console.log('Initialization complete:', initStatus)
        } else {
          console.log('Application already initialized')
        }
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : String(error)
        console.error('Initialization error:', errorMessage)
        setInitError(errorMessage)
      } finally {
        setIsInitializing(false)
      }
    }

    initialize()
  }, [])

  return {
    isInitializing,
    initError,
    setupStatus,
  }
}

interface InitializationScreenProps {
  isInitializing: boolean
  initError: string | null
}

export const InitializationScreen: React.FC<InitializationScreenProps> = ({
  isInitializing,
  initError,
}) => {
  if (!isInitializing && !initError) {
    return null
  }

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg p-8 max-w-md w-full mx-4">
        {isInitializing && (
          <div className="text-center">
            <div className="mb-4">
              <div className="inline-block">
                <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500"></div>
              </div>
            </div>
            <h2 className="text-xl font-semibold text-gray-800 mb-2">
              Initializing Application
            </h2>
            <p className="text-gray-600">
              Setting up Whisper models for speech recognition...
            </p>
            <p className="text-sm text-gray-500 mt-4">
              This may take a few minutes on first run
            </p>
          </div>
        )}

        {initError && (
          <div className="text-center">
            <div className="mb-4">
              <div className="inline-block text-red-500">
                <svg
                  className="w-12 h-12"
                  fill="currentColor"
                  viewBox="0 0 20 20"
                >
                  <path
                    fillRule="evenodd"
                    d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
                    clipRule="evenodd"
                  />
                </svg>
              </div>
            </div>
            <h2 className="text-xl font-semibold text-gray-800 mb-2">
              Initialization Failed
            </h2>
            <p className="text-gray-600 text-sm mb-4">
              {initError}
            </p>
            <button
              onClick={() => window.location.reload()}
              className="bg-blue-500 hover:bg-blue-600 text-white font-medium py-2 px-4 rounded"
            >
              Retry
            </button>
          </div>
        )}
      </div>
    </div>
  )
}
