import { useState, useEffect } from 'react'
import { getSyncDataTask, createOrUpdateSyncDataTask } from '../../lib/api'

const SyncDataTaskPage = () => {
  const [task, setTask] = useState({
    name: '',
    interval_minutes: 60,
    status: true,
  })
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState('')
  const [success, setSuccess] = useState('')

  useEffect(() => {
    fetchSyncDataTask()
  }, [])

  const fetchSyncDataTask = async () => {
    setLoading(true)
    setError('')

    try {
      const response = await getSyncDataTask()
      const { data } = response.data
      if (data) {
        setTask({
          name: data.name,
          interval_minutes: data.interval_minutes,
          status: data.status,
        })
      }
    } catch (err) {
      setError('获取同步任务信息失败')
      console.error('Fetch sync data task error:', err)
    } finally {
      setLoading(false)
    }
  }

  const handleChange = (e) => {
    const { name, value, type, checked } = e.target
    setTask(prev => ({
      ...prev,
      [name]: type === 'checkbox' ? checked : type === 'number' ? parseInt(value) : value,
    }))
  }

  const handleSubmit = async (e) => {
    e.preventDefault()
    setSaving(true)
    setError('')
    setSuccess('')

    try {
      await createOrUpdateSyncDataTask(task)
      setSuccess('同步任务配置已保存')
      // 重新获取任务信息以确保数据是最新的
      fetchSyncDataTask()
    } catch (err) {
      setError('保存同步任务配置失败')
      console.error('Save sync data task error:', err)
    } finally {
      setSaving(false)
    }
  }

  return (
    <div className="max-w-3xl mx-auto">
      <div className="mb-8 text-center">
        <h1 className="mb-2 text-3xl font-extrabold text-gray-900">
          数据同步任务配置
        </h1>
        <p className="text-gray-600">
          配置和管理漏洞数据同步任务
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
              同步任务设置
            </h2>
          </div>
          <form onSubmit={handleSubmit} className="px-4 py-5 sm:px-6">
            <div className="grid grid-cols-1 gap-y-6 gap-x-4 sm:grid-cols-6">
              <div className="sm:col-span-4">
                <label htmlFor="name" className="block text-sm font-medium text-gray-700">
                  任务名称
                </label>
                <div className="mt-1">
                  <input
                    type="text"
                    name="name"
                    id="name"
                    value={task.name}
                    onChange={handleChange}
                    required
                    className="block w-full px-3 py-2 placeholder-gray-400 border border-gray-300 rounded-md shadow-sm appearance-none focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                  />
                </div>
              </div>

              <div className="sm:col-span-4">
                <label htmlFor="interval_minutes" className="block text-sm font-medium text-gray-700">
                  同步间隔（分钟）
                </label>
                <div className="mt-1">
                  <input
                    type="number"
                    name="interval_minutes"
                    id="interval_minutes"
                    min="1"
                    max="1440"
                    value={task.interval_minutes}
                    onChange={handleChange}
                    required
                    className="block w-full px-3 py-2 placeholder-gray-400 border border-gray-300 rounded-md shadow-sm appearance-none focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                  />
                </div>
                <p className="mt-2 text-sm text-gray-500">
                  设置数据同步的时间间隔，范围1-1440分钟（1天）
                </p>
              </div>

              <div className="sm:col-span-4">
                <div className="flex items-start">
                  <div className="flex items-center h-5">
                    <input
                      id="status"
                      name="status"
                      type="checkbox"
                      checked={task.status}
                      onChange={handleChange}
                      className="w-4 h-4 text-indigo-600 border-gray-300 rounded focus:ring-indigo-500"
                    />
                  </div>
                  <div className="ml-3 text-sm">
                    <label htmlFor="status" className="font-medium text-gray-700">
                      启用任务
                    </label>
                    <p className="text-gray-500">
                      启用后将按照设定的时间间隔自动同步数据
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

export default SyncDataTaskPage
