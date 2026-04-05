@echo off
REM Windows一键启动（用于仓库管理员的简易测试）
setlocal

pushd %~dp0..

REM 1) 创建虚拟环境
IF NOT EXIST "venv" (
    python -m venv venv
)

REM 2) 激活虚拟环境
CALL venv\Scripts\activate.bat

REM 3) 安装依赖
pip install --upgrade pip
pip install -r backend\requirements.txt

REM 4) 设置环境变量并启动服务
set FLASK_APP=backend/app.py
set FLASK_ENV=development
echo 启动 Flask 服务器...
start "SteelPipeDB" cmd /k "python backend\app.py"

REM 5) 自动打开浏览器
start "" http://localhost:5000/

echo 服务已在后台运行，浏览器应自动打开 http://localhost:5000/
popd
pause
exit /b 0
