import { useState, useEffect } from 'react'
import { Link } from 'react-router-dom'
import { getVulnerabilities, getPlugins } from '../../lib/api'

const VulnerabilityListPage = () => {
  const [vulnerabilities, setVulnerabilities] = useState([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')
  const [pageNo, setPageNo] = useState(1)
  const [pageSize] = useState(10)
  const [totalCount, setTotalCount] = useState(0)
  const [plugins, setPlugins] = useState([])
  // 新的筛选条件状态
  const [filters, setFilters] = useState({
    cve: '',
    title: '',
    pushed: '',
    source: ''  // 存储插件的link字段
  })

  useEffect(() => {
    fetchPlugins()
  }, [])

  useEffect(() => {
    fetchVulnerabilities()
  }, [pageNo, filters])

  const fetchPlugins = async () => {
    try {
      const response = await getPlugins()
      setPlugins(response.data.data)
    } catch (err) {
      console.error('获取插件列表失败:', err)
    }
  }

  const fetchVulnerabilities = async () => {
    setLoading(true)
    setError('')

    try {
      const params = {
        pageNo,
        pageSize,
        ...filters
      }
      // 清理空值参数
      Object.keys(params).forEach(key => {
        if (params[key] === '' || params[key] === undefined) {
          delete params[key]
        }
      })

      const response = await getVulnerabilities(params)
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
const handleFilterChange = (e) => {
  const { name, value } = e.target
  setFilters(prev => ({
    ...prev,
    [name]: value
  }))
  // 重置页码到第一页
  setPageNo(1)
}

const handleSourceFilterChange = (e) => {
  const selectedLink = e.target.value
  setFilters(prev => ({
    ...prev,
    source: selectedLink
  }))
  // 重置页码到第一页
  setPageNo(1)
}


  const handleResetFilters = () => {
    setFilters({
      cve: '',
      title: '',
      pushed: '',
      source: ''
    })
    // 重置页码到第一页
    setPageNo(1)
  }

  const totalPages = Math.ceil(totalCount / pageSize)

  return (
    <div className="mx-auto max-w-7xl">
      <div className="mb-8 text-center">
        <h1 className="mb-2 text-3xl font-extrabold text-gray-900">
          漏洞信息列表
        </h1>
        <p className="text-gray-600">
          当前共有 {totalCount} 个漏洞信息
        </p>
      </div>

      {/* 筛选控件 */}
      <div className="mb-6">
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-4">
          <div>
            <label htmlFor="cve" className="block text-sm font-medium text-gray-700">
              CVE编号
            </label>
            <input
              type="text"
              name="cve"
              id="cve"
              value={filters.cve}
              onChange={handleFilterChange}
              placeholder="搜索CVE编号..."
              className="block w-full py-2 pl-4 pr-12 leading-5 placeholder-gray-500 bg-white border border-gray-300 rounded-md focus:outline-none focus:placeholder-gray-400 focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
            />
          </div>
          <div>
            <label htmlFor="title" className="block text-sm font-medium text-gray-700">
              漏洞标题
            </label>
            <input
              type="text"
              name="title"
              id="title"
              value={filters.title}
              onChange={handleFilterChange}
              placeholder="搜索漏洞标题..."
              className="block w-full py-2 pl-4 pr-12 leading-5 placeholder-gray-500 bg-white border border-gray-300 rounded-md focus:outline-none focus:placeholder-gray-400 focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
            />
          </div>
          <div>
            <label htmlFor="pushed" className="block text-sm font-medium text-gray-700">
              推送状态
            </label>
            <select
              name="pushed"
              id="pushed"
              value={filters.pushed}
              onChange={handleFilterChange}
              className="block w-full py-2 pl-4 pr-12 leading-5 placeholder-gray-500 bg-white border border-gray-300 rounded-md focus:outline-none focus:placeholder-gray-400 focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
            >
              <option value="">全部</option>
              <option value="true">已推送</option>
              <option value="false">未推送</option>
            </select>
          </div>
          <div>
            <label htmlFor="source" className="block text-sm font-medium text-gray-700">
              来源
            </label>
            <select
              name="source"
              id="source"
              value={filters.source}
              onChange={handleSourceFilterChange}
              className="block w-full py-2 pl-4 pr-12 leading-5 placeholder-gray-500 bg-white border border-gray-300 rounded-md focus:outline-none focus:placeholder-gray-400 focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
            >
              <option value="">全部</option>
              {plugins.map((plugin) => (
                <option key={plugin.link} value={plugin.link}>
                  {plugin.display_name}
                </option>
              ))}
            </select>
          </div>
        </div>
        <div className="flex justify-end mt-4">
          <button
            onClick={handleResetFilters}
            className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
          >
            重置筛选
          </button>
        </div>
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
      ) : (
        <>
          <div className="overflow-hidden bg-white shadow sm:rounded-md">
            <ul className="divide-y divide-gray-200">
              {vulnerabilities.map((vuln) => (
                <li key={vuln.id}>
                  <Link to={`/vulns/${vuln.id}`} className="block hover:bg-gray-50">
                    <div className="px-4 py-4 sm:px-6">
                      <div className="flex items-center justify-between">
                        <p className="text-sm font-medium text-indigo-600 truncate">{vuln.title}</p>
                        <div className="flex flex-shrink-0 ml-2">
                          <span className={`px-2 inline-flex text-xs leading-5 font-semibold rounded-full ${
                            vuln.severity === 'Critical' ? 'bg-red-100 text-red-800' :
                            vuln.severity === 'High' ? 'bg-orange-100 text-orange-800' :
                            vuln.severity === 'Medium' ? 'bg-yellow-100 text-yellow-800' :
                            'bg-green-100 text-green-800'
                          }`}>
                            {vuln.severity}
                          </span>
                        </div>
                      </div>
                      <div className="mt-2 sm:flex sm:justify-between">
                        <div className="sm:flex">
                          <p className="flex items-center text-sm text-gray-500">
                            CVE: {vuln.cve || 'N/A'}
                          </p>
                          <p className="flex items-center mt-1 text-sm text-gray-500 sm:mt-0 sm:ml-6">
                            来源: {vuln.source}
                          </p>
                          <div className="flex flex-wrap gap-2 mt-2 sm:mt-0 sm:ml-6">
                            {vuln.tags && vuln.tags.slice(0, 5).map((tag, index) => (
                              <span key={index} className="px-2 py-1 text-xs font-medium text-indigo-800 bg-indigo-100 rounded-full">
                                {tag}
                              </span>
                            ))}
                            {vuln.tags && vuln.tags.length > 5 && (
                              <span className="px-2 py-1 text-xs font-medium text-gray-800 bg-gray-100 rounded-full">
                                +{vuln.tags.length - 5}
                              </span>
                            )}
                          </div>
                        </div>
                        <div className="flex items-center mt-2 text-sm text-gray-500 sm:mt-0">
                          <svg className="flex-shrink-0 mr-1.5 h-5 w-5 text-gray-400" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
                            <path fillRule="evenodd" d="M6 2a1 1 0 00-1 1v1H4a2 2 0 00-2 2v10a2 2 0 002 2h12a2 2 0 002-2V6a2 2 0 00-2-2h-1V3a1 1 0 10-2 0v1H7V3a1 1 0 00-1-1zm0 5a1 1 0 000 2h8a1 1 0 100-2H6z" clipRule="evenodd" />
                          </svg>
                          <p>
                            更新时间: <time dateTime={vuln.updated_at}>{new Date(vuln.updated_at).toLocaleDateString('zh-CN')}</time>
                          </p>
                          <span className={`ml-4 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                            vuln.pushed ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-800'
                          }`}>
                            {vuln.pushed ? '已推送' : '未推送'}
                          </span>
                        </div>
                      </div>
                      <div className="mt-2">
                        <p className="text-sm text-gray-500 line-clamp-2">
                          {vuln.description}
                        </p>
                      </div>
                    </div>
                  </Link>
                </li>
              ))}
            </ul>
          </div>

          {/* 分页组件 */}
          <div className="flex items-center justify-between mt-6">
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
