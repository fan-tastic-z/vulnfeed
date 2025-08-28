import { useState, useEffect } from 'react'
import { getDingBotConfig, createOrUpdateDingBotConfig } from '../../lib/api'

const DingBotConfigPage = () => {
  const [config, setConfig] = useState({
    access_token: '',
    secret_token: '',
    status: true,
  })
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState('')
  const [success, setSuccess] = useState('')

  useEffect(() => {
    fetchDingBotConfig()
  }, [])

  const fetchDingBotConfig = async () => {
    setLoading(true)
    setError('')

    try {
      const response = await getDingBotConfig()
      const { data } = response.data
      if (data) {
        setConfig({
          access_token: data.access_token,
          secret_token: data.secret_token,
          status: data.status,
        })
      }
    } catch (err) {
      setError('获取钉钉机器人配置失败')
      console.error('Fetch ding bot config error:', err)
    } finally {
      setLoading(false)
    }
  }

  const handleChange = (e) => {
    const { name, value, type, checked } = e.target
    setConfig(prev => ({
      ...prev,
      [name]: type === 'checkbox' ? checked : value,
    }))
  }

  const handleSubmit = async (e) => {
    e.preventDefault()
    setSaving(true)
    setError('')
    setSuccess('')

    try {
      await createOrUpdateDingBotConfig(config)
      setSuccess('钉钉机器人配置已保存')
      // 重新获取配置信息以确保数据是最新的
      fetchDingBotConfig()
    } catch (err) {
      setError('保存钉钉机器人配置失败')
      console.error('Save ding bot config error:', err)
    } finally {
      setSaving(false)
    }
  }

  return (
    <div className="max-w-3xl mx-auto">
      <div className="mb-8 text-center">
        <h1 className="mb-2 text-3xl font-extrabold text-gray-900">
          钉钉机器人配置
        </h1>
        <p className="text-gray-600">
          配置钉钉机器人以接收漏洞推送通知
        </p>
      </div>

      {error && (
        <div className="p-4 mb-6 border border-red-200 rounded-md bg-red-50">
          <div className="text-sm text-red-800">
            {error}
          </div>
        </div>
      )}

      {success && (
        <div className="p-4 mb-6 border border-green-200 rounded-md bg-green-50">
          <div className="text-sm text-green-800">
            {success}
          </div>
        </div>
      )}

      {loading ? (
        <div className="flex items-center justify-center h-64">
          <div className="w-12 h-12 border-b-2 border-indigo-600 rounded-full animate-spin"></div>
        </div>
      ) : (
        <div className="overflow-hidden bg-white shadow sm:rounded-lg">
          <div className="px-4 py-5 border-b border-gray-200 sm:px-6">
            <h2 className="text-lg font-medium leading-6 text-gray-900">
              机器人配置
            </h2>
          </div>
          <form onSubmit={handleSubmit} className="px-4 py-5 sm:px-6">
            <div className="grid grid-cols-1 gap-y-6 gap-x-4 sm:grid-cols-6">
              <div className="sm:col-span-4">
                <label htmlFor="access_token" className="block text-sm font-medium text-gray-700">
                  Access Token
                </label>
                <div className="mt-1">
                  <input
                    type="password"
                    name="access_token"
                    id="access_token"
                    value={config.access_token}
                    onChange={handleChange}
                    required
                    className="block w-full px-3 py-2 placeholder-gray-400 border border-gray-300 rounded-md shadow-sm appearance-none focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                  />
                </div>
                <p className="mt-2 text-sm text-gray-500">
                  钉钉机器人的Access Token
                </p>
              </div>

              <div className="sm:col-span-4">
                <label htmlFor="secret_token" className="block text-sm font-medium text-gray-700">
                  Secret Token
                </label>
                <div className="mt-1">
                  <input
                    type="password"
                    name="secret_token"
                    id="secret_token"
                    value={config.secret_token}
                    onChange={handleChange}
                    required
                    className="block w-full px-3 py-2 placeholder-gray-400 border border-gray-300 rounded-md shadow-sm appearance-none focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                  />
                </div>
                <p className="mt-2 text-sm text-gray-500">
                  钉钉机器人的Secret Token（用于签名验证）
                </p>
              </div>

              <div className="sm:col-span-4">
                <div className="flex items-start">
                  <div className="flex items-center h-5">
                    <input
                      id="status"
                      name="status"
                      type="checkbox"
                      checked={config.status}
                      onChange={handleChange}
                      className="w-4 h-4 text-indigo-600 border-gray-300 rounded focus:ring-indigo-500"
                    />
                  </div>
                  <div className="ml-3 text-sm">
                    <label htmlFor="status" className="font-medium text-gray-700">
                      启用推送
                    </label>
                    <p className="text-gray-500">
                      启用后将通过钉钉机器人推送漏洞信息
                    </p>
                  </div>
                </div>
              </div>
            </div>

            <div className="pt-5 mt-8 border-t border-gray-200">
              <div className="flex justify-end">
                <button
                  type="submit"
                  disabled={saving}
                  className="inline-flex justify-center px-4 py-2 ml-3 text-sm font-medium text-white bg-indigo-600 border border-transparent rounded-md shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50"
                >
                  {saving ? '保存中...' : '保存配置'}
                </button>
              </div>
            </div>
          </form>
        </div>
      )}
    </div>
  )
}

export default DingBotConfigPage