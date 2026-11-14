"""POST api/client-errors {message, detail?, url?}: a failure in the browser, logged at ERROR so it
reaches the alert webhook like a server error. Open to visitors (a crash on the sign-in page counts
too), without CSRF (sent with sendBeacon), and rate limited per client."""
import logging

from rest_framework.decorators import api_view, authentication_classes, permission_classes, throttle_classes
from rest_framework.response import Response
from rest_framework.throttling import AnonRateThrottle

log = logging.getLogger("taskscape.client")


class PerClient(AnonRateThrottle):
    rate = "20/hour"

    def get_cache_key(self, request, view):
        return self.cache_format % {"scope": "client-errors", "ident": self.get_ident(request)}


@api_view(["POST"])
@authentication_classes([])
@permission_classes([])
@throttle_classes([PerClient])
def client_errors(request):
    message = str(request.data.get("message", ""))[:500]
    if not message:
        return Response({"message": ["This field is required."]}, status=400)
    user = request.session.get("_auth_user_id", "-")
    log.error("browser error on %s (user %s): %s\n%s", str(request.data.get("url", "?"))[:200], user, message, str(request.data.get("detail", ""))[:4000])
    return Response(status=204)
