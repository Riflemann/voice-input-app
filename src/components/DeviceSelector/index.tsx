import React, { useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useAudioStore, type InputDevice } from '../../stores/audioStore'

export function DeviceSelector(): React.ReactElement {
  const {
    selectedDevice,
    setSelectedDevice,
    availableDevices,
    setAvailableDevices,
    isRecording
  } = useAudioStore()

  // Загружаем список устройств при монтировании
  useEffect(() => {
    const loadDevices = async () => {
      try {
        const devices = await invoke<InputDevice[]>('get_input_device_names')
        setAvailableDevices(devices)
        
        // Если устройство не выбрано, загружаем дефолтное
        if (!selectedDevice && devices.length > 0) {
          const defaultDevice = await invoke<InputDevice>('get_default_input_device_name')
          setSelectedDevice(defaultDevice)
        }
      } catch (err) {
        console.error('Failed to load audio devices:', err)
      }
    }
    
    loadDevices()
  }, [])

  const handleDeviceChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
    const deviceName = event.target.value
    const device = availableDevices.find(d => d.name === deviceName)
    if (device) {
      setSelectedDevice(device)
    }
  }

  return (
    <div className="flex flex-col gap-2">
      <label className="text-sm font-medium text-gray-700">
        Устройство ввода
      </label>
      <select
        value={selectedDevice?.name || ''}
        onChange={handleDeviceChange}
        disabled={isRecording}
        className="px-3 py-2 border border-gray-300 rounded-lg bg-white text-gray-900 
                   disabled:bg-gray-100 disabled:cursor-not-allowed
                   focus:ring-2 focus:ring-blue-500 focus:border-transparent
                   transition-colors"
      >
        {availableDevices.length === 0 ? (
          <option value="">Загрузка устройств...</option>
        ) : (
          availableDevices.map((device) => (
            <option key={device.name} value={device.name}>
              {device.name}
            </option>
          ))
        )}
      </select>
      {isRecording && (
        <p className="text-xs text-gray-500">
          Остановите запись для смены устройства
        </p>
      )}
    </div>
  )
}
