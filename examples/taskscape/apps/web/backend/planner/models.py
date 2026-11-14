"""The reference's data model (common/schema.hcl), made multi-user: every row belongs to a user
through its project. Ordering is an explicit sort_order, as in the reference; the Trash is a
soft delete (deleted_at) on tasks."""
from django.conf import settings
from django.db import models


class Project(models.Model):
    owner = models.ForeignKey(settings.AUTH_USER_MODEL, on_delete=models.CASCADE, related_name="projects")
    name = models.CharField(max_length=200)
    sort_order = models.IntegerField(default=0)
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    class Meta:
        ordering = ["sort_order", "created_at"]
        indexes = [models.Index(fields=["owner", "sort_order"])]


class TaskList(models.Model):
    project = models.ForeignKey(Project, on_delete=models.CASCADE, related_name="lists")
    name = models.CharField(max_length=200)
    sort_order = models.IntegerField(default=0)
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    class Meta:
        ordering = ["sort_order", "created_at"]
        indexes = [models.Index(fields=["project", "sort_order"])]


class Task(models.Model):
    list = models.ForeignKey(TaskList, on_delete=models.CASCADE, related_name="tasks")
    parent = models.ForeignKey("self", null=True, blank=True, on_delete=models.CASCADE, related_name="children")
    title = models.CharField(max_length=500)
    notes = models.TextField(blank=True, default="")  # the one-line subtitle under the title
    done = models.BooleanField(default=False)
    done_at = models.DateTimeField(null=True, blank=True)
    sort_order = models.IntegerField(default=0)
    deleted_at = models.DateTimeField(null=True, blank=True)  # in the Trash when set
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    class Meta:
        ordering = ["sort_order", "created_at"]
        indexes = [
            models.Index(fields=["list", "parent", "sort_order"]),
            models.Index(fields=["deleted_at"]),
        ]

    def subtree_ids(self):
        """This task and every descendant, alive or not."""
        ids, frontier = [self.id], [self.id]
        while frontier:
            frontier = list(Task.objects.filter(parent_id__in=frontier).values_list("id", flat=True))
            ids += frontier
        return ids


class Note(models.Model):
    """A rich-text note block on a task (sanitized HTML)."""

    task = models.ForeignKey(Task, on_delete=models.CASCADE, related_name="note_items")
    content = models.TextField(blank=True, default="")
    sort_order = models.IntegerField(default=0)
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    class Meta:
        ordering = ["sort_order", "created_at"]


class Attachment(models.Model):
    class Kind(models.TextChoices):
        FILE = "file"  # uploaded to MEDIA_ROOT
        LINK = "link"  # a URL

    task = models.ForeignKey(Task, on_delete=models.CASCADE, related_name="attachments")
    kind = models.CharField(max_length=8, choices=Kind.choices)
    name = models.CharField(max_length=255)
    file = models.FileField(upload_to="attachments/%Y/%m/", blank=True)
    url = models.URLField(max_length=2000, blank=True)
    size = models.BigIntegerField(default=0)
    mime = models.CharField(max_length=120, blank=True)
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        ordering = ["created_at"]
