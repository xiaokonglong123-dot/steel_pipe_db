import { useState } from 'react'
import { useNavigate } from 'react-router-dom'

import { Problem } from '../components/ui'
import { useSession } from '../session'

export default function LoginPage() {
  const { login } = useSession()
  const navigate = useNavigate()
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState<unknown>(null)
  const [busy, setBusy] = useState(false)

  async function submit(event: React.FormEvent) {
    event.preventDefault()
    setBusy(true)
    setError(null)
    try {
      await login(username.trim(), password)
      navigate('/tubes', { replace: true })
    } catch (cause) {
      setError(cause)
    } finally {
      setBusy(false)
    }
  }

  return (
    <main className="login-shell">
      <form className="login-card" onSubmit={submit}>
        <h1>SteelPipe</h1>
        <p className="muted">无缝钢管与筛管库存管理系统</p>
        <label className="field">
          <span className="field-label">用户名</span>
          <input
            className="input"
            autoComplete="username"
            value={username}
            onChange={(event) => setUsername(event.target.value)}
            required
          />
        </label>
        <label className="field">
          <span className="field-label">口令</span>
          <input
            className="input"
            type="password"
            autoComplete="current-password"
            value={password}
            onChange={(event) => setPassword(event.target.value)}
            required
          />
        </label>
        <Problem error={error} />
        <button className="btn btn-accent" type="submit" disabled={busy}>
          {busy ? '登录中…' : '登录'}
        </button>
        <p className="field-hint">连续失败 10 次将锁定 300 秒。</p>
      </form>
    </main>
  )
}
