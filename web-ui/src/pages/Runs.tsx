import React, { useState, useEffect } from 'react'
import { useParams, useNavigate } from 'react-router-dom'

interface RunSummary {
  run_id: string
  request: string
  status: string
  started_at: string
  total_tokens: number
  changed_files: string[]
}

interface RunDetail {
  run_id: string
  request: string
  status: string
  started_at: string
  total_tokens: number
  approvals: any[]
  patches: any[]
  commands: any[]
}

function Runs() {
  const { runId } = useParams<{ runId: string }>()
  const navigate = useNavigate()
  const [runs, setRuns] = useState<RunSummary[]>([])
  const [selectedRun, setSelectedRun] = useState<RunDetail | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    fetchRuns()
  }, [])

  useEffect(() => {
    if (runId) {
      fetchRunDetail(runId)
    }
  }, [runId])

  const fetchRuns = async () => {
    try {
      const response = await fetch('/api/runs')
      if (!response.ok) throw new Error('Failed to fetch runs')
      const data = await response.json()
      setRuns(data)
      setLoading(false)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch runs')
      setLoading(false)
    }
  }

  const fetchRunDetail = async (id: string) => {
    try {
      const response = await fetch(`/api/runs/${id}`)
      if (!response.ok) throw new Error('Failed to fetch run detail')
      const data = await response.json()
      setSelectedRun(data)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch run detail')
    }
  }

  const getStatusClass = (status: string) => {
    switch (status.toLowerCase()) {
      case 'running':
        return 'status-running'
      case 'completed':
        return 'status-completed'
      case 'failed':
        return 'status-failed'
      default:
        return 'status-pending'
    }
  }

  if (loading) {
    return <div className="container" style={{ padding: '20px' }}>Loading...</div>
  }

  if (error) {
    return <div className="container" style={{ padding: '20px', color: 'var(--error)' }}>Error: {error}</div>
  }

  return (
    <div className="main">
      <div className="sidebar">
        <h3 style={{ marginBottom: '16px' }}>Runs</h3>
        <div className="run-list">
          {runs.map((run) => (
            <div
              key={run.run_id}
              className={`run-card ${selectedRun?.run_id === run.run_id ? 'active' : ''}`}
              onClick={() => navigate(`/runs/${run.run_id}`)}
            >
              <div className="run-header">
                <span className="run-title">{run.request}</span>
                <span className={`run-status ${getStatusClass(run.status)}`}>
                  {run.status}
                </span>
              </div>
              <div className="run-meta">
                {run.total_tokens} tokens • {run.changed_files?.length || 0} files
              </div>
              <div className="progress-bar">
                <div
                  className="progress-fill"
                  style={{ width: `${run.status === 'completed' ? 100 : run.status === 'running' ? 50 : 0}%` }}
                />
              </div>
            </div>
          ))}
        </div>
      </div>
      <div className="content">
        {selectedRun ? (
          <div>
            <h2 style={{ marginBottom: '16px' }}>{selectedRun.request}</h2>
            <div className="card">
              <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '8px' }}>
                <span>Status: <span className={`run-status ${getStatusClass(selectedRun.status)}`}>{selectedRun.status}</span></span>
                <span>Tokens: {selectedRun.total_tokens}</span>
              </div>
              <div className="run-meta">Started: {new Date(selectedRun.started_at).toLocaleString()}</div>
            </div>
            {selectedRun.approvals?.length > 0 && (
              <div className="card">
                <h3 style={{ marginBottom: '8px' }}>Approvals</h3>
                {selectedRun.approvals.map((approval, i) => (
                  <div key={i} style={{ padding: '8px 0', borderBottom: '1px solid var(--border)' }}>
                    <div>{approval.title}</div>
                    <div className="run-meta">Status: {approval.status}</div>
                  </div>
                ))}
              </div>
            )}
            {selectedRun.patches?.length > 0 && (
              <div className="card">
                <h3 style={{ marginBottom: '8px' }}>Patches</h3>
                {selectedRun.patches.map((patch, i) => (
                  <div key={i} style={{ padding: '8px 0', borderBottom: '1px solid var(--border)' }}>
                    <div>{patch.file_path}</div>
                    <div className="run-meta">Status: {patch.status}</div>
                  </div>
                ))}
              </div>
            )}
            {selectedRun.commands?.length > 0 && (
              <div className="card">
                <h3 style={{ marginBottom: '8px' }}>Commands</h3>
                {selectedRun.commands.map((cmd, i) => (
                  <div key={i} style={{ padding: '8px 0', borderBottom: '1px solid var(--border)' }}>
                    <div><code>{cmd.command}</code></div>
                    <div className="run-meta">Status: {cmd.status}</div>
                  </div>
                ))}
              </div>
            )}
          </div>
        ) : (
          <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100%', color: 'var(--text-secondary)' }}>
            Select a run from the sidebar
          </div>
        )}
      </div>
    </div>
  )
}

export default Runs