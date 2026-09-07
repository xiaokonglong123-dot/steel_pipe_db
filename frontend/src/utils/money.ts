// utils/money.ts — 金额/数量工具
//
// 后端 Decimal 序列化为 JSON string（ADR-R6）。前端一律按 string 处理，
// 禁止 parseFloat 参与加法累计。这里用 BigInt 缩放实现安全的十进制加减乘。

type Rep = { readonly val: bigint; readonly scale: number }

const MAX_SCALE = 30

/** 解析十进制字符串 → { val, scale }（val 为缩放后的 bigint，可含负号）。 */
function parse(s: string): Rep {
  let str = (s ?? "").trim()
  if (!str) str = "0"
  let neg = false
  if (str.startsWith("-")) { neg = true; str = str.slice(1) }
  const dot = str.indexOf(".")
  let int = dot === -1 ? str : str.slice(0, dot)
  let frac = dot === -1 ? "" : str.slice(dot + 1)
  if (frac.length > MAX_SCALE) frac = frac.slice(0, MAX_SCALE)
  int = int.replace(/^\d+$/, int) // keep as-is; leading zeros harmless for BigInt
  // strip trailing zeros to reduce scale
  const stripped = frac.replace(/0+$/, "")
  const scale = stripped.length
  let val = BigInt((int === "" ? "0" : int) + (stripped === "" ? "" : stripped))
  if (neg) val = -val
  return { val, scale }
}

/** 将 val+scale 还原成十进制字符串（去尾零）。 */
function toStr(r: Rep): string {
  const neg = r.val < 0n
  let v = neg ? -r.val : r.val
  let s = v.toString()
  if (r.scale === 0) return (neg ? "-" : "") + s
  while (s.length <= r.scale) s = "0" + s
  const intPart = s.slice(0, s.length - r.scale)
  const fracPart = s.slice(s.length - r.scale).replace(/0+$/, "")
  return (neg ? "-" : "") + intPart + (fracPart ? "." + fracPart : "")
}

function align(a: Rep, b: Rep): { a: Rep; b: Rep } {
  if (a.scale < b.scale) return { a: { val: a.val * 10n ** BigInt(b.scale - a.scale), scale: b.scale }, b }
  if (a.scale > b.scale) return { a, b: { val: b.val * 10n ** BigInt(a.scale - b.scale), scale: a.scale } }
  return { a, b }
}

/** 加法：add("1.5","2.25") → "3.75"。可传多个操作数。 */
export function add(...parts: Array<string | number>): string {
  let acc: Rep | null = null
  for (const p of parts) {
    const cur = parse(String(p))
    if (acc === null) { acc = cur; continue }
    const { a, b } = align(acc, cur)
    acc = { val: a.val + b.val, scale: a.scale }
  }
  return toStr(acc ?? { val: 0n, scale: 0 })
}

/** 乘法：mul("10","12.5") → "125"。 */
export function mul(a: string | number, b: string | number): string {
  const pa = parse(String(a))
  const pb = parse(String(b))
  return toStr({ val: pa.val * pb.val, scale: pa.scale + pb.scale })
}

/** 求和（数组）。 */
export function sum(items: readonly (string | number)[]): string {
  return add(...items.map((i) => String(i)))
}

/** 减法：sub("10","2.5") → "7.5"。 */
export function sub(a: string | number, b: string | number): string {
  const { a: aa, b: bb } = align(parse(String(a)), parse(String(b)))
  return toStr({ val: aa.val - bb.val, scale: aa.scale })
}

function cmpA(a: Rep, b: Rep): number {
  const { a: x, b: y } = align(a, b)
  return x.val < y.val ? -1 : x.val > y.val ? 1 : 0
}

export function gt(a: string | number, b: string | number): boolean {
  return cmpA(parse(String(a)), parse(String(b))) > 0
}
export function gte(a: string | number, b: string | number): boolean {
  return cmpA(parse(String(a)), parse(String(b))) >= 0
}
export function lt(a: string | number, b: string | number): boolean {
  return cmpA(parse(String(a)), parse(String(b))) < 0
}
export function eq(a: string | number, b: string | number): boolean {
  return cmpA(parse(String(a)), parse(String(b))) === 0
}

/** 千分位格式化，默认保留 2 位小数；空/非法输入安全返回 "0.00"。 */
export function formatAmount(s: string | number | null | undefined, dp = 2): string {
  if (s === null || s === undefined || s === "") return "0.00"
  const r = parse(String(s))
  const neg = r.val < 0n
  let v = neg ? -r.val : r.val
  let digits = v.toString()
  if (r.scale === 0) {
    // 整数：直接补 0 dp 位小数
    const intStr = digits
    const grouped = groupInt(intStr)
    return (neg ? "-" : "") + grouped + "." + "0".repeat(dp)
  }
  while (digits.length <= r.scale) digits = "0" + digits
  let intPart = digits.slice(0, digits.length - r.scale)
  let fracPart = digits.slice(digits.length - r.scale)
  // 四舍五入到 dp 位
  if (fracPart.length > dp) {
    const keep = fracPart.slice(0, dp)
    const next = fracPart.slice(dp, dp + 1)
    if (next && Number(next) >= 5) {
      const rounded = (BigInt(keep || "0") + 1n).toString()
      if (rounded.length > dp) {
        intPart = String(BigInt(intPart) + 1n)
        fracPart = rounded.slice(1)
      } else {
        fracPart = rounded.padStart(dp, "0")
      }
    } else {
      fracPart = keep
    }
  }
  fracPart = fracPart.padEnd(dp, "0")
  return (neg ? "-" : "") + groupInt(intPart) + "." + fracPart
}

function groupInt(intStr: string): string {
  if (intStr.length <= 3) return intStr
  return intStr.replace(/\B(?=(\d{3})+(?!\d))/g, ",")
}

/** 展示用数量（保留原精度，去尾零）。 */
export function formatQty(s: string | number | null | undefined): string {
  if (s === null || s === undefined || s === "") return "0"
  return toStr(parse(String(s)))
}