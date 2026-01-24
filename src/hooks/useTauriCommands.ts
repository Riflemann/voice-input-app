import { invoke } from '@tauri-apps/api/core'

interface UseTauriCommandsReturn {
  callCommand: (command: string, payload?: Record<string, unknown>) => Promise<unknown>
}

export function useTauriCommands(): UseTauriCommandsReturn {
  const callCommand = async (command: string, payload?: Record<string, unknown>): Promise<unknown> => {
    try {
      const result = await invoke(command, payload)
      return result
    } catch (error) {
      console.error(`Ошибка команды ${command}:`, error)
      throw error
    }
  }

  return { callCommand }
}
