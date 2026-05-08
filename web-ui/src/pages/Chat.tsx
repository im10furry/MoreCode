import { useState, useRef, useEffect } from 'react'

interface Message {
  id: string
  role: 'user' | 'assistant' | 'system'
  content: string
  timestamp: Date
  error?: boolean
  errorReason?: string
}

function formatTime(date: Date): string {
  return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })
}

function avatarChar(role: string): string {
  if (role === 'user') return 'U'
  if (role === 'assistant') return 'MC'
  return '!'
}

function usernameFor(role: string): string {
  if (role === 'user') return 'You'
  if (role === 'assistant') return 'MoreCode'
  return 'System'
}

function isNetworkError(msg: string): { reason: string } | null {
  const lower = msg.toLowerCase()
  if (lower.includes('failed to fetch')) return { reason: 'ERR_CONNECTION_REFUSED' }
  if (lower.includes('network')) return { reason: 'ERR_NETWORK' }
  if (lower.includes('timeout')) return { reason: 'ERR_TIMEOUT' }
  if (lower.includes('server returned 5')) return { reason: 'ERR_SERVER_ERROR' }
  if (lower.includes('server returned 4')) return { reason: 'ERR_BAD_REQUEST' }
  return null
}

function Chat() {
  const [messages, setMessages] = useState<Message[]>([])
  const [input, setInput] = useState('')
  const [loading, setLoading] = useState(false)
  const messagesEndRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!input.trim() || loading) return

    const userMessage: Message = {
      id: Date.now().toString(),
      role: 'user',
      content: input.trim(),
      timestamp: new Date(),
    }

    setMessages((prev) => [...prev, userMessage])
    setInput('')
    setLoading(true)

    try {
      const response = await fetch('/api/runs', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ request: userMessage.content }),
      })

      if (!response.ok) {
        const errData = await response.json().catch(() => ({}))
        throw new Error((errData as any).error || `Server returned ${response.status}`)
      }

      const data = await response.json()

      const assistantMessage: Message = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: data.run_id
          ? `Run started: ${data.run_id}\nNavigate to Runs to view progress.`
          : JSON.stringify(data, null, 2),
        timestamp: new Date(),
      }

      setMessages((prev) => [...prev, assistantMessage])
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'Failed to send message'
      const netInfo = isNetworkError(msg)
      const errorReason = netInfo ? `${netInfo.reason} — check backend service` : undefined

      const errorMessage: Message = {
        id: (Date.now() + 1).toString(),
        role: 'system',
        content: msg,
        timestamp: new Date(),
        error: true,
        errorReason,
      }
      setMessages((prev) => [...prev, errorMessage])
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="chat-container">
      <div className="chat-header">
        <div>
          <div className="chat-header-title">Chat</div>
          <div className="chat-header-subtitle">new session</div>
        </div>
        <div className="chat-header-actions">
          <button type="button" className="icon-button" aria-label="Clear session">🗑</button>
          <button type="button" className="icon-button" aria-label="More options">⋯</button>
        </div>
      </div>

      <div className="chat-messages">
        {messages.length > 0 && (
          <div className="chat-session-separator">
            <span>{formatTime(messages[0].timestamp)} — session start</span>
          </div>
        )}

        {messages.length === 0 && (
          <div className="empty-state">
            <div className="empty-state-icon">{'>'}</div>
            <div className="empty-state-title">MoreCode Chat</div>
            <div className="empty-state-hint">
              Send a request to start a new run
            </div>
          </div>
        )}

        {messages.map((msg) => (
          <div key={msg.id} className={`chat-msg ${msg.role}${msg.error ? ' error' : ''}`}>
            <div className="chat-msg-body">
              <div className="chat-msg-header">
                <div className={`chat-msg-avatar ${msg.role}`}>{avatarChar(msg.role)}</div>
                <div className="chat-msg-meta">
                  <span className="chat-msg-username">{usernameFor(msg.role)}</span>
                  <span className="chat-msg-time">{formatTime(msg.timestamp)}</span>
                </div>
              </div>
              <div className="chat-msg-text">{msg.content}</div>
              {msg.error && msg.errorReason && (
                <div className="chat-msg-reason">{msg.errorReason}</div>
              )}
            </div>
          </div>
        ))}

        {loading && (
          <div className="chat-msg assistant loading">
            <div className="chat-msg-body">
              <div className="chat-msg-header">
                <div className="chat-msg-avatar assistant">MC</div>
                <div className="chat-msg-meta">
                  <span className="chat-msg-username">MoreCode</span>
                  <span className="chat-msg-time">{formatTime(new Date())}</span>
                </div>
              </div>
              <div className="chat-msg-text">Processing</div>
            </div>
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      <form className="chat-input-area" onSubmit={handleSubmit}>
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder="Describe what you want to build..."
          disabled={loading}
          autoFocus
        />
        <button type="submit" disabled={loading || !input.trim()}>
          ➤
        </button>
      </form>
    </div>
  )
}

export default Chat
