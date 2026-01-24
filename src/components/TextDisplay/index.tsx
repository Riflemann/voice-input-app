import React from 'react'

interface TextDisplayProps {
  text?: string
  isLoading?: boolean
}

export function TextDisplay({ text, isLoading }: TextDisplayProps): React.ReactElement {
  return (
    <div className="p-6 bg-white rounded-lg shadow">
      {isLoading && (
        <p className="text-gray-500 animate-pulse">Обработка...</p>
      )}
      {text && (
        <p className="text-lg text-gray-800 font-medium">{text}</p>
      )}
      {!text && !isLoading && (
        <p className="text-gray-400">Распознанный текст появится здесь</p>
      )}
    </div>
  )
}
