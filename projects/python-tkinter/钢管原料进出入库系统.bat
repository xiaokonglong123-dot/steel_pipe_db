
@echo off
chcp 65001 >nul
title 钢管原料进出入库管理系统
echo 正在启动钢管原料进出入库管理系统...
python main.py
if errorlevel 1 (
    echo.
    echo 系统启动失败，请检查是否已安装Python 3.6或更高版本
    pause
)
