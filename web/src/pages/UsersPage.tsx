import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import { api, type Schemas } from '../api/client'
import { Field, Loader, PageHead, Problem, ROLE_LABEL } from '../components/ui'
import { useSession } from '../session'

type User = Schemas['UserDto']

export default function UsersPage() {
  const { isAdmin } = useSession()
  const queryClient = useQueryClient()
  const [error, setError] = useState<unknown>(null)
  const [form, setForm] = useState({ username: '', password: '', display_name: '', role: 'operator' as User['role'] })

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
      setForm({ username: '', password: '', display_name: '', role: 'operator' })
      setError(null)
      invalidate()
    },
    onError: (cause) => setError(cause),
  })

  const toggle = useMutation({
    mutationFn: (row: User) => api(`/api/users/${row.id}/active`, { method: 'POST', body: { active: !row.active } }),
    onSuccess: () => {
      setError(null)
      invalidate()
    },
    onError: (cause) => setError(cause),
  })

  const revoke = useMutation({
    mutationFn: (row: User) => api<Schemas['RevokeResponse']>(`/api/users/${row.id}/revoke`, { method: 'POST', body: {} }),
    onSuccess: () => {
      setError(null)
      invalidate()
    },
    onError: (cause) => setError(cause),
  })

  const resetPassword = useMutation({
    mutationFn: ({ row, password }: { row: User; password: string }) =>
      api(`/api/users/${row.id}/password`, { method: 'POST', body: { password } }),
    onSuccess: () => {
      setError(null)
      invalidate()
    },
    onError: (cause) => setError(cause),
  })

  if (!isAdmin) return <Problem error={new Error('仅管理员可访问')} />

  return (
    <section>
      <PageHead title="用户管理" />
      <Problem error={error} />

      <form
        className="toolbar"
        onSubmit={(event) => {
          event.preventDefault()
          create.mutate()
        }}
      >
        <Field label="用户名"><input className="input input-sm" value={form.username} onChange={(e) => setForm({ ...form, username: e.target.value })} required /></Field>
        <Field label="显示名"><input className="input input-sm" value={form.display_name} onChange={(e) => setForm({ ...form, display_name: e.target.value })} /></Field>
        <Field label="初始口令"><input className="input input-sm" type="password" value={form.password} onChange={(e) => setForm({ ...form, password: e.target.value })} required /></Field>
        <Field label="角色">
          <select className="select select-sm" value={form.role} onChange={(e) => setForm({ ...form, role: e.target.value as User['role'] })}>
            <option value="operator">操作员</option>
            <option value="viewer">查看者</option>
            <option value="admin">管理员</option>
          </select>
        </Field>
        <button className="btn btn-accent" type="submit" disabled={create.isPending}>新建</button>
      </form>

      {list.isLoading ? (
        <Loader />
      ) : (
        <table className="table">
          <thead>
            <tr><th>#</th><th>用户名</th><th>显示名</th><th>角色</th><th>状态</th><th>会话代数</th><th>操作</th></tr>
          </thead>
          <tbody>
            {(list.data?.users ?? []).map((row) => (
              <tr key={row.id}>
                <td className="mono">{row.id}</td>
                <td>{row.username}</td>
                <td>{row.display_name}</td>
                <td>{ROLE_LABEL(row.role)}</td>
                <td><span className={`badge ${row.active ? 'is-ok' : 'is-none'}`}>{row.active ? '启用' : '停用'}</span></td>
                <td className="mono">{row.id}</td>
                <td className="row-actions">
                  <button className="btn btn-subtle btn-sm" onClick={() => toggle.mutate(row)}>
                    {row.active ? '停用' : '启用'}
                  </button>
                  <button className="btn btn-subtle btn-sm" onClick={() => revoke.mutate(row)}>
                    吊销会话
                  </button>
                  <button
                    className="btn btn-ghost btn-sm"
                    onClick={() => {
                      const next = window.prompt(`为 ${row.username} 设置新口令（至少 8 位）`)
                      if (next && next.length >= 8) resetPassword.mutate({ row, password: next })
                    }}
                  >
                    重置口令
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
      <p className="field-hint">停用与改密都会递增该用户 token_version，旧 token 立即失效。</p>
    </section>
  )
}
