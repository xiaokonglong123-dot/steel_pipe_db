import { Navigate, NavLink, Route, Routes, useLocation } from 'react-router-dom'

import { Loader, ROLE_LABEL } from './components/ui'
import AuditPage from './pages/AuditPage'
import CheckDetailPage from './pages/CheckDetailPage'
import ChecksPage from './pages/ChecksPage'
import ImportPage from './pages/ImportPage'
import LocationsPage from './pages/LocationsPage'
import LoginPage from './pages/LoginPage'
import StandardsPage from './pages/StandardsPage'
import StockInPage from './pages/StockInPage'
import TubeDetailPage from './pages/TubeDetailPage'
import TubesPage from './pages/TubesPage'
import UsersPage from './pages/UsersPage'
import { useSession } from './session'

const NAV = [
  { to: '/tubes', label: '管子' },
  { to: '/stock-in', label: '入库', write: true },
  { to: '/import', label: '导入', write: true },
  { to: '/locations', label: '库位' },
  { to: '/standards', label: '标准' },
  { to: '/checks', label: '盘点' },
  { to: '/users', label: '用户', admin: true },
  { to: '/audit', label: '审计', admin: true },
]

export default function App() {
  const { user, loading, isAdmin, canWrite, logout } = useSession()
  const location = useLocation()

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
      <header className="topbar">
        <span className="brand">SteelPipe</span>
        <nav className="nav">
          {NAV.filter((item) => (!item.admin || isAdmin) && (!item.write || canWrite)).map((item) => (
            <NavLink
              key={item.to}
              to={item.to}
              className={({ isActive }) => (isActive ? 'nav-item is-active' : 'nav-item')}
            >
              {item.label}
            </NavLink>
          ))}
        </nav>
        <span className="topbar-user">
          {user.display_name} · {ROLE_LABEL(user.role)}
          <button className="btn btn-ghost btn-sm" onClick={() => void logout()}>
            退出
          </button>
        </span>
      </header>

      <main className="page">
        <Routes>
          <Route path="/" element={<Navigate to="/tubes" replace />} />
          <Route path="/login" element={<Navigate to="/tubes" replace />} />
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
          <Route path="*" element={<Navigate to="/tubes" replace />} />
        </Routes>
      </main>
    </div>
  )
}
