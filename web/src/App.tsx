import { useEffect, useState } from 'react'
import { Navigate, Route, Routes, useLocation } from 'react-router-dom'

import { CommandPalette, SideNav, Topbar, useStats } from './components/shell'
import { Loader } from './components/ui'
import AuditPage from './pages/AuditPage'
import CheckDetailPage from './pages/CheckDetailPage'
import ChecksPage from './pages/ChecksPage'
import DashboardPage from './pages/DashboardPage'
import ImportPage from './pages/ImportPage'
import LocationsPage from './pages/LocationsPage'
import LoginPage from './pages/LoginPage'
import StandardsPage from './pages/StandardsPage'
import StockInPage from './pages/StockInPage'
import TubeDetailPage from './pages/TubeDetailPage'
import TubesPage from './pages/TubesPage'
import UsersPage from './pages/UsersPage'
import { useSession } from './session'

function isTyping(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false
  const tag = target.tagName
  return tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || target.isContentEditable
}

export default function App() {
  const { user, loading, isAdmin, canWrite } = useSession()
  const location = useLocation()
  const stats = useStats()
  const [commandOpen, setCommandOpen] = useState(false)

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
        event.preventDefault()
        setCommandOpen((open) => !open)
        return
      }
      if (event.key === '/' && !isTyping(event.target)) {
        event.preventDefault()
        setCommandOpen(true)
        return
      }
      if (event.key === 'Escape') setCommandOpen(false)
    }
    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [])

  if (loading) return <Loader label="读取会话…" />

  if (!user) {
    return (
      <Routes>
        <Route path="/login" element={<LoginPage />} />
        <Route path="*" element={<Navigate to="/login" replace state={{ from: location.pathname }} />} />
      </Routes>
    )
  }

  return (
    <div className="app-shell">
      <Topbar onCommand={() => setCommandOpen(true)} />
      <div className="shell-body">
        <SideNav stats={stats} onCommand={() => setCommandOpen(true)} />
        <main className="page">
          <Routes>
            <Route path="/" element={<DashboardPage />} />
            <Route path="/login" element={<Navigate to="/" replace />} />
            <Route path="/tubes" element={<TubesPage />} />
            <Route path="/tubes/:pipeId" element={<TubeDetailPage />} />
            <Route path="/stock-in" element={canWrite ? <StockInPage /> : <Navigate to="/tubes" replace />} />
            <Route path="/import" element={canWrite ? <ImportPage /> : <Navigate to="/tubes" replace />} />
            <Route path="/locations" element={<LocationsPage />} />
            <Route path="/standards" element={<StandardsPage />} />
            <Route path="/checks" element={<ChecksPage />} />
            <Route path="/checks/:id" element={<CheckDetailPage />} />
            <Route path="/users" element={isAdmin ? <UsersPage /> : <Navigate to="/tubes" replace />} />
            <Route path="/audit" element={isAdmin ? <AuditPage /> : <Navigate to="/tubes" replace />} />
            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
        </main>
      </div>
      <CommandPalette open={commandOpen} onClose={() => setCommandOpen(false)} />
    </div>
  )
}
