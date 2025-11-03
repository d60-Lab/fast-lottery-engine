#!/bin/sh
set -eu

echo "[entrypoint] applying migrations..."
/usr/local/bin/migrate || { echo "[entrypoint] migrate failed" >&2; exit 1; }

echo "[entrypoint] preparing inventory..."
/usr/local/bin/db_prepare || { echo "[entrypoint] db_prepare failed" >&2; exit 1; }

echo "[entrypoint] starting server..."
exec /usr/local/bin/fast-lottery-engine
