import { useState, useEffect } from 'react'
import { Link } from 'react-router-dom'
import { getVulnerabilities } from '../../lib/api'

const VulnerabilityListPage = () => {
  const [vulnerabilities, setVulnerabilities] = useState([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')
  const [pageNo, setPageNo] = useState(1)
  const [pageSize] = useState(10)
  const [totalCount, setTotalCount] = useState(0)
  const [searchTerm, setSearchTerm] = useState('')

  useEffect(() => {
    fetchVulnerabilities()
  }, [pageNo, searchTerm])

  const fetchVulnerabilities = async () => {
    setLoading(true)
    setError('')

    try {
      const response = await getVulnerabilities(pageNo, pageSize, searchTerm)
      const { data, total_count } = response.data.data
      setVulnerabilities(data)
      setTotalCount(total_count)
    } catch (err) {
      setError('获取漏洞列表失败')
      console.error('Fetch vulnerabilities error:', err)
    } finally {
      setLoading(false)
    }
  }

  const handlePageChange = (newPageNo) => {
    setPageNo(newPageNo)
  }

  const handleSearch = (e) => {
    e.preventDefault()
    // 重置页码到第一页
    setPageNo(1)
  }

  const totalPages = Math.ceil(totalCount / pageSize)

  return (
    <div className="max-w-7xl mx-auto">
      <div className="text-center mb-8">
        <h1 className="text-3xl font-extrabold text-gray-900 mb-2">
          漏洞信息列表
        </h1>
        <p className="text-gray-600">
          当前共有 {totalCount} 个漏洞信息
        </p>
      </div>

      {/* 搜索框 */}
      <div className="mb-6">
        <form onSubmit={handleSearch} className="max-w-md mx-auto">
          <div className="relative">
            <input
              type="text"
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              placeholder="搜索漏洞标题或描述..."
              className="block w-full pl-4 pr-12 py-2 border border-gray-300 rounded-md leading-5 bg-white placeholder-gray-500 focus:outline-none focus:placeholder-gray-400 focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
            />
            <button
              type="submit"
              className="absolute inset-y-0 right-0 flex items-center pr-3"
            >
              <svg className="h-5 w-5 text-gray-400" fill="currentColor" viewBox="0 0 20 20">
                <path fillRule="evenodd" d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z" clipRule="evenodd" />
              </svg>
            </button>
          </div>
        </form>
      </div>

      {error && (
        <div className="bg-red-50 border border-red-200 rounded-md p-4 mb-6">
          <div className="text-red-800 text-sm">
            {error}
          </div>
        </div>
      )}

      {loading ? (
        <div className="flex justify-center items-center h-64">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
        </div>
      ) : (
        <>
          <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
            {vulnerabilities.map((vuln) => (
              <div key={vuln.id} className="bg-white rounded-xl shadow-lg overflow-hidden hover:shadow-xl transition-shadow duration-300 ease-in-out">
                <Link to={`/vulns/${vuln.id}`} className="block h-full">
                  <div className="p-6 h-full flex flex-col">
                    <div className="flex justify-between items-start">
                      <h3 className="text-lg font-bold text-gray-900 truncate">{vuln.title}</h3>
                      <span className={`ml-2 px-2 py-1 text-xs font-semibold rounded-full ${
                        vuln.severity === 'Critical' ? 'bg-red-100 text-red-800' :
                        vuln.severity === 'High' ? 'bg-orange-100 text-orange-800' :
                        vuln.severity === 'Medium' ? 'bg-yellow-100 text-yellow-800' :
                        'bg-green-100 text-green-800'
                      }`}>
                        {vuln.severity}
                      </span>
                    </div>

                    <p className="mt-3 text-gray-600 text-sm flex-grow">
                      {vuln.description.substring(0, 120)}...
                    </p>

                    <div className="mt-4 flex flex-wrap gap-2">
                      {vuln.tags && vuln.tags.slice(0, 3).map((tag, index) => (
                        <span key={index} className="px-2 py-1 bg-indigo-100 text-indigo-800 text-xs font-medium rounded-full">
                          {tag}
                        </span>
                      ))}
                      {vuln.tags && vuln.tags.length > 3 && (
                        <span className="px-2 py-1 bg-gray-100 text-gray-800 text-xs font-medium rounded-full">
                          +{vuln.tags.length - 3}
                        </span>
                      )}
                    </div>

                    <div className="mt-4 flex items-center justify-between text-sm">
                      <div className="flex items-center">
                        <svg className="h-4 w-4 text-gray-500 mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                        </svg>
                        <span className="text-gray-500">CVE: {vuln.cve || 'N/A'}</span>
                      </div>
                      <div className="flex items-center">
                        <svg className="h-4 w-4 text-gray-500 mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                        </svg>
                        <span className="text-gray-500">
                          {vuln.pushed ? '已推送' : '未推送'}
                        </span>
                      </div>
                    </div>

                    <div className="mt-3 text-xs text-gray-400">
                      更新时间: {new Date(vuln.updated_at).toLocaleDateString('zh-CN')}
                    </div>
                  </div>
                </Link>
              </div>
            ))}
          </div>

          {/* 分页组件 */}
          <div className="mt-6 flex items-center justify-between">
            <div className="text-sm text-gray-700">
              显示第 {(pageNo - 1) * pageSize + 1} 到 {Math.min(pageNo * pageSize, totalCount)} 条记录，
              总共 {totalCount} 条记录
            </div>
            <div className="flex space-x-2">
              <button
                onClick={() => handlePageChange(pageNo - 1)}
                disabled={pageNo === 1}
                className={`px-4 py-2 text-sm font-medium rounded-md ${
                  pageNo === 1
                    ? 'bg-gray-100 text-gray-400 cursor-not-allowed'
                    : 'bg-white text-gray-700 hover:bg-gray-50 border border-gray-300'
                }`}
              >
                上一页
              </button>
              <span className="px-4 py-2 text-sm text-gray-700">
                第 {pageNo} 页，共 {totalPages} 页
              </span>
              <button
                onClick={() => handlePageChange(pageNo + 1)}
                disabled={pageNo === totalPages}
                className={`px-4 py-2 text-sm font-medium rounded-md ${
                  pageNo === totalPages
                    ? 'bg-gray-100 text-gray-400 cursor-not-allowed'
                    : 'bg-white text-gray-700 hover:bg-gray-50 border border-gray-300'
                }`}
              >
                下一页
              </button>
            </div>
          </div>
        </>
      )}
    </div>
  )
}

export default VulnerabilityListPage
