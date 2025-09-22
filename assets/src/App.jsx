import { Routes, Route } from 'react-router-dom'
import LoginPage from './routes/auth/login'
import HomePage from './routes/home'
import VulnerabilityListPage from './routes/vuln/list'
import VulnerabilityDetailPage from './routes/vuln/detail'
import SyncDataTaskPage from './routes/sync/task'
import PluginListPage from './routes/plugins/list'
import DingBotConfigPage from './routes/dingbot/config'
import SecNoticeListPage from './routes/secnotice/list'
import ProtectedRoute from './components/ProtectedRoute'
import Layout from './components/Layout'

function App() {
  return (
    <div>
      <Routes>
        <Route path="/login" element={<LoginPage />} />
        <Route
          path="/"
          element={
            <ProtectedRoute>
              <Layout>
                <VulnerabilityListPage />
              </Layout>
            </ProtectedRoute>
          }
        />
        <Route
          path="/vulns"
          element={
            <ProtectedRoute>
              <Layout>
                <VulnerabilityListPage />
              </Layout>
            </ProtectedRoute>
          }
        />
        <Route
          path="/vulns/:id"
          element={
            <ProtectedRoute>
              <Layout>
                <VulnerabilityDetailPage />
              </Layout>
            </ProtectedRoute>
          }
        />
        <Route
          path="/sync/task"
          element={
            <ProtectedRoute>
              <Layout>
                <SyncDataTaskPage />
              </Layout>
            </ProtectedRoute>
          }
        />
        <Route
          path="/plugins"
          element={
            <ProtectedRoute>
              <Layout>
                <PluginListPage />
              </Layout>
            </ProtectedRoute>
          }
        />
        <Route
          path="/dingbot/config"
          element={
            <ProtectedRoute>
              <Layout>
                <DingBotConfigPage />
              </Layout>
            </ProtectedRoute>
          }
        />
        <Route
          path="/secnotice"
          element={
            <ProtectedRoute>
              <Layout>
                <SecNoticeListPage />
              </Layout>
            </ProtectedRoute>
          }
        />
      </Routes>
    </div>
  )
}

export default App
