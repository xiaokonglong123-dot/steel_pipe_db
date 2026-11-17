# 部署说明（systemd + SQLite 单实例）

## 目录布局

```text
/srv/steelpipe/
├── steelpipe            # 二进制（cargo build --release）
├── data/steelpipe.db    # 数据库（WAL 三个文件都在这里）
├── data/backup/         # 备份
└── scripts/             # backup.sh / restore.sh
/etc/steelpipe/env       # 环境变量（0600，含 JWT 密钥）
```bash

## 安装

```bash
sudo useradd --system --home /srv/steelpipe --shell /usr/sbin/nologin steelpipe
sudo install -d -o steelpipe -g steelpipe /srv/steelpipe/{data,data/backup}
sudo install -m 0755 target/release/steelpipe /srv/steelpipe/steelpipe
sudo install -m 0644 deploy/steelpipe.service /etc/systemd/system/steelpipe.service

sudo install -d -m 0750 /etc/steelpipe
sudo tee /etc/steelpipe/env >/dev/null <<'EOF'
STEELPIPE_JWT_SECRET=<openssl rand -base64 48 的输出>
STEELPIPE_DATABASE_URL=sqlite:///srv/steelpipe/data/steelpipe.db
STEELPIPE_BIND=127.0.0.1:8080
STEELPIPE_COOKIE_SECURE=true
EOF
sudo chmod 0600 /etc/steelpipe/env
sudo chown root:root /etc/steelpipe/env

sudo systemctl daemon-reload
sudo systemctl enable --now steelpipe
sudo systemctl status steelpipe
```bash

## 首次引导管理员

```bash
sudo systemctl stop steelpipe
sudo -u steelpipe env $(cat /etc/steelpipe/env | xargs) \
  /srv/steelpipe/steelpipe serve     # 临时以 STEELPIPE_ADMIN_* 环境变量启动一次
```cron

或在 `/etc/steelpipe/env` 临时加入 `STEELPIPE_ADMIN_USERNAME` / `STEELPIPE_ADMIN_PASSWORD`，
启动后**立即删除这两行**（引导只在启动时执行，同名用户会被重置口令）。

## cron

```cron
30 1 * * * /srv/steelpipe/scripts/backup.sh /srv/steelpipe/data/steelpipe.db /srv/steelpipe/data/backup
0 2 * * * cd /srv/steelpipe && ./steelpipe reconcile
0 3 1 * * cd /srv/steelpipe && ./steelpipe audit verify --batch 5000
```bash

## 升级

```bash
sudo systemctl stop steelpipe
sudo -u steelpipe /srv/steelpipe/scripts/backup.sh
sudo install -m 0755 target/release/steelpipe /srv/steelpipe/steelpipe
sudo systemctl start steelpipe      # 启动时自动应用 sqlx 迁移
```text

回滚：`scripts/restore.sh <备份>` 后换回旧二进制。

## 约束

- **不要起多个实例**：SQLite 单写者 + 审计链串行化都依赖它。
- 反代需开启 HTTPS 并让 `STEELPIPE_COOKIE_SECURE=true` 与之匹配；否则 cookie 不会带上 `Secure`。
- `data/` 必须可写，且不要放在网络文件系统上（WAL 锁不可靠）。
