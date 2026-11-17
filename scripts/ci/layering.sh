#!/usr/bin/env bash
# 分层断言：直接依赖图违反架构约束时失败（docs/2026-11-17-rewrite-reference.md §5）。
#   domain / auth / audit : 无 IO、无 Web
#   infra                 : 只有持久化，不认识 HTTP 与用例
#   service               : 用例编排，不认识 HTTP
#   api                   : HTTP 适配器，不直接碰 sqlx / infra
set -euo pipefail

fail=0

# 只看直接依赖（--depth 1）：允许 service 经 infra 间接用到 sqlx
direct_deps() {
  cargo tree -p "$1" -e normal --depth 1 --prefix none 2>/dev/null | awk 'NF {print $1}' | sort -u
}

assert_no_dep() {
  local crate=$1
  shift
  local deps
  deps=$(direct_deps "$crate")
  if [ -z "$deps" ]; then
    echo "无法解析依赖树：$crate" >&2
    fail=1
    return
  fi
  local needle
  for needle in "$@"; do
    if grep -qx -- "$needle" <<<"$deps"; then
      echo "分层违规：$crate 直接依赖了 $needle"
      fail=1
    fi
  done
}

assert_no_dep steelpipe-domain axum sqlx tokio steelpipe-infra steelpipe-service
assert_no_dep steelpipe-auth   axum sqlx tokio steelpipe-infra steelpipe-service
assert_no_dep steelpipe-audit  axum sqlx tokio steelpipe-infra steelpipe-service
assert_no_dep steelpipe-compliance axum sqlx tokio steelpipe-infra steelpipe-service
assert_no_dep steelpipe-infra  axum steelpipe-api steelpipe-service
assert_no_dep steelpipe-service axum steelpipe-api
assert_no_dep steelpipe-api    sqlx steelpipe-infra

if [ "$fail" -eq 0 ]; then
  echo "分层断言通过"
fi
exit "$fail"
