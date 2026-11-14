from django.contrib.auth.models import AbstractUser
from django.db import models


class User(AbstractUser):
    """Signs in with email; username stays for Django's admin tooling."""

    email = models.EmailField(unique=True)


class Choice3(models.TextChoices):
    SYSTEM = "system"
    ON = "on"
    OFF = "off"


class Theme(models.TextChoices):
    SYSTEM = "system"
    LIGHT = "light"
    DARK = "dark"


class UserSettings(models.Model):
    """Per-account preferences, so they follow the user across devices (decision 17)."""

    user = models.OneToOneField(User, on_delete=models.CASCADE, related_name="settings")
    theme = models.CharField(max_length=8, choices=Theme.choices, default=Theme.SYSTEM)
    reduced_motion = models.CharField(max_length=8, choices=Choice3.choices, default=Choice3.SYSTEM)
    show_completed = models.BooleanField(default=True)
    last_project = models.ForeignKey("planner.Project", null=True, blank=True, on_delete=models.SET_NULL, related_name="+")
    last_list = models.ForeignKey("planner.TaskList", null=True, blank=True, on_delete=models.SET_NULL, related_name="+")
    updated_at = models.DateTimeField(auto_now=True)


class Shortcut(models.Model):
    """A user's own binding for an in-app command; commands without a row use the web default."""

    user = models.ForeignKey(User, on_delete=models.CASCADE, related_name="shortcuts")
    command = models.CharField(max_length=40)
    accel = models.CharField(max_length=40, blank=True)  # "" means intentionally unbound

    class Meta:
        constraints = [models.UniqueConstraint(fields=["user", "command"], name="one_binding_per_command")]
