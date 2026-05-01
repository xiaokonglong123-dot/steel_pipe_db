#!/bin/bash

# 检查依赖
command -v cargo >/dev/null 2>&1 || { echo >&2 "需要安装 Rust/Cargo。"; exit 1; }
command -v npm >/dev/null 2>&1 || { echo >&2 "需要安装 Node.js/npm。"; exit 1; }

# 获取脚本所在目录
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_DIR="$DIR/projects/rust-axum-vue3"

function start_dev() {
    echo "正在启动开发模式..."
    # 启动后端
    cd "$PROJECT_DIR/server" && cargo run &
    BACKEND_PID=$!
    
    # 启动前端
    cd "$PROJECT_DIR/web" && npm run dev &
    FRONTEND_PID=$!
    
    echo "后端 PID: $BACKEND_PID"
    echo "前端 PID: $FRONTEND_PID"
    
    trap "kill $BACKEND_PID $FRONTEND_PID; exit" INT TERM EXIT
    wait
}

function build_prod() {
    echo "正在构建生产版本..."
    
    # 构建前端
    echo "构建前端..."
    cd "$PROJECT_DIR/web" && npm install && npm run build
    
    # 构建后端
    echo "构建后端..."
    cd "$PROJECT_DIR/server" && cargo build --release
    
    echo "构建完成！构建产物在 projects/rust-axum-vue3/server/target/release/"
}

case "$1" in
    dev)
        start_dev
        ;;
    build)
        build_prod
        ;;
    *)
        echo "用法: $0 {dev|build}"
        exit 1
        ;;
esac
