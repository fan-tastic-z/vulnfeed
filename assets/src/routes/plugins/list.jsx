import { useState, useEffect } from 'react'
import api from '../../lib/api'

const PluginListPage = () => {
  const [plugins, setPlugins] = useState([])
  const [noticePlugins, setNoticePlugins] = useState([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState(null)

  useEffect(() => {
    const fetchPlugins = async () => {
      try {
        setLoading(true)
        // 获取漏洞插件列表
        const response = await api.get('/plugins')
        setPlugins(response.data.data)

        // 获取安全公告插件列表
        const noticeResponse = await api.get('/notices')
        setNoticePlugins(noticeResponse.data.data)

        setError(null)
      } catch (err) {
        setError('获取插件列表失败')
        console.error('Failed to fetch plugins:', err)
      } finally {
        setLoading(false)
      }
    }

    fetchPlugins()
  }, [])

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="w-12 h-12 border-b-2 border-indigo-600 rounded-full animate-spin"></div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="p-4 border-l-4 border-red-400 bg-red-50">
        <div className="flex">
          <div className="flex-shrink-0">
            <svg className="w-5 h-5 text-red-400" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
              <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
            </svg>
          </div>
          <div className="ml-3">
            <p className="text-sm text-red-700">{error}</p>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="px-4 py-8 mx-auto max-w-7xl sm:px-6 lg:px-8">
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-gray-900">插件列表</h1>
        <p className="mt-1 text-sm text-gray-500">当前支持的数据同步插件</p>
      </div>

      {/* 漏洞插件列表 */}
      <div className="mb-8">
        <h2 className="mb-4 text-xl font-semibold text-gray-800">漏洞数据插件</h2>
        {plugins.length === 0 ? (
          <div className="py-6 text-center">
            <svg className="w-12 h-12 mx-auto text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4" />
            </svg>
            <h3 className="mt-2 text-sm font-medium text-gray-900">暂无漏洞插件</h3>
            <p className="mt-1 text-sm text-gray-500">当前系统中没有配置任何漏洞插件。</p>
          </div>
        ) : (
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {plugins.map((plugin) => (
              <div key={plugin.name} className="overflow-hidden bg-white rounded-lg shadow">
                <div className="p-4 sm:p-5">
                  <div className="flex items-center">
                    <div className="flex-shrink-0 p-2 bg-indigo-100 rounded-md">
                      <svg className="w-5 h-5 text-indigo-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" />
                      </svg>
                    </div>
                    <div className="ml-3">
                      <h3 className="text-base font-medium text-gray-900">{plugin.display_name}</h3>
                      <p className="text-xs text-gray-500">数据同步插件</p>
                    </div>
                  </div>
                </div>
                <div className="px-4 py-3 bg-gray-50 sm:px-5">
                  <div className="flex justify-center">
                    <a
                      href={plugin.link}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="inline-flex items-center px-3 py-1 text-sm font-medium text-indigo-700 bg-indigo-100 border border-transparent rounded-md hover:bg-indigo-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
                    >
                      查看数据源
                    </a>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* 安全公告插件列表 */}
      <div>
        <h2 className="mb-4 text-xl font-semibold text-gray-800">安全公告插件</h2>
        {noticePlugins.length === 0 ? (
          <div className="py-6 text-center">
            <svg className="w-12 h-12 mx-auto text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
            </svg>
            <h3 className="mt-2 text-sm font-medium text-gray-900">暂无安全公告插件</h3>
            <p className="mt-1 text-sm text-gray-500">当前系统中没有配置任何安全公告插件。</p>
          </div>
        ) : (
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {noticePlugins.map((plugin) => (
              <div key={plugin.name} className="overflow-hidden bg-white rounded-lg shadow">
                <div className="p-4 sm:p-5">
                  <div className="flex items-center">
                    <div className="flex-shrink-0 p-2 bg-green-100 rounded-md">
                      <svg className="w-5 h-5 text-green-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
                      </svg>
                    </div>
                    <div className="ml-3">
                      <h3 className="text-base font-medium text-gray-900">{plugin.display_name}</h3>
                      <p className="text-xs text-gray-500">安全公告插件</p>
                    </div>
                  </div>
                </div>
                <div className="px-4 py-3 bg-gray-50 sm:px-5">
                  <div className="flex justify-center">
                    <a
                      href={plugin.link}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="inline-flex items-center px-3 py-1 text-sm font-medium text-green-700 bg-green-100 border border-transparent rounded-md hover:bg-green-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500"
                    >
                      查看数据源
                    </a>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}

export default PluginListPage
