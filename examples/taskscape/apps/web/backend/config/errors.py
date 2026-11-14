import logging

from rest_framework.views import exception_handler as drf_handler

log = logging.getLogger("taskscape.api")


def exception_handler(exc, context):
    """DRF's handler, plus a log line for every server error so it reaches the alert webhook."""
    response = drf_handler(exc, context)
    view = context.get("view").__class__.__name__ if context.get("view") else "?"
    if response is None:
        log.exception("unhandled error in %s", view)
    elif response.status_code >= 500:
        log.error("server error %s in %s: %s", response.status_code, view, exc)
    return response
