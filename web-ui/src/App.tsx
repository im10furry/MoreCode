import { BrowserRouter as Router, Routes, Route, Link, useLocation } from 'react-router-dom'
import Runs from './pages/Runs'
import Chat from './pages/Chat'

const navItems = [
  { to: '/', label: 'Runs', icon: '\u25B8' },
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
          const isActive = item.to === '/'
            ? location.pathname === '/' || location.pathname.startsWith('/runs')
            : location.pathname.startsWith(item.to)
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

function App() {
  return (
    <Router>
      <div className="app-shell">
        <Sidebar />
        <main className="app-main">
          <Routes>
            <Route path="/" element={<Runs />} />
            <Route path="/chat" element={<Chat />} />
            <Route path="/runs/:runId" element={<Runs />} />
          </Routes>
        </main>
      </div>
    </Router>
  )
}

export default App
