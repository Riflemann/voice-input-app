import React from 'react'

interface HistoryListProps {
  history?: string[]
}

export function HistoryList({ history }: HistoryListProps): React.ReactElement {
  return (
    <div className="p-6 bg-white rounded-lg shadow">
      <h3 className="text-lg font-bold text-gray-900 mb-4">История</h3>
      {history && history.length > 0 ? (
        <ul className="space-y-2">
          {history.map((item, index) => (
            <li key={index} className="px-4 py-2 bg-gray-50 rounded border-l-4 border-blue-500 text-gray-700">
              {item}
            </li>
          ))}
        </ul>
      ) : (
        <p className="text-gray-400 text-center py-8">История пуста</p>
      )}
    </div>
  )
}
