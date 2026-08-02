#!/bin/bash
# PostgreSQL 18.4 用户级实例管理脚本
# 安装位置: ~/.local/pgsql (从 Arch extra 仓库 pkg.tar.zst 解压)
# 数据目录: ~/.local/pgsql/data
# 监听: /tmp socket, 端口 5432
#
# 用法: scripts/pg-dev.sh {start|stop|status|init|psql}
#   start  - 启动服务器 (后台, 日志 ~/.local/pgsql/server.log)
#   stop   - 停止服务器
#   status - 检查服务器状态
#   init   - 初始化数据目录并重建 steel_pipe / steel_pipe_test 数据库
#   psql   - 打开 psql 连接到 steel_pipe

set -euo pipefail

PGSQL_HOME="${HOME}/.local/pgsql"
PGBIN="${PGSQL_HOME}/bin"
PGDATA="${PGSQL_HOME}/data"
PGLOG="${PGSQL_HOME}/server.log"
PGPORT="${PGPORT:-5432}"
PGSOCKET="/tmp"
# libpq 动态库位于 lib/ 顶层 (从 postgresql-libs 包解压)
export LD_LIBRARY_PATH="${PGSQL_HOME}/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

require_installed() {
    if [ ! -x "${PGBIN}/postgres" ]; then
        echo "错误: PostgreSQL 未安装于 ${PGSQL_HOME}" >&2
        echo "从 Arch extra 仓库下载 postgresql + postgresql-libs 解压到此目录:" >&2
        echo "  tar --zstd -xf postgresql-*.pkg.tar.zst -C ${PGSQL_HOME}" >&2
        echo "  tar --zstd -xf postgresql-libs-*.pkg.tar.zst -C ${PGSQL_HOME}" >&2
        echo "然后整理布局: bin/ lib/ share/postgresql/ lib/postgresql/ (见 README)" >&2
        exit 1
    fi
}

cmd_start() {
    require_installed
    if "${PGBIN}/pg_isready" -h "${PGSOCKET}" -p "${PGPORT}" >/dev/null 2>&1; then
        echo "PostgreSQL 已在运行 (${PGSOCKET}:${PGPORT})"
        return 0
    fi
    if [ ! -d "${PGDATA}" ]; then
        echo "错误: 数据目录 ${PGDATA} 不存在, 请先运行: $0 init" >&2
        exit 1
    fi
    "${PGBIN}/pg_ctl" -D "${PGDATA}" -l "${PGLOG}" \
        -o "-p ${PGPORT} -k ${PGSOCKET}" start
    echo "PostgreSQL 已启动 (${PGSOCKET}:${PGPORT}, 日志: ${PGLOG})"
}

cmd_stop() {
    require_installed
    if "${PGBIN}/pg_isready" -h "${PGSOCKET}" -p "${PGPORT}" >/dev/null 2>&1; then
        "${PGBIN}/pg_ctl" -D "${PGDATA}" stop
        echo "PostgreSQL 已停止"
    else
        echo "PostgreSQL 未在运行"
    fi
}

cmd_status() {
    require_installed
    "${PGBIN}/pg_isready" -h "${PGSOCKET}" -p "${PGPORT}"
}

cmd_init() {
    require_installed
    if [ -d "${PGDATA}" ]; then
        echo "数据目录已存在, 正在停止并删除旧数据..."
        "${PGBIN}/pg_ctl" -D "${PGDATA}" stop >/dev/null 2>&1 || true
        rm -rf "${PGDATA}"
    fi
    "${PGBIN}/initdb" -D "${PGDATA}" -L "${PGSQL_HOME}/share/postgresql" \
        -U postgres --encoding=UTF8 --locale=C
    cmd_start
    sleep 1
    # 应用数据库 (steel_pipe) 和测试数据库 (steel_pipe_test)
    PGHOST="${PGSOCKET}" PGUSER=postgres "${PGBIN}/psql" -d postgres \
        -c "CREATE DATABASE steel_pipe;" -c "CREATE DATABASE steel_pipe_test;"
    echo "数据库 steel_pipe / steel_pipe_test 已创建"
    echo "提示: 迁移由应用启动时自动执行 (sqlx::migrate!)"
}

cmd_psql() {
    require_installed
    PGHOST="${PGSOCKET}" PGUSER=postgres "${PGBIN}/psql" -d "${1:-steel_pipe}"
}

case "${1:-}" in
    start)  cmd_start ;;
    stop)   cmd_stop ;;
    status) cmd_status ;;
    init)   cmd_init ;;
    psql)   cmd_psql "${2:-}" ;;
    *)
        echo "用法: $0 {start|stop|status|init|psql [db]}"
        exit 1
        ;;
esac
