/** EntitySelect 远程下拉的共享类型（放 .ts 而非 .vue：CI 用原生 tsc，解析不了 .vue 的命名导出） */
export interface Selectable {
  readonly id: number
  readonly name: string
  readonly code?: string | null
}

export type SelectApi = (query: string) => Promise<readonly Selectable[]>
