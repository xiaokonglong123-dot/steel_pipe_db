Windows 用法说明（管理员友好）

1. 准备工作
- 确保已安装 Python 3.x，且已加入系统 PATH。
- 将本仓库完整下载并解压到本地，如：C:\steel_pipe_db

2. 启动服务
- 双击 windows/start_server.bat，或在命令提示符中进入仓库目录后执行 start_server.bat。
- 批处理脚本会创建虚拟环境、安装依赖并启动 Flask 服务器，默认监听 http://localhost:5000/
- 启动后会自动在浏览器打开界面，若未自动打开，请手动在浏览器中访问 http://localhost:5000/

3. 测试数据与操作
- 导入测试数据：使用界面导入 Pipes CSV（pipes_test.csv）
- 导出数据：使用界面导出 CSV，下载到本地
- 日常管理：通过网页界面执行 CRUD、搜索、筛选、排序、分页

4. 常见问题
- 如果首次启动失败，请确认端口未被占用，Python 环境可访问。
- 如有依赖安装失败，请确保网络通畅或改用离线依赖包。
