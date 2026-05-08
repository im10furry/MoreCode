import { BrowserRouter as Router, Routes, Route, Link, useLocation, Navigate } from 'react-router-dom'
import Runs from './pages/Runs'
import Chat from './pages/Chat'

const navItems = [
  { to: '/runs', label: 'Runs', icon: '\u25B8' },
  { to: '/chat', label: 'Chat', icon: '>' },
]

function Sidebar() {
  const location = useLocation()

  return (
    <aside className="sidebar">
      <div className="sidebar-header">
        <span className="sidebar-logo">MC</span>
        <span className="sidebar-title">MoreCode</span>
      </div>
      <nav className="sidebar-nav">
        {navItems.map((item) => {
          const isActive = location.pathname.startsWith(item.to)
          return (
            <Link
              key={item.to}
              to={item.to}
              className={`sidebar-item${isActive ? ' active' : ''}`}
            >
              <span className="sidebar-item-icon">{item.icon}</span>
              <span className="sidebar-item-label">{item.label}</span>
            </Link>
          )
        })}
      </nav>
      <div className="sidebar-footer">
        <span className="sidebar-footer-text">MoreCode v1.0</span>
      </div>
    </aside>
  )
}

function StatusBar() {
  return (
    <div className="status-bar">
      <div className="status-bar-left">
        <span className="status-bar-item">
          <span className="status-bar-dot online" />
          connected
        </span>
        <span className="status-bar-item">MoreCode v1.0</span>
      </div>
      <div className="status-bar-right">
        <span className="status-bar-item">localhost:3001</span>
        <span className="status-bar-item">UTF-8</span>
      </div>
    </div>
  )
}

function App() {
  return (
    <Router>
      <div className="app-shell">
        <div className="app-body">
          <Sidebar />
          <main className="app-main">
            <Routes>
              <Route path="/" element={<Navigate to="/chat" replace />} />
              <Route path="/chat" element={<Chat />} />
              <Route path="/runs" element={<Runs />} />
              <Route path="/runs/:runId" element={<Runs />} />
            </Routes>
          </main>
        </div>
        <StatusBar />
      </div>
    </Router>
  )
}

export default App
