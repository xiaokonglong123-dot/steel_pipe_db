#!/bin/bash
# pg-install.sh — one-shot PostgreSQL 18.4 install + init + start + create DBs.
#
# WHY THIS SCRIPT EXISTS: the user-level PG at ~/.local/pgsql gets wiped by
# the environment's periodic cleanup of non-standard ~/.local entries (twice
# observed). Everything below is fully reproducible — DB contents are either
# migration-seeded or bootstrap-generated, so re-running this script from
# scratch is safe. Run it as the normal user (no root needed).
#
# Mirrors: Tsinghua Arch mirror (extra repo). CA bundle: Steam runtime certs
# (system /etc/ssl/certs is broken on this box).

set -euo pipefail

CA_BUNDLE="/home/yzp/.local/share/Steam/steamrt64/pv-runtime/steam-runtime-steamrt/steamrt3c_platform_3c.0.20260618.246540/files/etc/ssl/certs/ca-certificates.crt"
MIRROR="https://mirrors.tuna.tsinghua.edu.cn/archlinux/extra/os/x86_64"
PGDIR="$HOME/.local/pgsql"
PKG_VER="18.4-3"
DATA_DIR="$PGDIR/data"
PORT="${PG_PORT:-5432}"

echo "==> [1/6] Downloading PostgreSQL ${PKG_VER} packages"
TMP=$(mktemp -d /tmp/pg-install.XXXXXX)
curl -sL --cacert "$CA_BUNDLE" -o "$TMP/postgresql.pkg.tar.zst" \
    "$MIRROR/postgresql-${PKG_VER}-x86_64.pkg.tar.zst"
curl -sL --cacert "$CA_BUNDLE" -o "$TMP/postgresql-libs.pkg.tar.zst" \
    "$MIRROR/postgresql-libs-${PKG_VER}-x86_64.pkg.tar.zst"

echo "==> [2/6] Extracting into ${PGDIR}"
rm -rf "$PGDIR"
mkdir -p "$PGDIR"
tar --zstd -xf "$TMP/postgresql.pkg.tar.zst" -C "$PGDIR"
tar --zstd -xf "$TMP/postgresql-libs.pkg.tar.zst" -C "$PGDIR"
# Re-arrange the Arch layout into the paths PG binaries expect:
#   usr/bin -> bin, usr/lib/postgresql -> lib/postgresql (extensions),
#   usr/share/postgresql -> share (bki), usr/lib/* -> lib (libpq.so.5)
mv "$PGDIR/usr/bin" "$PGDIR/bin"
# keep the share/postgresql subdir: binaries are compiled with
# SHAREDIR=/usr/share/postgresql and initdb looks up timezonesets there
mkdir -p "$PGDIR/share"
mv "$PGDIR/usr/share/postgresql" "$PGDIR/share/postgresql"
mkdir -p "$PGDIR/lib"
mv "$PGDIR/usr/lib/postgresql" "$PGDIR/lib/postgresql"
mv "$PGDIR/usr/lib"/* "$PGDIR/lib/" 2>/dev/null || true
rm -rf "$PGDIR/usr"

echo "==> [3/6] initdb (data dir: ${DATA_DIR})"
export LD_LIBRARY_PATH="$PGDIR/lib"
"$PGDIR/bin/initdb" -D "$DATA_DIR" -L "$PGDIR/share/postgresql" \
    -U postgres --encoding=UTF8 --locale=C

echo "==> [4/6] Starting server (socket /tmp, port ${PORT})"
"$PGDIR/bin/pg_ctl" -D "$DATA_DIR" -l "$PGDIR/server.log" \
    -o "-p ${PORT} -k /tmp" start

echo "==> [5/6] Creating databases"
"$PGDIR/bin/psql" -h /tmp -p "$PORT" -U postgres -c "CREATE DATABASE steel_pipe;"
"$PGDIR/bin/psql" -h /tmp -p "$PORT" -U postgres -c "CREATE DATABASE steel_pipe_test;"

echo "==> [6/6] Verifying"
"$PGDIR/bin/pg_isready" -h /tmp -p "$PORT"
"$PGDIR/bin/psql" -h /tmp -p "$PORT" -U postgres -t -c \
    "SELECT datname FROM pg_database WHERE datname LIKE 'steel%';"

echo "==> DONE. Use scripts/pg-dev.sh to manage the server."
