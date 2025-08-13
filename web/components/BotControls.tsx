'use client'

import { useState } from 'react'
import { Play, Square, RefreshCw } from 'lucide-react'

type BotStatus = 'idle' | 'running' | 'stopping'

export default function BotControls() {
  const [status, setStatus] = useState<BotStatus>('idle')
  const [isLoading, setIsLoading] = useState(false)

  const handleStart = async () => {
    setIsLoading(true)
    try {
      // TODO: Implement start bot API call
      setStatus('running')
    } catch (error) {
      console.error('Failed to start bot:', error)
    } finally {
      setIsLoading(false)
    }
  }

  const handleStop = async () => {
    setIsLoading(true)
    setStatus('stopping')
    try {
      // TODO: Implement stop bot API call
      setStatus('idle')
    } catch (error) {
      console.error('Failed to stop bot:', error)
    } finally {
      setIsLoading(false)
    }
  }

  const handleRestart = async () => {
    await handleStop()
    await handleStart()
  }

  return (
    <div className="flex items-center space-x-4">
      <div className="flex items-center space-x-2">
        <span className="text-sm font-medium text-gray-700">Status:</span>
        <span className={`px-2 py-1 text-xs rounded-full font-medium ${
          status === 'running' 
            ? 'bg-green-100 text-green-800' 
            : status === 'stopping'
            ? 'bg-yellow-100 text-yellow-800'
            : 'bg-gray-100 text-gray-800'
        }`}>
          {status.toUpperCase()}
        </span>
      </div>

      <div className="flex space-x-2">
        {status === 'idle' ? (
          <button
            onClick={handleStart}
            disabled={isLoading}
            className="flex items-center space-x-2 px-4 py-2 bg-green-500 text-white rounded-lg hover:bg-green-600 disabled:opacity-50"
          >
            <Play className="w-4 h-4" />
            <span>Start Bot</span>
          </button>
        ) : (
          <>
            <button
              onClick={handleStop}
              disabled={isLoading || status === 'stopping'}
              className="flex items-center space-x-2 px-4 py-2 bg-red-500 text-white rounded-lg hover:bg-red-600 disabled:opacity-50"
            >
              <Square className="w-4 h-4" />
              <span>Stop</span>
            </button>
            <button
              onClick={handleRestart}
              disabled={isLoading || status === 'stopping'}
              className="flex items-center space-x-2 px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 disabled:opacity-50"
            >
              <RefreshCw className="w-4 h-4" />
              <span>Restart</span>
            </button>
          </>
        )}
      </div>
    </div>
  )
}