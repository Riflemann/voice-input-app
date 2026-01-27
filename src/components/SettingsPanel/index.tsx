import React, { useState } from 'react'

interface SettingsPanelProps {}

export function SettingsPanel({}: SettingsPanelProps): React.ReactElement {
  const [isOpen, setIsOpen] = useState(false)

  return (
    <div className="p-6 bg-white rounded-lg shadow">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-bold text-gray-900">Параметры</h3>
        <button
          onClick={() => setIsOpen(!isOpen)}
          className="text-gray-500 hover:text-gray-700"
        >
          {isOpen ? '▼' : '▶'}
        </button>
      </div>
      {isOpen && (
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <label className="text-gray-700">Язык</label>
            <select className="px-3 py-2 border border-gray-300 rounded-lg">
              <option>Русский</option>
              <option>English</option>
            </select>
          </div>
          {/* TODO: Добавить больше настроек */}
        </div>
      )}
    </div>
  )
}
