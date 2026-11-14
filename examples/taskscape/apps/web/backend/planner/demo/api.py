"""POST /api/demo/reset {dataset}: signs in the demo account with fresh demo data. Only routed when
DEMO_RESET=1 (screen tests and recordings); never in production."""
import logging

from django.contrib.auth import get_user_model, login
from rest_framework.decorators import api_view, authentication_classes, permission_classes
from rest_framework.response import Response

from planner import demo

log = logging.getLogger("taskscape.demo")
EMAIL = "demo@taskscape.test"


@api_view(["POST"])
@authentication_classes([])
@permission_classes([])
def reset(request):
    name = request.data.get("dataset", "standard")
    if name not in demo.DATASETS:
        return Response({"dataset": [f"one of {', '.join(sorted(demo.DATASETS))}"]}, status=400)
    User = get_user_model()
    user = User.objects.filter(email=EMAIL).first() or User.objects.create_user(username=EMAIL, email=EMAIL, password="taskscape-demo", first_name="Demo")
    # Accounts a test created (a sign-up) go too, so every test starts from the same state.
    User.objects.exclude(pk=user.pk).delete()
    counts = demo.load(user, name)
    login(request, user, backend="django.contrib.auth.backends.ModelBackend")
    log.info("demo reset %s %s", name, user.id)
    return Response({"dataset": name, **counts})
