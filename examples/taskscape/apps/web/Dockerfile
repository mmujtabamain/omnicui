# The web app in one container for previews: the built frontend, the Django API (gunicorn) and a
# Postgres of its own. Build from examples/taskscape:  docker build -f apps/web/Dockerfile .
# Production runs the same image with DATABASE_URL pointing at a managed Postgres instead.

FROM node:22-slim AS frontend
WORKDIR /src
COPY package.json package-lock.json ./
COPY packages/ui/package.json packages/ui/
COPY packages/ui/scripts packages/ui/scripts
COPY packages/eslint-plugin/package.json packages/eslint-plugin/
COPY apps/web/frontend/package.json apps/web/frontend/
COPY omnicui/targets/web/package.json omnicui/targets/web/
RUN PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1 npm ci --no-audit --no-fund
COPY packages packages
COPY apps/web/frontend apps/web/frontend
RUN node packages/ui/scripts/fonts.mjs && cd apps/web/frontend && npx vite build

FROM python:3.13-slim
RUN apt-get update && apt-get install -y --no-install-recommends postgresql && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY apps/web/backend/requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY apps/web/backend .
COPY --from=frontend /src/apps/web/frontend/dist /app/frontend
COPY apps/web/start.sh /start.sh
ENV FRONTEND_DIST=/app/frontend MEDIA_ROOT=/data/media PORT=8080
EXPOSE 8080
CMD ["sh", "/start.sh"]
