import React from 'react'
import { BrowserRouter as Router, Routes, Route, Link, useLocation } from 'react-router-dom'
import Runs from './pages/Runs'
import Chat from './pages/Chat'

function App() {
  return (
    <Router>
      <div className="header">
        <div className="container header-content">
          <Link to="/" className="logo">MoreCode</Link>
          <nav className="nav">
            <NavLink to="/">Runs</NavLink>
            <NavLink to="/chat">Chat</NavLink>
          </nav>
        </div>
      </div>
      <Routes>
        <Route path="/" element={<Runs />} />
        <Route path="/chat" element={<Chat />} />
        <Route path="/runs/:runId" element={<Runs />} />
      </Routes>
    </Router>
  )
}

function NavLink({ to, children }: { to: string; children: React.ReactNode }) {
  const location = useLocation()
  const isActive = location.pathname === to || (to !== '/' && location.pathname.startsWith(to))

  return (
    <Link to={to} className={isActive ? 'active' : ''}>
      {children}
    </Link>
  )
}

export default App