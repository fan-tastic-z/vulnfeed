import { Routes, Route } from 'react-router-dom'
import LoginPage from './routes/auth/login'
import HomePage from './routes/home'
import VulnerabilityListPage from './routes/vuln/list'
import VulnerabilityDetailPage from './routes/vuln/detail'
import SyncDataTaskPage from './routes/sync/task'
import PluginListPage from './routes/plugins/list'
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
            <Layout>
              <VulnerabilityListPage />
            </Layout>
          }
        />
        <Route
          path="/vulns"
          element={
            <Layout>
              <VulnerabilityListPage />
            </Layout>
          }
        />
        <Route
          path="/vulns/:id"
          element={
            <Layout>
              <VulnerabilityDetailPage />
            </Layout>
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
            <Layout>
              <PluginListPage />
            </Layout>
          }
        />
      </Routes>
    </div>
  )
}

export default App
