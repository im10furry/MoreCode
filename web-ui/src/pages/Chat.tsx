import { useState, useRef, useEffect } from 'react'

interface Message {
  id: string
  role: 'user' | 'assistant' | 'system'
  content: string
  timestamp: Date
}

function formatTime(date: Date): string {
  return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}

function avatarChar(role: string): string {
  if (role === 'user') return 'U'
  if (role === 'assistant') return 'MC'
  return '!'
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

      if (!response.ok) throw new Error(`Server returned ${response.status}`)

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
      const errorMessage: Message = {
        id: (Date.now() + 1).toString(),
        role: 'system',
        content: err instanceof Error ? err.message : 'Failed to send message',
        timestamp: new Date(),
      }
      setMessages((prev) => [...prev, errorMessage])
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="chat-container">
      <div className="chat-messages">
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
          <div key={msg.id} className={`chat-msg ${msg.role}`}>
            <div className="chat-msg-avatar">{avatarChar(msg.role)}</div>
            <div>
              <div className="chat-msg-bubble">
                {msg.content}
              </div>
              <div className="chat-msg-time">{formatTime(msg.timestamp)}</div>
            </div>
          </div>
        ))}

        {loading && (
          <div className="chat-msg assistant">
            <div className="chat-msg-avatar" style={{ background: 'var(--accent-green)', color: '#111' }}>MC</div>
            <div>
              <div className="chat-msg-bubble" style={{
                background: 'var(--bg-card)',
                border: '1px solid var(--border-subtle)',
                padding: 'var(--space-3) var(--space-5)',
                borderRadius: 'var(--radius-md)',
              }}>
                <span className="loading" style={{
                  display: 'inline-flex',
                  alignItems: 'center',
                  gap: 'var(--space-3)',
                  color: 'var(--text-tertiary)',
                  fontSize: 'var(--text-sm)',
                  height: 'auto',
                }}>
                  Processing
                </span>
              </div>
            </div>
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      <form className="chat-input-area" onSubmit={handleSubmit}>
        <span className="prompt-sign">$</span>
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder="Describe what you want to build..."
          disabled={loading}
          autoFocus
        />
        <button type="submit" disabled={loading || !input.trim()}>
          {loading ? '...' : 'Run'}
        </button>
      </form>
    </div>
  )
}

export default Chat
