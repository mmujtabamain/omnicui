"""Session sign-up, sign-in and the signed-in user's settings and shortcuts."""
import logging

from django.contrib.auth import authenticate, login, logout
from django.db import transaction
from django.middleware.csrf import get_token
from rest_framework import serializers, status
from rest_framework.decorators import api_view, permission_classes
from rest_framework.permissions import AllowAny
from rest_framework.response import Response

from planner.models import Project, TaskList

from .models import Shortcut, User, UserSettings

log = logging.getLogger("taskscape.accounts")


class SignupSerializer(serializers.Serializer):
    email = serializers.EmailField()
    password = serializers.CharField(min_length=8, write_only=True)
    name = serializers.CharField(max_length=150, required=False, allow_blank=True)

    def validate_email(self, v):
        if User.objects.filter(email__iexact=v).exists():
            raise serializers.ValidationError("An account with this email already exists.")
        return v.lower()


class SettingsSerializer(serializers.ModelSerializer):
    class Meta:
        model = UserSettings
        fields = ["theme", "reduced_motion", "show_completed", "last_project", "last_list"]

    def validate(self, data):
        user = self.context["request"].user
        p, l = data.get("last_project"), data.get("last_list")
        if p and p.owner_id != user.id:
            raise serializers.ValidationError({"last_project": "Not your project."})
        if l and l.project.owner_id != user.id:
            raise serializers.ValidationError({"last_list": "Not your list."})
        return data


def me_payload(user):
    return {"id": user.id, "email": user.email, "name": user.first_name}


def settings_of(user):
    s, _ = UserSettings.objects.get_or_create(user=user)
    return s


@api_view(["GET"])
@permission_classes([AllowAny])
def csrf(request):
    """Sets the CSRF cookie; the frontend calls this once before its first write."""
    return Response({"csrfToken": get_token(request)})


@api_view(["POST"])
@permission_classes([AllowAny])
def signup(request):
    s = SignupSerializer(data=request.data)
    s.is_valid(raise_exception=True)
    with transaction.atomic():
        user = User.objects.create_user(username=s.validated_data["email"], email=s.validated_data["email"],
                                        password=s.validated_data["password"], first_name=s.validated_data.get("name", ""))
        UserSettings.objects.create(user=user)
        # A new account starts with one project and one list, as the reference's first launch does.
        project = Project.objects.create(owner=user, name="Personal")
        TaskList.objects.create(project=project, name="Inbox")
    login(request, user)
    log.info("signup user=%s", user.id)
    return Response(me_payload(user), status=status.HTTP_201_CREATED)


@api_view(["POST"])
@permission_classes([AllowAny])
def signin(request):
    email, password = request.data.get("email", ""), request.data.get("password", "")
    user = User.objects.filter(email__iexact=email).first()
    user = authenticate(request, username=user.username, password=password) if user else None
    if not user:
        log.info("sign-in refused for %s", email[:3] + "…")
        return Response({"detail": "Wrong email or password."}, status=status.HTTP_400_BAD_REQUEST)
    login(request, user)
    return Response(me_payload(user))


@api_view(["POST"])
def signout(request):
    logout(request)
    return Response(status=status.HTTP_204_NO_CONTENT)


@api_view(["GET"])
@permission_classes([AllowAny])
def me(request):
    if not request.user.is_authenticated:
        return Response({"detail": "Not signed in."}, status=status.HTTP_401_UNAUTHORIZED)
    return Response(me_payload(request.user))


@api_view(["GET", "PATCH"])
def user_settings(request):
    s = settings_of(request.user)
    if request.method == "PATCH":
        ser = SettingsSerializer(s, data=request.data, partial=True, context={"request": request})
        ser.is_valid(raise_exception=True)
        ser.save()
        return Response(ser.data)
    return Response(SettingsSerializer(s, context={"request": request}).data)


@api_view(["GET", "PUT"])
def shortcuts(request):
    """GET: the user's own bindings. PUT {command, accel}: set one ("" unbinds; null resets to the default)."""
    if request.method == "PUT":
        command, accel = request.data.get("command"), request.data.get("accel")
        if not command or len(command) > 40:
            return Response({"detail": "command is required."}, status=status.HTTP_400_BAD_REQUEST)
        if accel is None:
            Shortcut.objects.filter(user=request.user, command=command).delete()
        else:
            clash = Shortcut.objects.filter(user=request.user, accel=accel).exclude(command=command).exclude(accel="").first()
            if clash:
                return Response({"detail": f"Already used by {clash.command}."}, status=status.HTTP_409_CONFLICT)
            Shortcut.objects.update_or_create(user=request.user, command=command, defaults={"accel": accel})
    rows = Shortcut.objects.filter(user=request.user).values("command", "accel")
    return Response(list(rows))
