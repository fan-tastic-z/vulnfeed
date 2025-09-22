import { Link, useLocation } from 'react-router-dom'
import { useState, useRef, useEffect } from 'react'

const Navigation = () => {
  const location = useLocation()
  const [userMenuOpen, setUserMenuOpen] = useState(false)
  const userMenuRef = useRef(null)

  // 检查用户是否已登录
  const [isAuthenticated, setIsAuthenticated] = useState(false)

  useEffect(() => {
    const token = localStorage.getItem('token')
    setIsAuthenticated(!!token)
  }, [])

  const isActive = (path) => {
    return location.pathname === path
  }

  const handleLogout = () => {
    // 清除token
    localStorage.removeItem('token')
    // 重定向到登录页面
    window.location.href = '/login'
  }

  // 点击外部关闭用户菜单
  useEffect(() => {
    const handleClickOutside = (event) => {
      if (userMenuRef.current && !userMenuRef.current.contains(event.target)) {
        setUserMenuOpen(false)
      }
    }

    document.addEventListener('mousedown', handleClickOutside)
    return () => {
      document.removeEventListener('mousedown', handleClickOutside)
    }
  }, [])

  return (
    <nav className="bg-white shadow">
      <div className="px-4 mx-auto max-w-7xl sm:px-6 lg:px-8">
        <div className="flex justify-between h-16">
          <div className="flex">
            <div className="flex items-center flex-shrink-0">
              <span className="text-xl font-bold text-indigo-600">VulnFeed</span>
            </div>
            <div className="hidden sm:ml-6 sm:flex sm:space-x-8">
              <Link
                to="/vulns"
                className={`${
                  isActive('/vulns') || location.pathname === '/'
                    ? 'border-indigo-500 text-gray-900'
                    : 'border-transparent text-gray-500 hover:border-gray-300 hover:text-gray-700'
                } inline-flex items-center px-1 pt-1 border-b-2 text-sm font-medium`}
              >
                漏洞列表
              </Link>
              <Link
                to="/secnotice"
                className={`${
                  isActive('/secnotice')
                    ? 'border-indigo-500 text-gray-900'
                    : 'border-transparent text-gray-500 hover:border-gray-300 hover:text-gray-700'
                } inline-flex items-center px-1 pt-1 border-b-2 text-sm font-medium`}
              >
                安全公告
              </Link>
              <Link
                to="/plugins"
                className={`${
                  isActive('/plugins')
                    ? 'border-indigo-500 text-gray-900'
                    : 'border-transparent text-gray-500 hover:border-gray-300 hover:text-gray-700'
                } inline-flex items-center px-1 pt-1 border-b-2 text-sm font-medium`}
              >
                插件管理
              </Link>
              {isAuthenticated && (
                <Link
                  to="/sync/task"
                  className={`${
                    isActive('/sync/task')
                      ? 'border-indigo-500 text-gray-900'
                      : 'border-transparent text-gray-500 hover:border-gray-300 hover:text-gray-700'
                  } inline-flex items-center px-1 pt-1 border-b-2 text-sm font-medium`}
                >
                  同步任务
                </Link>
              )}
              {isAuthenticated && (
                <Link
                  to="/dingbot/config"
                  className={`${
                    isActive('/dingbot/config')
                      ? 'border-indigo-500 text-gray-900'
                      : 'border-transparent text-gray-500 hover:border-gray-300 hover:text-gray-700'
                  } inline-flex items-center px-1 pt-1 border-b-2 text-sm font-medium`}
                >
                  钉钉推送
                </Link>
              )}
            </div>
          </div>
          {isAuthenticated ? (
            <div className="hidden sm:ml-6 sm:flex sm:items-center">
              <div className="relative ml-3">
                <div>
                  <button
                    onClick={() => setUserMenuOpen(!userMenuOpen)}
                    className="flex text-sm rounded-full focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
                  >
                    <span className="sr-only">打开用户菜单</span>
                    <div className="flex items-center justify-center w-8 h-8 bg-indigo-100 rounded-full">
                      <span className="font-medium text-indigo-800">U</span>
                    </div>
                  </button>
                </div>
                {userMenuOpen && (
                  <div
                    ref={userMenuRef}
                    className="absolute right-0 w-48 py-1 mt-2 origin-top-right bg-white rounded-md shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none"
                  >
                    <button
                      onClick={handleLogout}
                      className="block w-full px-4 py-2 text-sm text-left text-gray-700 hover:bg-gray-100"
                    >
                      登出
                    </button>
                  </div>
                )}
              </div>
            </div>
          ) : (
            <div className="hidden sm:ml-6 sm:flex sm:items-center">
              <Link
                to="/login"
                className="inline-flex items-center px-4 py-2 text-sm font-medium text-white bg-indigo-600 border border-transparent rounded-md shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
              >
                登录
              </Link>
            </div>
          )}
        </div>
      </div>
    </nav>
  )
}

export default Navigation
