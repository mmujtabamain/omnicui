from rest_framework.authentication import SessionAuthentication


class Session(SessionAuthentication):
    """Session auth that answers 401, not 403, to a visitor who is not signed in (as `auth/me` does)."""

    def authenticate_header(self, request):
        return "Session"
