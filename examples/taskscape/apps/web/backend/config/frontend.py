"""Serves the built web app (FRONTEND_DIST) from the same origin as the API: its assets as files,
and index.html for every other path, so the app's own routes (/signin, /) load on refresh."""
from pathlib import Path

from django.conf import settings
from django.http import FileResponse, Http404
from django.views.static import serve


def asset(request, path):
    return serve(request, path, document_root=Path(settings.FRONTEND_DIST) / "assets")


def index(request, path=""):
    page = Path(settings.FRONTEND_DIST) / "index.html"
    if not page.exists():
        raise Http404
    response = FileResponse(page.open("rb"), content_type="text/html")
    response["Cache-Control"] = "no-cache"
    return response
