#!/usr/bin/env bash
# SteelPipe 恢复：先备份当前库，再用指定备份覆盖，最后校验迁移与链。
#
# 用法：先停止 steelpipe 服务，再执行 scripts/restore.sh <备份文件> [数据库路径]
# 可通过 STEELPIPE_BIN 指定用于迁移与审计链校验的二进制。
set -euo pipefail

SRC="${1:?用法：restore.sh <备份文件> [数据库路径]}"
DB="${2:-${STEELPIPE_DATABASE_URL:-sqlite://data/steelpipe.db}}"
DB="${DB#sqlite://}"
DB="${DB#sqlite:}"

[ -f "$SRC" ] || { echo "备份不存在：$SRC" >&2; exit 1; }

if [ -f "$DB" ]; then
  SAFE="$DB.pre-restore.$(date +%s)"
  sqlite3 "$DB" ".timeout 5000" ".backup '$SAFE'"
  echo "已保留现场：$SAFE"
fi

sqlite3 "$SRC" ".timeout 5000" ".backup '$DB'"
echo "已恢复 $SRC → $DB"
sqlite3 "$DB" "PRAGMA integrity_check;"
"${STEELPIPE_BIN:-./target/release/steelpipe}" --database "sqlite://$DB" audit verify --batch 1000
