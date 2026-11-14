import logging

from rest_framework.authentication import SessionAuthentication
from rest_framework.exceptions import PermissionDenied

log = logging.getLogger("taskscape.csrf")
SHOWN = ("HTTP_HOST", "HTTP_ORIGIN", "HTTP_X_FORWARDED_HOST", "HTTP_X_FORWARDED_PROTO", "HTTP_FORWARDED", "wsgi.url_scheme")


class Session(SessionAuthentication):
    """Session auth that answers 401, not 403, to a visitor who is not signed in (as `auth/me` does),
    and logs a CSRF refusal with the headers that decide it (host, origin, what a proxy forwarded)."""

    def authenticate_header(self, request):
        return "Session"

    def enforce_csrf(self, request):
        try:
            super().enforce_csrf(request)
        except PermissionDenied as e:
            meta = request._request.META
            log.warning("%s %s %s | %s", e.detail, request.method, request.path,
                        " ".join(f"{k}={meta.get(k)!r}" for k in SHOWN if k in meta))
            raise
