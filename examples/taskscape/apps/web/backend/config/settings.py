"""Settings come from the environment, so one image runs locally, in tests and in production.

DATABASE_URL     postgres://user:password@host:port/name (required; there is no SQLite fallback)
SECRET_KEY       required unless DEBUG is on
DEBUG            "1" for local development
ALLOWED_HOSTS    comma-separated
MEDIA_ROOT       where uploads go (default: media/ next to manage.py)
DEMO_RESET       "1" routes POST api/demo/reset (screen tests and recordings only)
ALERT_WEBHOOK_URL, ALERT_WEBHOOK_TOKEN   where production errors are posted (optional)
"""
import os
from pathlib import Path
from urllib.parse import urlparse

BASE_DIR = Path(__file__).resolve().parent.parent
DEBUG = os.environ.get("DEBUG") == "1"
# Routes POST api/demo/reset, which signs anyone in as the demo account: screen tests and recordings only.
DEMO_RESET = os.environ.get("DEMO_RESET") == "1"
SECRET_KEY = os.environ.get("SECRET_KEY") or ("dev-only-key" if DEBUG else None)
if not SECRET_KEY:
    raise RuntimeError("SECRET_KEY is required when DEBUG is off")
ALLOWED_HOSTS = [h for h in os.environ.get("ALLOWED_HOSTS", "localhost,127.0.0.1").split(",") if h]
CSRF_TRUSTED_ORIGINS = [o for o in os.environ.get("CSRF_TRUSTED_ORIGINS", "").split(",") if o]

INSTALLED_APPS = [
    "django.contrib.auth",
    "django.contrib.contenttypes",
    "django.contrib.sessions",
    "django.contrib.staticfiles",
    "rest_framework",
    "accounts",
    "planner",
]
MIDDLEWARE = [
    "django.middleware.security.SecurityMiddleware",
    "django.contrib.sessions.middleware.SessionMiddleware",
    "django.middleware.common.CommonMiddleware",
    "django.middleware.csrf.CsrfViewMiddleware",
    "django.contrib.auth.middleware.AuthenticationMiddleware",
]
ROOT_URLCONF = "config.urls"
WSGI_APPLICATION = "config.wsgi.application"
TEMPLATES = []

url = os.environ.get("DATABASE_URL")
if not url:
    raise RuntimeError("DATABASE_URL is required (postgres://user:password@host:port/name)")
db = urlparse(url)
DATABASES = {
    "default": {
        "ENGINE": "django.db.backends.postgresql",
        "NAME": db.path.lstrip("/"),
        "USER": db.username or "",
        "PASSWORD": db.password or "",
        "HOST": db.hostname or "",
        "PORT": str(db.port or 5432),
    }
}
DEFAULT_AUTO_FIELD = "django.db.models.BigAutoField"
AUTH_USER_MODEL = "accounts.User"
AUTH_PASSWORD_VALIDATORS = [
    {"NAME": "django.contrib.auth.password_validation.MinimumLengthValidator", "OPTIONS": {"min_length": 8}},
    {"NAME": "django.contrib.auth.password_validation.CommonPasswordValidator"},
]

STATIC_URL = "static/"
STATIC_ROOT = BASE_DIR / "staticfiles"
MEDIA_URL = "/media/"
MEDIA_ROOT = Path(os.environ.get("MEDIA_ROOT", BASE_DIR / "media"))
DATA_UPLOAD_MAX_MEMORY_SIZE = 25 * 1024 * 1024
FILE_UPLOAD_MAX_MEMORY_SIZE = 5 * 1024 * 1024
MAX_ATTACHMENT_BYTES = 25 * 1024 * 1024

USE_TZ = True
TIME_ZONE = "UTC"
LANGUAGE_CODE = "en-us"

SESSION_COOKIE_SAMESITE = "Lax"
CSRF_COOKIE_SAMESITE = "Lax"
SESSION_COOKIE_SECURE = not DEBUG
CSRF_COOKIE_SECURE = not DEBUG

REST_FRAMEWORK = {
    "DEFAULT_AUTHENTICATION_CLASSES": ["config.auth.Session"],
    "DEFAULT_PERMISSION_CLASSES": ["rest_framework.permissions.IsAuthenticated"],
    "DEFAULT_PAGINATION_CLASS": None,
    "EXCEPTION_HANDLER": "config.errors.exception_handler",
}

LOGGING = {
    "version": 1,
    "disable_existing_loggers": False,
    "formatters": {"plain": {"format": "%(asctime)s %(levelname)s %(name)s %(message)s"}},
    "handlers": {
        "console": {"class": "logging.StreamHandler", "formatter": "plain"},
        "alert": {"class": "config.alerts.AlertHandler", "level": "ERROR"},
    },
    "root": {"handlers": ["console", "alert"], "level": "INFO"},
    "loggers": {"django.request": {"handlers": ["console", "alert"], "level": "WARNING", "propagate": False}},
}
