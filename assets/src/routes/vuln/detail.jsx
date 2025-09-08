import { useState, useEffect } from 'react'
import { useParams, Link } from 'react-router-dom'
import { getVulnerabilityDetail } from '../../lib/api'

const VulnerabilityDetailPage = () => {
  const { id } = useParams()
  const [vulnerability, setVulnerability] = useState(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')

  useEffect(() => {
    fetchVulnerabilityDetail()
  }, [id])

  const fetchVulnerabilityDetail = async () => {
    setLoading(true)
    setError('')

    try {
      const response = await getVulnerabilityDetail(id)
      const { data } = response.data
      setVulnerability(data)
    } catch (err) {
      setError('获取漏洞详情失败')
      console.error('Fetch vulnerability detail error:', err)
    } finally {
      setLoading(false)
    }
  }

  const formatDate = (dateString) => {
    const date = new Date(dateString)
    return date.toLocaleDateString('zh-CN', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    })
  }

  return (
    <div className="max-w-4xl mx-auto">
      <div className="mb-6">
        <Link
          to="/vulns"
          className="inline-flex items-center text-indigo-600 hover:text-indigo-800"
        >
          <svg className="w-5 h-5 mr-2" fill="currentColor" viewBox="0 0 20 20">
            <path fillRule="evenodd" d="M9.707 16.707a1 1 0 01-1.414 0l-6-6a1 1 0 010-1.414l6-6a1 1 0 011.414 1.414L5.414 9H17a1 1 0 110 2H5.414l4.293 4.293a1 1 0 010 1.414z" clipRule="evenodd" />
          </svg>
          返回漏洞列表
        </Link>
      </div>

      {error && (
        <div className="p-4 mb-6 border border-red-200 rounded-md bg-red-50">
          <div className="text-sm text-red-800">
            {error}
          </div>
        </div>
      )}

      {loading ? (
        <div className="flex items-center justify-center h-64">
          <div className="w-12 h-12 border-b-2 border-indigo-600 rounded-full animate-spin"></div>
        </div>
      ) : vulnerability ? (
        <div className="overflow-hidden bg-white shadow sm:rounded-lg">
          <div className="px-4 py-5 border-b border-gray-200 sm:px-6">
            <h1 className="text-2xl font-medium leading-6 text-gray-900">
              {vulnerability.title}
            </h1>
            <div className="flex items-center mt-2">
              <span className={`px-2 inline-flex text-xs leading-5 font-semibold rounded-full ${
                vulnerability.severity === 'Critical' ? 'bg-red-100 text-red-800' :
                vulnerability.severity === 'High' ? 'bg-orange-100 text-orange-800' :
                vulnerability.severity === 'Medium' ? 'bg-yellow-100 text-yellow-800' :
                'bg-green-100 text-green-800'
              }`}>
                {vulnerability.severity}
              </span>
              <span className="ml-2 text-sm text-gray-500">
                CVE: {vulnerability.cve || 'N/A'}
              </span>
            </div>
          </div>
          <div className="px-4 py-5 sm:px-6">
            <div className="grid grid-cols-1 gap-y-4 gap-x-8 sm:grid-cols-2">
              <div>
                <h3 className="mb-2 text-lg font-medium text-gray-900">基本信息</h3>
                <dl className="grid grid-cols-1 gap-y-2">
                  <div>
                    <dt className="text-sm font-medium text-gray-500">漏洞编号</dt>
                    <dd className="mt-1 text-sm text-gray-900">{vulnerability.key}</dd>
                  </div>
                  <div>
                    <dt className="text-sm font-medium text-gray-500">披露日期</dt>
                    <dd className="mt-1 text-sm text-gray-900">
                      {vulnerability.disclosure ? formatDate(vulnerability.disclosure) : 'N/A'}
                    </dd>
                  </div>
                  <div>
                    <dt className="text-sm font-medium text-gray-500">来源</dt>
                    <dd className="mt-1 text-sm text-gray-900">{vulnerability.source}</dd>
                  </div>
                  <div>
                    <dt className="text-sm font-medium text-gray-500">详情链接</dt>
                    <dd className="mt-1 text-sm text-gray-900">
                      {vulnerability.detail_link ? (
                        <a
                          href={vulnerability.detail_link}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="text-indigo-600 hover:text-indigo-800"
                        >
                          {vulnerability.detail_link}
                        </a>
                      ) : (
                        'N/A'
                      )}
                    </dd>
                  </div>
                </dl>
              </div>
              <div>
                <h3 className="mb-2 text-lg font-medium text-gray-900">标签</h3>
                <div className="flex flex-wrap gap-2">
                  {vulnerability.tags.map((tag, index) => (
                    <span
                      key={index}
                      className="px-2 py-1 text-xs font-medium text-blue-800 bg-blue-100 rounded-full"
                    >
                      {tag}
                    </span>
                  ))}
                </div>
              </div>
            </div>

            <div className="mt-6">
              <h3 className="mb-2 text-lg font-medium text-gray-900">描述</h3>
              <p className="text-sm text-gray-900 whitespace-pre-wrap">
                {vulnerability.description}
              </p>
            </div>

            <div className="mt-6">
              <h3 className="mb-2 text-lg font-medium text-gray-900">解决方案</h3>
              <p className="text-sm text-gray-900 whitespace-pre-wrap">
                {vulnerability.solutions || '暂无解决方案信息'}
              </p>
            </div>

            {vulnerability.reference_links && vulnerability.reference_links.length > 0 && (
              <div className="mt-6">
                <h3 className="mb-2 text-lg font-medium text-gray-900">参考链接</h3>
                <ul className="text-sm text-gray-900 list-disc list-inside">
                  {vulnerability.reference_links.map((link, index) => (
                    <li key={index} className="mb-1">
                      <a
                        href={link}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-indigo-600 hover:text-indigo-800"
                      >
                        {link}
                      </a>
                    </li>
                  ))}
                </ul>
              </div>
            )}

            {vulnerability.github_search && vulnerability.github_search.length > 0 && (
              <div className="mt-6">
                <h3 className="mb-2 text-lg font-medium text-gray-900">GitHub搜索</h3>
                <ul className="text-sm text-gray-900 list-disc list-inside">
                  {vulnerability.github_search.map((search, index) => (
                    <li key={index} className="mb-1">
                      {search}
                    </li>
                  ))}
                </ul>
              </div>
            )}

            <div className="mt-6">
              <h3 className="mb-2 text-lg font-medium text-gray-900">触发原因</h3>
              <ul className="text-sm text-gray-900 list-disc list-inside">
                {vulnerability.reasons.map((reason, index) => (
                  <li key={index} className="mb-1">
                    {reason}
                  </li>
                ))}
              </ul>
            </div>

            <div className="grid grid-cols-1 mt-6 gap-y-2 sm:grid-cols-2">
              <div>
                <dt className="text-sm font-medium text-gray-500">创建时间</dt>
                <dd className="mt-1 text-sm text-gray-900">
                  {formatDate(vulnerability.created_at)}
                </dd>
              </div>
              <div>
                <dt className="text-sm font-medium text-gray-500">更新时间</dt>
                <dd className="mt-1 text-sm text-gray-900">
                  {formatDate(vulnerability.updated_at)}
                </dd>
              </div>
            </div>
          </div>
        </div>
      ) : (
        <div className="overflow-hidden bg-white shadow sm:rounded-lg">
          <div className="px-4 py-5 sm:px-6">
            <div className="text-center text-gray-500">
              未找到漏洞信息
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default VulnerabilityDetailPage
