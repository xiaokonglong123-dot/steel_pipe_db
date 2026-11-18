# 部署手册（systemd + SQLite 单实例）

以下示例以 `/srv/steelpipe` 为安装目录、专用 `steelpipe` 用户运行。服务默认只监听回环地址，应由 HTTPS 反向代理对外提供访问。

## 目录与权限

```text
/srv/steelpipe/
├── steelpipe             # 后端、前端与运维 CLI 的单一二进制
├── data/steelpipe.db     # SQLite 主库及 WAL 文件
├── data/backup/          # 在线备份
└── scripts/              # backup.sh、restore.sh
/etc/steelpipe/env        # 环境变量，root 所有，0600
```

```bash
sudo useradd --system --home /srv/steelpipe --shell /usr/sbin/nologin steelpipe
sudo install -d -o steelpipe -g steelpipe /srv/steelpipe/{data,data/backup,scripts}
sudo install -m 0755 target/release/steelpipe /srv/steelpipe/steelpipe
sudo install -m 0755 scripts/backup.sh scripts/restore.sh /srv/steelpipe/scripts/
sudo install -m 0644 deploy/steelpipe.service /etc/systemd/system/steelpipe.service
```

## 环境配置

```bash
sudo install -d -m 0750 /etc/steelpipe
sudo tee /etc/steelpipe/env >/dev/null <<'EOF'
STEELPIPE_JWT_SECRET=<openssl rand -base64 48 的输出>
STEELPIPE_DATABASE_URL=sqlite:///srv/steelpipe/data/steelpipe.db
STEELPIPE_BIND=127.0.0.1:8080
STEELPIPE_COOKIE_SECURE=true
STEELPIPE_TRUST_FORWARDED_HEADERS=false
EOF
sudo chown root:root /etc/steelpipe/env
sudo chmod 0600 /etc/steelpipe/env
```

`STEELPIPE_JWT_SECRET` 至少 32 字节。其他可选配置见项目根目录 `.env.example` 和 `README.md`。

## 首次启动与管理员引导

在 `/etc/steelpipe/env` 中临时加入：

```dotenv
STEELPIPE_ADMIN_USERNAME=root
STEELPIPE_ADMIN_PASSWORD=<至少 8 位的管理员口令>
```

启动服务并确认管理员创建成功：

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now steelpipe
sudo systemctl status steelpipe --no-pager
curl --fail http://127.0.0.1:8080/api/health
curl --fail http://127.0.0.1:8080/api/health/db
```

随后**立即删除** `STEELPIPE_ADMIN_USERNAME` 和 `STEELPIPE_ADMIN_PASSWORD` 两行，再重启服务：

```bash
sudo systemctl restart steelpipe
```

只要这两个变量存在，每次启动都会重置同名管理员口令并提升为管理员，不能长期保留。

## HTTPS 与反向代理

- 对外使用 HTTPS，保持 `STEELPIPE_COOKIE_SECURE=true`。
- 登录限流默认只信任 socket 对端地址。
- 若需要按真实客户端 IP 限流，反代必须覆盖外部传入的 `X-Forwarded-For`，再设置 `STEELPIPE_TRUST_FORWARDED_HEADERS=true`。
- 开启信任后，必须限制原服务只能从反代访问，不能允许客户端绕过代理直连。

## 定时备份与检查

示例 cron：

```cron
30 1 * * * /srv/steelpipe/scripts/backup.sh /srv/steelpipe/data/steelpipe.db /srv/steelpipe/data/backup
0 2 * * * cd /srv/steelpipe && /srv/steelpipe/steelpipe reconcile
0 3 1 * * cd /srv/steelpipe && /srv/steelpipe/steelpipe audit verify --batch 5000
```

备份脚本使用 `sqlite3 .backup`，支持服务运行时在线备份；不要用 `cp` 复制主库。日备保留 7 份、周备保留 4 份。

`reconcile` 会修复库位计数漂移并写审计；`audit verify` 在一致性读快照中校验审计链。数据库不得放在网络文件系统上。

## 升级

```bash
sudo systemctl stop steelpipe
sudo -u steelpipe /srv/steelpipe/scripts/backup.sh \
  /srv/steelpipe/data/steelpipe.db /srv/steelpipe/data/backup
sudo install -m 0755 target/release/steelpipe /srv/steelpipe/steelpipe
sudo systemctl start steelpipe
curl --fail http://127.0.0.1:8080/api/health/db
```

新二进制启动时自动应用迁移。升级前应确认备份可用；替换二进制前先停止服务，避免新旧进程同时写入。

## 回滚与恢复

先停止服务，使用**当前新二进制**执行恢复脚本，完成恢复、迁移和校验：

```bash
sudo systemctl stop steelpipe
sudo -u steelpipe env STEELPIPE_BIN=/srv/steelpipe/steelpipe \
  /srv/steelpipe/scripts/restore.sh /path/to/backup.db /srv/steelpipe/data/steelpipe.db
```

脚本会：

1. 通过 SQLite 在线备份保留当前数据库为 `.pre-restore.<timestamp>`；
2. 恢复指定备份；
3. 执行 `PRAGMA integrity_check`；
4. 通过 `STEELPIPE_BIN` 指定的二进制应用迁移并验证审计链。

任一步骤失败都会返回非零状态。确认校验通过后，再安装需要回退的旧二进制并启动服务；不要把尚未兼容旧版本迁移的数据库直接交给旧程序。

## 运行约束

- **只运行一个 `steelpipe serve` 实例。** SQLite 写事务和审计链均以单写者为前提。
- `data/` 必须对 `steelpipe` 用户可写，不能放在网络文件系统上。
- 恢复期间保持服务停止，不能对正在运行的数据库执行恢复。
- 备份、日志和环境文件不得提交到公开仓库。
