import { useEffect, useState } from 'react'
import { useNavigate, Navigate } from 'react-router-dom'

const ProtectedRoute = ({ children }) => {
  const [isAuthenticated, setIsAuthenticated] = useState(null)
  const navigate = useNavigate()

  useEffect(() => {
    // 检查用户是否已登录
    const token = localStorage.getItem('token')
    if (!token) {
      // 如果没有token，重定向到登录页面
      setIsAuthenticated(false)
    } else {
      // 如果有token，设置为已认证
      setIsAuthenticated(true)
    }
  }, [navigate])

  // 如果还在检查认证状态，显示加载状态
  if (isAuthenticated === null) {
    return <div className="flex justify-center items-center h-screen">
      <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
    </div>
  }

  // 如果未认证，重定向到登录页面
  if (!isAuthenticated) {
    return <Navigate to="/login" replace />
  }

  // 如果已认证，渲染子组件
  return children
}

export default ProtectedRoute
