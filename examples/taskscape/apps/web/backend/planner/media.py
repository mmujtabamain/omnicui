"""GET /media/<path>: an attachment's stored file, to its owner only (anyone else gets 404)."""
import logging
import mimetypes

from django.http import FileResponse, Http404

from planner.models import Attachment

log = logging.getLogger("taskscape.media")


def attachment_file(request, path):
    if not request.user.is_authenticated:
        raise Http404
    a = Attachment.objects.filter(file=path, task__list__project__owner=request.user).first()
    if not a or not a.file:
        raise Http404
    try:
        f = a.file.open("rb")
    except FileNotFoundError:
        log.error("attachment %s file missing: %s", a.id, path)
        raise Http404
    response = FileResponse(f, content_type=a.mime or mimetypes.guess_type(path)[0] or "application/octet-stream")
    response["Content-Disposition"] = f'inline; filename="{a.name}"'
    response["X-Content-Type-Options"] = "nosniff"
    return response
