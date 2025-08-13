'use client'

import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import ConfigEditor from '@/components/ConfigEditor'
import BotControls from '@/components/BotControls'
import { fetchConfig } from '@/lib/api'

export default function Home() {
  const [activeTab, setActiveTab] = useState<'config' | 'dashboard'>('config')
  
  const { data: configData, isLoading, error } = useQuery({
    queryKey: ['config'],
    queryFn: fetchConfig,
  })

  return (
    <div className="min-h-screen bg-gray-50">
      <header className="bg-white shadow">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center py-6">
            <h1 className="text-3xl font-bold text-gray-900">
              Solana Arbitrage Bot
            </h1>
            <BotControls />
          </div>
        </div>
      </header>

      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="mb-6">
          <nav className="flex space-x-4">
            <button
              onClick={() => setActiveTab('config')}
              className={`px-4 py-2 rounded-lg font-medium ${
                activeTab === 'config'
                  ? 'bg-blue-500 text-white'
                  : 'bg-white text-gray-700 hover:bg-gray-100'
              }`}
            >
              Configuration
            </button>
            <button
              onClick={() => setActiveTab('dashboard')}
              className={`px-4 py-2 rounded-lg font-medium ${
                activeTab === 'dashboard'
                  ? 'bg-blue-500 text-white'
                  : 'bg-white text-gray-700 hover:bg-gray-100'
              }`}
            >
              Dashboard
            </button>
          </nav>
        </div>

        <div className="bg-white rounded-lg shadow">
          {activeTab === 'config' ? (
            <div className="p-6">
              <h2 className="text-xl font-semibold mb-4">Bot Configuration</h2>
              {isLoading ? (
                <div className="flex justify-center py-8">
                  <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
                </div>
              ) : error ? (
                <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded">
                  Failed to load configuration. Make sure the Rust server is running on port 8080.
                </div>
              ) : (
                <ConfigEditor initialConfig={configData?.config || ''} />
              )}
            </div>
          ) : (
            <div className="p-6">
              <h2 className="text-xl font-semibold mb-4">Dashboard</h2>
              <p className="text-gray-500">Dashboard coming soon...</p>
            </div>
          )}
        </div>
      </main>
    </div>
  )
}