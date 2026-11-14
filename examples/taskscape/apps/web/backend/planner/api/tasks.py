"""The tasks resource (API.md, "Tasks: `tasks`"). A task sits in a list, optionally under a parent task.
Completing and trashing act on the live subtree; the Trash is a soft delete (deleted_at). Every write
logs one INFO line: `tasks <action> <ids> <user id>`."""
import logging

from django.db import transaction
from django.db.models import Q
from django.utils import timezone
from rest_framework import permissions, serializers, viewsets
from rest_framework.decorators import action
from rest_framework.exceptions import NotFound, ValidationError
from rest_framework.response import Response

from ..models import Attachment, Note, Task
from .common import lists_of, next_order, owned, projects_of, tasks_of

log = logging.getLogger("taskscape.tasks")


# --- serializers -----------------------------------------------------------------------------------

class AttachmentRefSerializer(serializers.ModelSerializer):
    """An attachment as nested in a task (output only; the attachments resource owns its writes)."""

    url = serializers.SerializerMethodField()

    class Meta:
        model = Attachment
        fields = ["id", "task", "kind", "name", "url", "size", "mime", "created_at"]

    def get_url(self, obj):
        if obj.kind == Attachment.Kind.FILE:
            return obj.file.url
        return obj.url


class NoteRefSerializer(serializers.ModelSerializer):
    """A note as nested in a task (output only; the notes resource owns its writes)."""

    class Meta:
        model = Note
        fields = ["id", "task", "content", "sort_order", "created_at", "updated_at"]


class TaskSerializer(serializers.ModelSerializer):
    attachments = AttachmentRefSerializer(many=True, read_only=True)
    note_items = NoteRefSerializer(many=True, read_only=True)

    class Meta:
        model = Task
        fields = ["id", "list", "parent", "title", "notes", "done", "done_at", "sort_order",
                  "created_at", "updated_at", "attachments", "note_items"]
        read_only_fields = ["id", "list", "parent", "done_at", "sort_order", "created_at", "updated_at"]


class TaskCreateSerializer(serializers.Serializer):
    # list and parent are plain ids here: the view looks them up so a foreign id answers 404, not 400.
    list = serializers.IntegerField()
    title = serializers.CharField(max_length=500)
    notes = serializers.CharField(required=False, allow_blank=True, trim_whitespace=False)
    parent = serializers.IntegerField(required=False, allow_null=True)


class TaskUpdateSerializer(serializers.Serializer):
    title = serializers.CharField(max_length=500, required=False)
    notes = serializers.CharField(required=False, allow_blank=True, trim_whitespace=False)
    done = serializers.BooleanField(required=False)


class TaskMoveSerializer(serializers.Serializer):
    list = serializers.IntegerField(required=False)
    parent = serializers.IntegerField(required=False, allow_null=True)
    sort_order = serializers.IntegerField(required=False)


class TaskReorderSerializer(serializers.Serializer):
    sort_order = serializers.IntegerField()


class TaskBulkSerializer(serializers.Serializer):
    ids = serializers.ListField(child=serializers.IntegerField(), allow_empty=False)
    action = serializers.ChoiceField(choices=["done", "undone", "trash", "move"])
    list = serializers.IntegerField(required=False)

    def validate(self, attrs):
        if attrs["action"] == "move" and "list" not in attrs:
            raise ValidationError({"list": ["This field is required for move."]})
        return attrs


# --- helpers ---------------------------------------------------------------------------------------

def _int_or_none(raw):
    try:
        return int(raw)
    except (TypeError, ValueError):
        return None


def _task_or_404(user, pk):
    """A live task of the user's; a bad id, a foreign id and a trashed id all answer 404."""
    return owned(tasks_of(user), _int_or_none(pk), "task")


def _parent_in_list(user, parent_id, task_list):
    """The parent must be a live task in task_list. Foreign or missing ids are 404."""
    parent = owned(tasks_of(user, include_trashed=True), parent_id, "task")
    if parent.deleted_at is not None or parent.list_id != task_list.id:
        raise ValidationError({"parent": ["Parent must be a live task in the same list."]})
    return parent


def _live_subtree_ids(task_id):
    """The task and its live descendants, the task first. Trashed branches are skipped."""
    ids = [task_id]
    frontier = [task_id]
    while frontier:
        frontier = list(Task.objects.filter(parent_id__in=frontier, deleted_at__isnull=True)
                        .values_list("id", flat=True))
        ids += frontier
    return ids


def _complete(task_ids, now):
    """Completes these tasks and their live descendants; done_at is stamped only where it was not done."""
    ids = []
    for tid in task_ids:
        ids += _live_subtree_ids(tid)
    Task.objects.filter(id__in=ids, done=False).update(done=True, done_at=now, updated_at=now)


def _uncomplete(task_ids, now):
    """Un-completes exactly these tasks, not their descendants."""
    Task.objects.filter(id__in=task_ids).update(done=False, done_at=None, updated_at=now)


def _move_to_list_top(user, ids, target, now):
    """Bulk move: each selected task goes to the top level of target with its subtree. A selected task
    whose parent is also selected travels with that parent instead."""
    selected = set(ids)
    by_id = {t.id: t for t in Task.objects.filter(id__in=ids)}
    position = next_order(tasks_of(user).filter(list=target, parent=None))
    for tid in ids:
        t = by_id[tid]
        if t.parent_id in selected:
            continue
        Task.objects.filter(id__in=t.subtree_ids()).update(list=target, updated_at=now)
        t.list = target
        t.parent = None
        t.sort_order = position
        t.save()
        position += 1


def _serialize(ids):
    """The tasks with these ids, in the order given."""
    found = {t.id: t for t in Task.objects.filter(id__in=ids).prefetch_related("attachments", "note_items")}
    return [TaskSerializer(found[i]).data for i in ids if i in found]


def _log_write(action_name, ids, user):
    log.info("tasks %s %s %s", action_name, ",".join(str(i) for i in ids), user.id)


# --- viewset ---------------------------------------------------------------------------------------

class TaskViewSet(viewsets.GenericViewSet):
    permission_classes = [permissions.IsAuthenticated]
    serializer_class = TaskSerializer

    def list(self, request):
        user = request.user
        qs = tasks_of(user)

        list_raw = request.query_params.get("list", "")
        if list_raw != "":
            qs = qs.filter(list_id=owned(lists_of(user), _int_or_none(list_raw), "list").id)

        project_raw = request.query_params.get("project", "")
        if project_raw != "":
            qs = qs.filter(list__project_id=owned(projects_of(user), _int_or_none(project_raw), "project").id)

        text = request.query_params.get("q", "").strip()
        if text:
            qs = qs.filter(Q(title__icontains=text) | Q(notes__icontains=text))

        qs = qs.prefetch_related("attachments", "note_items").order_by("sort_order", "created_at", "id")
        return Response(TaskSerializer(qs, many=True).data)

    def create(self, request):
        user = request.user
        ser = TaskCreateSerializer(data=request.data)
        ser.is_valid(raise_exception=True)
        d = ser.validated_data
        task_list = owned(lists_of(user), d["list"], "list")
        parent = None
        if d.get("parent") is not None:
            parent = _parent_in_list(user, d["parent"], task_list)
        task = Task.objects.create(
            list=task_list,
            parent=parent,
            title=d["title"],
            notes=d.get("notes", ""),
            sort_order=next_order(tasks_of(user).filter(list=task_list, parent=parent)),
        )
        _log_write("create", [task.id], user)
        return Response(TaskSerializer(task).data, status=201)

    def partial_update(self, request, pk=None):
        user = request.user
        task = _task_or_404(user, pk)
        ser = TaskUpdateSerializer(data=request.data, partial=True)
        ser.is_valid(raise_exception=True)
        d = ser.validated_data
        now = timezone.now()
        with transaction.atomic():
            for field in ("title", "notes"):
                if field in d:
                    setattr(task, field, d[field])
            task.save()
            if "done" in d:
                if d["done"]:
                    _complete([task.id], now)
                else:
                    _uncomplete([task.id], now)
        task.refresh_from_db()
        _log_write("update", [task.id], user)
        return Response(TaskSerializer(task).data)

    def destroy(self, request, pk=None):
        user = request.user
        task = _task_or_404(user, pk)
        now = timezone.now()
        ids = _live_subtree_ids(task.id)
        with transaction.atomic():
            Task.objects.filter(id__in=ids).update(deleted_at=now, updated_at=now)
        _log_write("trash", ids, user)
        return Response({"trashed": ids})

    @action(detail=True, methods=["post"])
    def move(self, request, pk=None):
        user = request.user
        task = _task_or_404(user, pk)
        ser = TaskMoveSerializer(data=request.data)
        ser.is_valid(raise_exception=True)
        d = ser.validated_data

        target = task.list
        if "list" in d:
            target = owned(lists_of(user), d["list"], "list")
        list_changed = target.id != task.list_id

        if "parent" in d:
            new_parent_id = d["parent"]
        else:
            new_parent_id = None if list_changed else task.parent_id

        new_parent = None
        if new_parent_id is not None:
            if new_parent_id in task.subtree_ids():
                raise ValidationError({"parent": ["A task can't be moved under itself or its own descendant."]})
            new_parent = _parent_in_list(user, new_parent_id, target)
        parent_changed = new_parent_id != task.parent_id

        if "sort_order" in d:
            sort_order = d["sort_order"]
        elif list_changed or parent_changed:
            sort_order = next_order(tasks_of(user).filter(list=target, parent=new_parent))
        else:
            sort_order = task.sort_order

        now = timezone.now()
        with transaction.atomic():
            if list_changed:
                # the whole subtree comes along, trashed branches included, so parent and child share a list
                Task.objects.filter(id__in=task.subtree_ids()).update(list=target, updated_at=now)
            task.list = target
            task.parent = new_parent
            task.sort_order = sort_order
            task.save()
        task.refresh_from_db()
        _log_write("move", [task.id], user)
        return Response(TaskSerializer(task).data)

    @action(detail=True, methods=["post"])
    def reorder(self, request, pk=None):
        user = request.user
        task = _task_or_404(user, pk)
        ser = TaskReorderSerializer(data=request.data)
        ser.is_valid(raise_exception=True)
        task.sort_order = ser.validated_data["sort_order"]
        task.save()
        _log_write("reorder", [task.id], user)
        return Response(TaskSerializer(task).data)

    @action(detail=False, methods=["post"])
    def bulk(self, request):
        user = request.user
        ser = TaskBulkSerializer(data=request.data)
        ser.is_valid(raise_exception=True)
        d = ser.validated_data
        ids = list(dict.fromkeys(d["ids"]))
        act = d["action"]
        now = timezone.now()

        with transaction.atomic():
            found = set(tasks_of(user).filter(id__in=ids).values_list("id", flat=True))
            if len(found) != len(ids):
                raise NotFound("No such task.")

            if act == "trash":
                trashed = list(dict.fromkeys(sid for tid in ids for sid in _live_subtree_ids(tid)))
                Task.objects.filter(id__in=trashed).update(deleted_at=now, updated_at=now)
                _log_write("bulk-trash", ids, user)
                return Response({"trashed": trashed})

            if act == "done":
                _complete(ids, now)
            elif act == "undone":
                _uncomplete(ids, now)
            else:  # move
                target = owned(lists_of(user), d["list"], "list")
                _move_to_list_top(user, ids, target, now)

        _log_write(f"bulk-{act}", ids, user)
        return Response({"tasks": _serialize(ids)})

    @action(detail=True, methods=["get"])
    def copy(self, request, pk=None):
        task = _task_or_404(request.user, pk)
        lines = []

        def walk(node, depth):
            mark = "x" if node.done else " "
            lines.append(f"{'  ' * depth}- [{mark}] {node.title}")
            for child in tasks_of(request.user).filter(parent_id=node.id).order_by("sort_order", "created_at", "id"):
                walk(child, depth + 1)

        walk(task, 0)
        return Response({"text": "\n".join(lines)})
