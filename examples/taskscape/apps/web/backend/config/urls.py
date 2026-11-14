import importlib
import logging

from django.conf import settings
from django.conf.urls.static import static
from django.urls import include, path
from rest_framework.routers import DefaultRouter

from accounts import api as accounts
from config.client_errors import client_errors

log = logging.getLogger("taskscape.urls")
RESOURCES = [
    ("projects", "ProjectViewSet", "project"),
    ("lists", "TaskListViewSet", "list"),
    ("tasks", "TaskViewSet", "task"),
    ("notes", "NoteViewSet", "note"),
    ("attachments", "AttachmentViewSet", "attachment"),
    ("trash", "TrashViewSet", "trash"),
]

router = DefaultRouter(trailing_slash=False)
for module, viewset, basename in RESOURCES:
    # One broken resource must not take the others down; it is logged (and alerted) instead.
    try:
        router.register(module, getattr(importlib.import_module(f"planner.api.{module}"), viewset), basename=basename)
    except Exception:
        log.exception("resource %s failed to load", module)

urlpatterns = [
    path("api/auth/csrf", accounts.csrf),
    path("api/auth/signup", accounts.signup),
    path("api/auth/signin", accounts.signin),
    path("api/auth/signout", accounts.signout),
    path("api/auth/me", accounts.me),
    path("api/settings", accounts.user_settings),
    path("api/shortcuts", accounts.shortcuts),
    path("api/client-errors", client_errors),
    path("api/", include(router.urls)),
]
if settings.DEMO_RESET:
    from planner.demo import api as demo_api

    urlpatterns.insert(0, path("api/demo/reset", demo_api.reset))
if settings.DEBUG:
    urlpatterns += static(settings.MEDIA_URL, document_root=settings.MEDIA_ROOT)
