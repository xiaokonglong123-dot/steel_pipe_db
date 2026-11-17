/** 把后端 ISO 时间格式化为界面用短文本；空值统一显示 —。 */
export function fmtTime(value: string | null | undefined, withDate = true): string {
  if (!value) return '—'
  const text = value.replace('T', ' ')
  return withDate ? text.slice(5, 16) : text.slice(11, 19)
}

export function fmtFull(value: string | null | undefined): string {
  if (!value) return '—'
  return value.replace('T', ' ').slice(0, 19)
}

export function todayLabel(): string {
  const now = new Date()
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())} ${pad(now.getHours())}:${pad(now.getMinutes())}`
}