#!/usr/bin/env bash
# SteelPipe 备份：sqlite3 .backup（禁止 cp —— WAL 未 checkpoint 时 cp 出来的库是坏的）。
#
# 用法：scripts/backup.sh [数据库路径] [备份目录]
# cron：30 1 * * * /srv/steelpipe/scripts/backup.sh
set -euo pipefail

DB="${1:-${STEELPIPE_DATABASE_URL:-sqlite://data/steelpipe.db}}"
DB="${DB#sqlite://}"
DB="${DB#sqlite:}"
DIR="${2:-data/backup}"

[ -f "$DB" ] || { echo "数据库不存在：$DB" >&2; exit 1; }
mkdir -p "$DIR"

backup() {
  local target="$1"
  sqlite3 "$DB" ".timeout 5000" ".backup '$target'"
  echo "已备份 $DB → $target（$(du -h "$target" | cut -f1)）"
}

# 保留最近 N 份同 pattern 的备份（显式文件路径，逐个 rm）
prune() {
  local pattern="$1" keep="$2"
  local files=() count=0
  shopt -s nullglob
  local candidates=("$DIR"/$pattern)
  shopt -u nullglob
  if [ "${#candidates[@]}" -eq 0 ]; then
    return 0
  fi
  # 文件名即 ISO 日期，字典序 = 时间序；倒序后保留前 keep 份
  mapfile -t files < <(printf '%s\n' "${candidates[@]}" | sort -r)

  count=${#files[@]}
  if [ "$count" -le "$keep" ]; then
    return 0
  fi
  local idx=$((keep))
  while [ "$idx" -lt "$count" ]; do
    rm -f -- "${files[$idx]}"
    echo "清理旧备份：${files[$idx]}"
    idx=$((idx + 1))
  done
}

backup "$DIR/steelpipe-$(date +%F).db"
backup "$DIR/steelpipe-$(date +%G-W%V).db"

prune 'steelpipe-????-??-??.db' 7
prune 'steelpipe-????-W??.db' 4

echo "当前备份（最多 12 条）："
shopt -s nullglob
recent=("$DIR"/steelpipe-*.db)
shopt -u nullglob
if [ "${#recent[@]}" -gt 0 ]; then
  printf '%s\n' "${recent[@]}" | sort -r | head -12
fi
