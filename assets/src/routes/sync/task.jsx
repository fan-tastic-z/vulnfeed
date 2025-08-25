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
      <div className="text-center mb-8">
        <h1 className="text-3xl font-extrabold text-gray-900 mb-2">
          数据同步任务配置
        </h1>
        <p className="text-gray-600">
          配置和管理漏洞数据同步任务
        </p>
      </div>

      {error && (
        <div className="bg-red-50 border border-red-200 rounded-md p-4 mb-6">
          <div className="text-red-800 text-sm">
            {error}
          </div>
        </div>
      )}

      {success && (
        <div className="bg-green-50 border border-green-200 rounded-md p-4 mb-6">
          <div className="text-green-800 text-sm">
            {success}
          </div>
        </div>
      )}

      {loading ? (
        <div className="flex justify-center items-center h-64">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
        </div>
      ) : (
        <div className="bg-white shadow overflow-hidden sm:rounded-lg">
          <div className="px-4 py-5 sm:px-6 border-b border-gray-200">
            <h2 className="text-lg leading-6 font-medium text-gray-900">
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
                    className="shadow-sm focus:ring-indigo-500 focus:border-indigo-500 block w-full sm:text-sm border-gray-300 rounded-md"
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
                    className="shadow-sm focus:ring-indigo-500 focus:border-indigo-500 block w-full sm:text-sm border-gray-300 rounded-md"
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
                      className="focus:ring-indigo-500 h-4 w-4 text-indigo-600 border-gray-300 rounded"
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

            <div className="mt-8 border-t border-gray-200 pt-5">
              <div className="flex justify-end">
                <button
                  type="submit"
                  disabled={saving}
                  className="ml-3 inline-flex justify-center py-2 px-4 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50"
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
