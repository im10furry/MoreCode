import { useState, useEffect } from 'react'
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
  approvals: Approval[]
  patches: Patch[]
  commands: Command[]
  steps?: Step[]
  events?: RunEvent[]
}

interface Approval {
  approval_id: string
  title: string
  reason: string
  status: string
  choice?: string
}

interface Patch {
  patch_id: string
  file_path: string
  kind: string
  status: string
  patch_preview: string
  rationale?: string
}

interface Command {
  command_id: string
  command: string
  title: string
  status: string
  exit_code?: number
  stdout_tail?: string
  stderr_tail?: string
}

interface Step {
  step_id: string
  title: string
  status: string
  summary?: string
  token_used: number
}

interface RunEvent {
  sequence: number
  event: any
}

function statusLabel(status: string): string {
  switch (status.toLowerCase()) {
    case 'running': return 'running'
    case 'completed':
    case 'done': return 'completed'
    case 'failed':
    case 'error': return 'failed'
    case 'pending': return 'pending'
    case 'skipped': return 'pending'
    default: return 'pending'
  }
}

function statusBadgeClass(status: string): string {
  const s = statusLabel(status)
  if (s === 'running') return 'badge badge-info'
  if (s === 'completed') return 'badge badge-success'
  if (s === 'failed') return 'badge badge-error'
  return 'badge badge-muted'
}

function formatTime(iso: string): string {
  try {
    const d = new Date(iso)
    return d.toLocaleString(undefined, {
      month: 'short', day: 'numeric',
      hour: '2-digit', minute: '2-digit',
    })
  } catch {
    return iso
  }
}

function formatTokens(n: number): string {
  if (n >= 1000000) return (n / 1000000).toFixed(1) + 'M'
  if (n >= 1000) return (n / 1000).toFixed(1) + 'k'
  return n.toString()
}

/* ===== Step icon based on step_id ===== */
function stepIcon(stepId: string, st: string): string {
  const s = statusLabel(st)
  if (s === 'running') return '\u25CF'
  if (s === 'failed') return '\u2716'
  if (s === 'completed') return '\u2714'
  if (stepId.includes('explorer') || stepId.includes('impact')) return '\u25CB'
  if (stepId.includes('plan')) return '\u25A3'
  if (stepId.includes('code') || stepId.includes('coder')) return '\u25A0'
  if (stepId.includes('review')) return '\u25C9'
  if (stepId.includes('test')) return '\u25B6'
  return '\u25CF'
}

function stepIconColor(stepId: string, st: string): string {
  const s = statusLabel(st)
  if (s === 'running') return 'var(--accent-blue)'
  if (s === 'failed') return 'var(--accent-red)'
  if (s === 'completed') return 'var(--accent-green)'
  if (stepId.includes('explorer')) return 'var(--syntax-variable)'
  if (stepId.includes('impact')) return 'var(--syntax-function)'
  if (stepId.includes('plan')) return 'var(--accent-purple)'
  if (stepId.includes('code')) return 'var(--accent-teal)'
  if (stepId.includes('review')) return 'var(--accent-yellow)'
  if (stepId.includes('test')) return 'var(--accent-orange)'
  return 'var(--text-tertiary)'
}

/* ===== Main Component ===== */
function Runs() {
  const { runId } = useParams<{ runId: string }>()
  const navigate = useNavigate()
  const [runs, setRuns] = useState<RunSummary[]>([])
  const [selectedRun, setSelectedRun] = useState<RunDetail | null>(null)
  const [activeTab, setActiveTab] = useState<'overview' | 'steps' | 'approvals' | 'patches' | 'commands'>('overview')
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    fetchRuns()
  }, [])

  useEffect(() => {
    if (runId) {
      fetchRunDetail(runId)
      setActiveTab('overview')
    }
  }, [runId])

  const fetchRuns = async () => {
    try {
      const resp = await fetch('/api/runs')
      if (!resp.ok) throw new Error('Failed to fetch runs')
      const data = await resp.json()
      setRuns(Array.isArray(data) ? data : data.runs ?? [])
      setLoading(false)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch runs')
      setLoading(false)
    }
  }

  const fetchRunDetail = async (id: string) => {
    try {
      const resp = await fetch(`/api/runs/${id}`)
      if (!resp.ok) throw new Error('Failed to fetch run detail')
      const data = await resp.json()
      setSelectedRun(data)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch run detail')
    }
  }

  if (loading) return <div className="loading">Loading</div>
  if (error) return (
    <div className="empty-state">
      <div className="empty-state-icon">!</div>
      <div className="empty-state-title">Error</div>
      <div className="empty-state-hint">{error}</div>
    </div>
  )

  /* ===== Split layout: run list + detail ===== */
  return (
    <div className="run-detail">
      {/* ===== Run list sidebar ===== */}
      <div className="run-sidebar">
        <div className="run-sidebar-header">Runs</div>
        <div className="run-list" style={{ flex: 1, overflowY: 'auto' }}>
          {runs.map((run) => (
            <div
              key={run.run_id}
              className={`run-item${selectedRun?.run_id === run.run_id ? ' active' : ''}`}
              onClick={() => navigate(`/runs/${run.run_id}`)}
            >
              <span className={`status-dot ${statusLabel(run.status)}`} />
              <span className="run-item-request">{run.request}</span>
              <span className="run-item-meta">{formatTokens(run.total_tokens)}</span>
            </div>
          ))}
          {runs.length === 0 && (
            <div style={{ padding: 'var(--space-5)', color: 'var(--text-tertiary)', fontSize: 'var(--text-sm)' }}>
              No runs yet
            </div>
          )}
        </div>
      </div>

      {/* ===== Run detail content ===== */}
      <div className="run-content">
        {selectedRun ? (
          <>
            {/* Summary bar */}
            <div className="summary-bar">
              <span className={`status-dot ${statusLabel(selectedRun.status)}`} />
              <span className="summary-bar-item" style={{ fontWeight: 600, color: 'var(--text-primary)' }}>
                {selectedRun.request}
              </span>
              <span className="summary-bar-item">
                <span className={`badge ${statusBadgeClass(selectedRun.status).split(' ')[1]}`}>
                  {selectedRun.status}
                </span>
              </span>
              <span className="summary-bar-item">
                <span className="summary-bar-label">Tokens</span>
                {formatTokens(selectedRun.total_tokens)}
              </span>
              <span className="summary-bar-item">
                <span className="summary-bar-label">Started</span>
                {formatTime(selectedRun.started_at)}
              </span>
              {selectedRun.patches && (
                <span className="summary-bar-item">
                  <span className="summary-bar-label">Patches</span>
                  {selectedRun.patches.length}
                </span>
              )}
            </div>

            {/* Tabs */}
            <div className="run-tabs">
              {(['overview', 'steps', 'approvals', 'patches', 'commands'] as const).map((tab) => (
                <div
                  key={tab}
                  className={`run-tab${activeTab === tab ? ' active' : ''}`}
                  onClick={() => setActiveTab(tab)}
                >
                  {tab.charAt(0).toUpperCase() + tab.slice(1)}
                  {tab === 'steps' && selectedRun.steps && (
                    <span className="run-tab-count">{selectedRun.steps.length}</span>
                  )}
                  {tab === 'approvals' && (
                    <span className="run-tab-count">{selectedRun.approvals?.length ?? 0}</span>
                  )}
                  {tab === 'patches' && (
                    <span className="run-tab-count">{selectedRun.patches?.length ?? 0}</span>
                  )}
                </div>
              ))}
            </div>

            {/* Tab body */}
            <div className="run-tab-body">
              {activeTab === 'overview' && <OverviewTab run={selectedRun} />}
              {activeTab === 'steps' && <StepsTab run={selectedRun} />}
              {activeTab === 'approvals' && <ApprovalsTab run={selectedRun} />}
              {activeTab === 'patches' && <PatchesTab run={selectedRun} />}
              {activeTab === 'commands' && <CommandsTab run={selectedRun} />}
            </div>
          </>
        ) : (
          <div className="empty-state">
            <div className="empty-state-icon">{'\u25B8'}</div>
            <div className="empty-state-title">Select a run</div>
            <div className="empty-state-hint">Choose a run from the sidebar to view details</div>
          </div>
        )}
      </div>
    </div>
  )
}

/* ===== Tab: Overview ===== */
function OverviewTab({ run }: { run: RunDetail }) {
  const statusColor = statusLabel(run.status) === 'running' ? 'var(--accent-blue)'
    : statusLabel(run.status) === 'completed' ? 'var(--accent-green)'
    : statusLabel(run.status) === 'failed' ? 'var(--accent-red)'
    : 'var(--text-tertiary)'

  return (
    <div style={{ padding: 'var(--space-5)' }}>
      <div className="detail-card">
        <div className="detail-card-header">Status</div>
        <div className="detail-card-body">
          <div style={{ display: 'flex', gap: 'var(--space-6)', flexWrap: 'wrap' }}>
            <div>
              <div className="summary-bar-label">Status</div>
              <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--space-2)', marginTop: 'var(--space-1)' }}>
                <span className="status-dot" style={{ background: statusColor }} />
                <span style={{ color: statusColor, fontWeight: 500 }}>{run.status}</span>
              </div>
            </div>
            <div>
              <div className="summary-bar-label">Tokens</div>
              <div style={{ fontFamily: 'var(--font-mono)', marginTop: 'var(--space-1)' }}>
                {formatTokens(run.total_tokens)}
              </div>
            </div>
            <div>
              <div className="summary-bar-label">Started</div>
              <div style={{ marginTop: 'var(--space-1)' }}>{formatTime(run.started_at)}</div>
            </div>
            <div>
              <div className="summary-bar-label">Patches</div>
              <div style={{ fontFamily: 'var(--font-mono)', marginTop: 'var(--space-1)' }}>
                {run.patches?.length ?? 0}
              </div>
            </div>
            <div>
              <div className="summary-bar-label">Approvals</div>
              <div style={{ fontFamily: 'var(--font-mono)', marginTop: 'var(--space-1)' }}>
                {run.approvals?.length ?? 0}
              </div>
            </div>
          </div>
        </div>
      </div>

      {run.steps && run.steps.length > 0 && (
        <div className="detail-card">
          <div className="detail-card-header">Recent Steps</div>
          <div className="detail-card-body" style={{ padding: 0 }}>
            <div className="step-list">
              {run.steps.slice(-6).map((step) => (
                <div key={step.step_id} className="step-item">
                  <span className="step-icon" style={{ color: stepIconColor(step.step_id, step.status) }}>
                    {stepIcon(step.step_id, step.status)}
                  </span>
                  <div className="step-body">
                    <div className="step-title">{step.title || step.step_id}</div>
                    {step.summary && <div className="step-summary">{step.summary}</div>}
                  </div>
                  <span className="step-tokens">{formatTokens(step.token_used)}</span>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

/* ===== Tab: Steps ===== */
function StepsTab({ run }: { run: RunDetail }) {
  const steps = run.steps ?? []
  if (steps.length === 0) {
    return (
      <div className="empty-state">
        <div className="empty-state-icon">{'\u25CB'}</div>
        <div className="empty-state-hint">No step data available</div>
      </div>
    )
  }

  return (
    <div className="step-list">
      {steps.map((step) => (
        <div key={step.step_id} className="step-item">
          <span className="step-icon" style={{ color: stepIconColor(step.step_id, step.status) }}>
            {stepIcon(step.step_id, step.status)}
          </span>
          <div className="step-body">
            <div className="step-title">
              {step.title || step.step_id}
              <span className={`badge ${statusBadgeClass(step.status)}`} style={{ marginLeft: 'var(--space-3)' }}>
                {step.status}
              </span>
            </div>
            {step.summary && <div className="step-summary">{step.summary}</div>}
          </div>
          <span className="step-tokens">{formatTokens(step.token_used)}</span>
        </div>
      ))}
    </div>
  )
}

/* ===== Tab: Approvals ===== */
function ApprovalsTab({ run }: { run: RunDetail }) {
  const approvals = run.approvals ?? []
  if (approvals.length === 0) {
    return (
      <div className="empty-state">
        <div className="empty-state-icon">{'\u2714'}</div>
        <div className="empty-state-hint">No approvals needed</div>
      </div>
    )
  }

  return (
    <div>
      {approvals.map((approval) => (
        <div key={approval.approval_id} className="approval-item">
          <span className="step-icon" style={{
            color: approval.status === 'approved' ? 'var(--accent-green)' :
                   approval.status === 'rejected' ? 'var(--accent-red)' : 'var(--text-tertiary)'
          }}>
            {approval.status === 'approved' ? '\u2714' :
             approval.status === 'rejected' ? '\u2716' : '\u25CF'}
          </span>
          <div className="approval-info">
            <div className="approval-title">{approval.title}</div>
            <div className="approval-reason">{approval.reason}</div>
          </div>
          <span className={`badge ${approval.status === 'approved' ? 'badge-success' :
            approval.status === 'rejected' ? 'badge-error' : 'badge-muted'}`}>
            {approval.status}
          </span>
        </div>
      ))}
    </div>
  )
}

/* ===== Tab: Patches ===== */
function PatchesTab({ run }: { run: RunDetail }) {
  const patches = run.patches ?? []
  const [expandedIdx, setExpandedIdx] = useState<number | null>(null)

  if (patches.length === 0) {
    return (
      <div className="empty-state">
        <div className="empty-state-icon">{'\u25A0'}</div>
        <div className="empty-state-hint">No patches generated</div>
      </div>
    )
  }

  return (
    <div>
      {patches.map((patch, i) => (
        <div key={patch.patch_id}>
          <div
            className="patch-item"
            onClick={() => setExpandedIdx(expandedIdx === i ? null : i)}
            style={{ cursor: 'pointer' }}
          >
            <span className="step-icon" style={{
              color: patch.status === 'accepted' ? 'var(--accent-green)' :
                     patch.status === 'rejected' ? 'var(--accent-red)' : 'var(--accent-yellow)'
            }}>
              {patch.status === 'accepted' ? '\u2714' :
               patch.status === 'rejected' ? '\u2716' : '\u25B8'}
            </span>
            <div className="patch-info">
              <div className="patch-file">{patch.file_path}</div>
              {patch.rationale && <div className="patch-status">{patch.rationale}</div>}
            </div>
            <span className={`badge ${patch.status === 'accepted' ? 'badge-success' :
              patch.status === 'rejected' ? 'badge-error' : 'badge-warning'}`}>
              {patch.status}
            </span>
          </div>
          {expandedIdx === i && patch.patch_preview && (
            <div className="patch-preview" style={{ margin: '0 var(--space-5) var(--space-4)' }}>
              {patch.patch_preview.split('\n').map((line, j) => {
                let cls = 'patch-context'
                if (line.startsWith('+')) cls = 'patch-add'
                else if (line.startsWith('-')) cls = 'patch-remove'
                return <div key={j} className={cls}>{line}</div>
              })}
            </div>
          )}
        </div>
      ))}
    </div>
  )
}

/* ===== Tab: Commands ===== */
function CommandsTab({ run }: { run: RunDetail }) {
  const commands = run.commands ?? []
  if (commands.length === 0) {
    return (
      <div className="empty-state">
        <div className="empty-state-icon">$</div>
        <div className="empty-state-hint">No commands executed</div>
      </div>
    )
  }

  return (
    <div style={{ padding: 'var(--space-5)' }}>
      {commands.map((cmd) => (
        <div key={cmd.command_id} className="detail-card" style={{ margin: '0 0 var(--space-5) 0' }}>
          <div className="detail-card-header">
            {cmd.title || cmd.command_id}
            <span className={`badge ${statusBadgeClass(cmd.status)}`} style={{ marginLeft: 'var(--space-3)' }}>
              {cmd.status}
            </span>
            {cmd.exit_code !== undefined && (
              <span style={{ marginLeft: 'auto', fontFamily: 'var(--font-mono)', fontSize: 'var(--text-xs)' }}>
                exit: {cmd.exit_code}
              </span>
            )}
          </div>
          <div className="detail-card-body" style={{ padding: 0 }}>
            <div className="command-block">
              <div className="command-prompt">
                <span className="command-text">{cmd.command}</span>
              </div>
            </div>
            {cmd.stdout_tail && (
              <div className="command-output" style={{ padding: 'var(--space-4) var(--space-5)' }}>
                {cmd.stdout_tail}
              </div>
            )}
            {cmd.stderr_tail && (
              <div className="command-output error" style={{ padding: 'var(--space-4) var(--space-5)' }}>
                {cmd.stderr_tail}
              </div>
            )}
          </div>
        </div>
      ))}
    </div>
  )
}

export default Runs
