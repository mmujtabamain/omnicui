"""Behind a proxy that terminates TLS (TRUST_PROXY=1), a request reaches Django as http. When the
proxy says so with X-Forwarded-Proto, SECURE_PROXY_SSL_HEADER handles it; some proxies do not send
it. Then a write from https://<this host> fails Django's CSRF origin check (https vs http), so a
request whose Origin is https on this very host is taken as https. Only the scheme is inferred: the
host must still match, so no other origin is trusted."""
from django.conf import settings


class HttpsFromSameOrigin:
    def __init__(self, get_response):
        self.get_response = get_response

    def __call__(self, request):
        if settings.TRUST_PROXY and not request.is_secure():
            origin = request.META.get("HTTP_ORIGIN", "")
            if origin == f"https://{request.get_host()}":
                request.META["wsgi.url_scheme"] = "https"
        return self.get_response(request)
