#!/usr/bin/env bash
# 分层断言：依赖图违反架构约束时直接失败。
# 规则（docs/2026-11-17-rewrite-reference.md §5）：
#   steelpipe-api   不得依赖 sqlx / steelpipe-infra（HTTP 层不认识数据库驱动）
#   steelpipe-infra 不得依赖 axum（持久层不认识 HTTP）
set -euo pipefail

fail=0

assert_no_dep() {
  local crate=$1
  shift
  local deps
  deps=$(cargo tree -p "$crate" -e normal --prefix none --no-dedupe 2>/dev/null | awk 'NF {print $1}' | sort -u)
  if [ -z "$deps" ]; then
    echo "无法解析依赖树：$crate" >&2
    fail=1
    return
  fi
  local needle
  for needle in "$@"; do
    if grep -qx -- "$needle" <<<"$deps"; then
      echo "分层违规：$crate 依赖了 $needle"
      fail=1
    fi
  done
}

assert_no_dep steelpipe-api sqlx steelpipe-infra
assert_no_dep steelpipe-infra axum steelpipe-api

if [ "$fail" -eq 0 ]; then
  echo "分层断言通过"
fi
exit "$fail"
