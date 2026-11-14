#!/bin/sh
# Starts the web app. Without DATABASE_URL it runs a Postgres of its own in /data (a preview);
# DEMO_DATA=1 loads the demo account (demo@taskscape.test / taskscape-demo) on every start.
set -e
if [ -z "$DATABASE_URL" ]; then
  PG=$(ls -d /usr/lib/postgresql/*/bin | tail -1)
  mkdir -p /data/pg /data/media && chown -R postgres /data
  [ -f /data/pg/PG_VERSION ] || su postgres -c "$PG/initdb -D /data/pg -A trust >/dev/null"
  su postgres -c "$PG/pg_ctl -D /data/pg -o '-k /tmp -h 127.0.0.1' -l /data/pg.log -w start"
  su postgres -c "psql -h 127.0.0.1 -tc \"SELECT 1 FROM pg_database WHERE datname='taskscape'\" | grep -q 1 || createdb -h 127.0.0.1 taskscape"
  export DATABASE_URL=postgres://postgres@127.0.0.1:5432/taskscape
fi
export ALLOWED_HOSTS="${ALLOWED_HOSTS:-*}" TRUST_PROXY="${TRUST_PROXY:-1}"
export SECRET_KEY="${SECRET_KEY:-$(python -c 'import secrets; print(secrets.token_urlsafe(40))')}"
python manage.py migrate --noinput
[ "$DEMO_DATA" = "1" ] && python manage.py seed_demo
exec gunicorn config.wsgi --bind "0.0.0.0:${PORT:-8080}" --workers 3 --access-logfile - --error-logfile -
