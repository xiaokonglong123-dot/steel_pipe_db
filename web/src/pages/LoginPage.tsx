import { useState } from 'react'
import { useNavigate } from 'react-router-dom'

import { IconBox } from '../components/icons'
import { Field, Problem } from '../components/ui'
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
    <div className="login-shell">
      <aside className="login-visual">
        <div>
          <div className="login-mark">
            <IconBox size={26} />
          </div>
          <h1>SteelPipe</h1>
          <p>无缝钢管与筛管（API 5CT）单根级库存。合规校验、哈希审计链、盘点流转，全部落在一个二进制与一个 SQLite 文件里。</p>
        </div>
        <div className="login-facts">
          <div>
            <strong>单根级</strong>
            <span>每根管子独立状态与库位</span>
          </div>
          <div>
            <strong>API 5CT</strong>
            <span>钢级、壁厚、外径、长度校验</span>
          </div>
          <div>
            <strong>SHA-256</strong>
            <span>防篡改审计链</span>
          </div>
        </div>
      </aside>

      <main className="login-panel">
        <form className="login-card" onSubmit={submit}>
          <div>
            <h2>登录</h2>
            <p className="sub">使用局域网账号进入库存系统</p>
          </div>
          <Field label="用户名">
            <input
              className="input"
              autoComplete="username"
              autoFocus
              value={username}
              onChange={(event) => setUsername(event.target.value)}
              required
            />
          </Field>
          <Field label="口令">
            <input
              className="input"
              type="password"
              autoComplete="current-password"
              value={password}
              onChange={(event) => setPassword(event.target.value)}
              required
            />
          </Field>
          <Problem error={error} />
          <button className="btn btn-accent" type="submit" disabled={busy}>
            {busy ? '登录中…' : '登录'}
          </button>
          <p className="hint" style={{ textAlign: 'center' }}>
            连续失败 10 次将锁定 300 秒
          </p>
        </form>
      </main>
    </div>
  )
}