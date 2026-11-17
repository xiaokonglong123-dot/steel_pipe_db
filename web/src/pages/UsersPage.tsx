import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import { api, type Schemas } from '../api/client'
import {
  Badge,
  Empty,
  Field,
  Loader,
  PageHead,
  Problem,
  PromptDialog,
  ROLE_LABEL,
  useToast,
} from '../components/ui'
import { fmtTime } from '../lib/format'
import { useSession } from '../session'

type User = Schemas['UserDto']

export default function UsersPage() {
  const { isAdmin } = useSession()
  const queryClient = useQueryClient()
  const { push } = useToast()
  const [error, setError] = useState<unknown>(null)
  const [form, setForm] = useState({
    username: '',
    password: '',
    display_name: '',
    role: 'operator' as User['role'],
  })
  const [resetFor, setResetFor] = useState<User | null>(null)

  const list = useQuery({
    queryKey: ['users'],
    queryFn: () => api<Schemas['UserListResponse']>('/api/users'),
  })

  const invalidate = () => void queryClient.invalidateQueries({ queryKey: ['users'] })

  const create = useMutation({
    mutationFn: () =>
      api<User>('/api/users', {
        method: 'POST',
        body: {
          username: form.username.trim(),
          password: form.password,
          display_name: form.display_name.trim() || form.username.trim(),
          role: form.role,
        },
      }),
    onSuccess: () => {
      push('ok', `已创建用户 ${form.username.trim()}`)
      setForm({ username: '', password: '', display_name: '', role: 'operator' })
      setError(null)
      invalidate()
    },
    onError: (cause) => setError(cause),
  })

  const toggle = useMutation({
    mutationFn: (row: User) =>
      api(`/api/users/${row.id}/active`, { method: 'POST', body: { active: !row.active } }),
    onSuccess: (_data, row) => {
      push('ok', row.active ? `已停用 ${row.username}` : `已启用 ${row.username}`)
      setError(null)
      invalidate()
    },
    onError: (cause) => setError(cause),
  })

  const revoke = useMutation({
    mutationFn: (row: User) =>
      api<Schemas['RevokeResponse']>(`/api/users/${row.id}/revoke`, { method: 'POST', body: {} }),
    onSuccess: (_data, row) => {
      push('ok', `已吊销 ${row.username} 的全部会话`)
      setError(null)
      invalidate()
    },
    onError: (cause) => setError(cause),
  })

  const resetPassword = useMutation({
    mutationFn: ({ row, password }: { row: User; password: string }) =>
      api(`/api/users/${row.id}/password`, { method: 'POST', body: { password } }),
    onSuccess: (_data, vars) => {
      push('ok', `已重置 ${vars.row.username} 的口令`)
      setResetFor(null)
      setError(null)
      invalidate()
    },
    onError: (cause) => setError(cause),
  })

  if (!isAdmin) return <Problem error={new Error('仅管理员可访问')} />

  const rows = list.data?.users ?? []

  return (
    <section>
      <PageHead title="用户" sub="停用与改密都会递增该用户 token_version，旧 token 立即失效" />
      <Problem error={error} />

      <form
        className="card"
        style={{ marginBottom: 16 }}
        onSubmit={(event) => {
          event.preventDefault()
          create.mutate()
        }}
      >
        <div className="card-head">
          <h2>新建用户</h2>
        </div>
        <div className="card-pad form-grid">
          <Field label="用户名">
            <input
              className="input"
              value={form.username}
              onChange={(event) => setForm({ ...form, username: event.target.value })}
              required
            />
          </Field>
          <Field label="显示名">
            <input
              className="input"
              value={form.display_name}
              onChange={(event) => setForm({ ...form, display_name: event.target.value })}
            />
          </Field>
          <Field label="初始口令" hint="至少 8 位">
            <input
              className="input"
              type="password"
              autoComplete="new-password"
              value={form.password}
              onChange={(event) => setForm({ ...form, password: event.target.value })}
              required
            />
          </Field>
          <Field label="角色">
            <select
              className="select"
              value={form.role}
              onChange={(event) => setForm({ ...form, role: event.target.value as User['role'] })}
            >
              <option value="operator">操作员</option>
              <option value="viewer">查看者</option>
              <option value="admin">管理员</option>
            </select>
          </Field>
          <div className="form-actions" style={{ alignItems: 'flex-end' }}>
            <button className="btn btn-accent" type="submit" disabled={create.isPending}>
              {create.isPending ? '创建中…' : '新建用户'}
            </button>
          </div>
        </div>
      </form>

      {list.isLoading ? (
        <Loader />
      ) : rows.length === 0 ? (
        <Empty>还没有用户</Empty>
      ) : (
        <div className="table-wrap">
          <table className="table">
            <thead>
              <tr>
                <th className="num">#</th>
                <th>用户名</th>
                <th>显示名</th>
                <th>角色</th>
                <th>状态</th>
                <th>创建时间</th>
                <th className="col-actions" />
              </tr>
            </thead>
            <tbody>
              {rows.map((row) => (
                <tr key={row.id}>
                  <td className="num">{row.id}</td>
                  <td className="mono">{row.username}</td>
                  <td>{row.display_name}</td>
                  <td>
                    <Badge tone={row.role === 'admin' ? 'info' : 'none'}>{ROLE_LABEL(row.role)}</Badge>
                  </td>
                  <td>
                    <Badge tone={row.active ? 'ok' : 'none'}>{row.active ? '启用' : '停用'}</Badge>
                  </td>
                  <td className="mono dim">{fmtTime(row.created_at)}</td>
                  <td className="col-actions">
                    <div className="row-actions">
                      <button className="btn btn-sm" type="button" onClick={() => toggle.mutate(row)}>
                        {row.active ? '停用' : '启用'}
                      </button>
                      <button className="btn btn-sm" type="button" onClick={() => revoke.mutate(row)}>
                        吊销会话
                      </button>
                      <button className="btn btn-ghost btn-sm" type="button" onClick={() => setResetFor(row)}>
                        重置口令
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      <PromptDialog
        open={resetFor !== null}
        title={`重置 ${resetFor?.username ?? ''} 的口令`}
        label="新口令"
        hint="至少 8 位；提交后该用户的旧会话全部失效"
        confirmLabel="重置口令"
        busy={resetPassword.isPending}
        onSubmit={(password) => {
          if (!resetFor) return
          if (password.length < 8) {
            setError(new Error('口令至少 8 位'))
            return
          }
          resetPassword.mutate({ row: resetFor, password })
        }}
        onCancel={() => setResetFor(null)}
      />
    </section>
  )
}