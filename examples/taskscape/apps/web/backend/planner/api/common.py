"""What every resource shares: everything a user can reach goes through their own projects."""
from rest_framework.exceptions import NotFound

from ..models import Attachment, Note, Project, Task, TaskList


def projects_of(user):
    return Project.objects.filter(owner=user)


def lists_of(user):
    return TaskList.objects.filter(project__owner=user)


def tasks_of(user, include_trashed=False):
    qs = Task.objects.filter(list__project__owner=user)
    return qs if include_trashed else qs.filter(deleted_at__isnull=True)


def notes_of(user):
    return Note.objects.filter(task__list__project__owner=user)


def attachments_of(user):
    return Attachment.objects.filter(task__list__project__owner=user)


def owned(qs, pk, what="item"):
    """The row with this id, if the user owns it; otherwise 404 (never 403, so ids don't leak)."""
    obj = qs.filter(pk=pk).first()
    if obj is None:
        raise NotFound(f"No such {what}.")
    return obj


def next_order(qs):
    last = qs.order_by("-sort_order").values_list("sort_order", flat=True).first()
    return 0 if last is None else last + 1
