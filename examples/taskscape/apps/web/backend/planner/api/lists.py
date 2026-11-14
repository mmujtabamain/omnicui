"""The lists resource (API.md, "Lists: `lists`"): a project's ordered lists with live task counts,
the atomic import of a task tree, and the export of a list as a tree. Everything is scoped to the
signed-in user's projects; another user's row answers 404."""
import logging
from collections import defaultdict

from django.core.files.storage import default_storage
from django.db import transaction
from django.db.models import Count, Q
from django.utils import timezone
from rest_framework import serializers, status, viewsets
from rest_framework.decorators import action
from rest_framework.exceptions import NotFound, ValidationError
from rest_framework.permissions import IsAuthenticated
from rest_framework.response import Response

from ..models import Attachment, Task, TaskList
from .common import lists_of, next_order, owned, projects_of, tasks_of

log = logging.getLogger("taskscape.lists")

SORT_MIN, SORT_MAX = -(2**31), 2**31 - 1


def with_counts(qs):
    """Annotates task_count and done_count: live tasks only, top-level and nested alike."""
    live = Q(tasks__deleted_at__isnull=True)
    return qs.annotate(
        task_count=Count("tasks", filter=live),
        done_count=Count("tasks", filter=live & Q(tasks__done=True)),
    )


def _id(value, what):
    """A path id as an int; one that cannot be an id is simply not found."""
    try:
        return int(value)
    except (TypeError, ValueError):
        raise NotFound(f"No such {what}.")


class TaskListSerializer(serializers.ModelSerializer):
    """A list as the API shows it. Needs the queryset to come from with_counts()."""

    project = serializers.IntegerField(source="project_id", read_only=True)
    task_count = serializers.IntegerField(read_only=True)
    done_count = serializers.IntegerField(read_only=True)

    class Meta:
        model = TaskList
        fields = ["id", "project", "name", "sort_order", "created_at", "updated_at", "task_count", "done_count"]
        read_only_fields = ["id", "name", "sort_order", "created_at", "updated_at"]


class ListCreateSerializer(serializers.Serializer):
    project = serializers.IntegerField()
    name = serializers.CharField(max_length=200)  # trimmed; blank after trimming is 400


class ListUpdateSerializer(serializers.Serializer):
    name = serializers.CharField(max_length=200, required=False)
    sort_order = serializers.IntegerField(required=False, min_value=SORT_MIN, max_value=SORT_MAX)
    project = serializers.IntegerField(required=False)


class ImportTaskSerializer(serializers.Serializer):
    title = serializers.CharField(max_length=500)
    notes = serializers.CharField(required=False, default="", allow_blank=True, trim_whitespace=False)
    done = serializers.BooleanField(required=False, default=False)
    children = serializers.ListField(required=False, default=list)

    def validate_children(self, value):
        nested = ImportTaskSerializer(data=value, many=True)
        nested.is_valid(raise_exception=True)
        return nested.validated_data


class ImportSerializer(serializers.Serializer):
    tasks = ImportTaskSerializer(many=True)


class TaskOutSerializer(serializers.ModelSerializer):
    """A task in the shape API.md gives for `tasks`. Only ever used for freshly created tasks,
    which have no attachments or note blocks yet."""

    list = serializers.IntegerField(source="list_id", read_only=True)
    parent = serializers.IntegerField(source="parent_id", read_only=True)
    attachments = serializers.SerializerMethodField()
    note_items = serializers.SerializerMethodField()

    class Meta:
        model = Task
        fields = [
            "id", "list", "parent", "title", "notes", "done", "done_at", "sort_order",
            "created_at", "updated_at", "attachments", "note_items",
        ]

    def get_attachments(self, obj):
        return []

    def get_note_items(self, obj):
        return []


def _append_tree(lst, nodes, parent, now):
    """Appends nodes under parent (None = top level) after its live children, recursively.
    Returns the created rows of this level, in order."""
    base = next_order(Task.objects.filter(list=lst, parent=parent, deleted_at__isnull=True))
    created = []
    for offset, node in enumerate(nodes):
        task = Task.objects.create(
            list=lst,
            parent=parent,
            title=node["title"],
            notes=node["notes"],
            done=node["done"],
            done_at=now if node["done"] else None,
            sort_order=base + offset,
        )
        _append_tree(lst, node["children"], task, now)
        created.append(task)
    return created


def _export_tree(tasks):
    """Nests live tasks (already in order) by parent. A live task whose parent is not live
    (cannot normally happen, since trashing takes whole subtrees) is exported at the top level."""
    live_ids = {t.id for t in tasks}
    children = defaultdict(list)
    roots = []
    for t in tasks:
        if t.parent_id is None or t.parent_id not in live_ids:
            roots.append(t)
        else:
            children[t.parent_id].append(t)

    def node(t):
        return {
            "title": t.title,
            "notes": t.notes,
            "done": t.done,
            "children": [node(c) for c in children[t.id]],
        }

    return [node(t) for t in roots]


def _remove_files(names):
    for name in names:
        default_storage.delete(name)


class TaskListViewSet(viewsets.GenericViewSet):
    permission_classes = [IsAuthenticated]
    serializer_class = TaskListSerializer

    def get_queryset(self):
        return with_counts(lists_of(self.request.user))

    def _detail(self, pk):
        return TaskListSerializer(self.get_queryset().get(pk=pk)).data

    def list(self, request):
        qs = self.get_queryset()
        raw = request.query_params.get("project")
        if raw is not None:
            try:
                project_id = int(raw)
            except ValueError:
                raise ValidationError({"project": ["A valid integer is required."]})
            owned(projects_of(request.user), project_id, "project")
            qs = qs.filter(project_id=project_id)
        qs = qs.order_by("project__sort_order", "project__created_at", "project_id", "sort_order", "created_at", "id")
        return Response(TaskListSerializer(qs, many=True).data)

    def create(self, request):
        body = ListCreateSerializer(data=request.data)
        body.is_valid(raise_exception=True)
        user = request.user
        project = owned(projects_of(user), body.validated_data["project"], "project")
        lst = TaskList.objects.create(
            project=project,
            name=body.validated_data["name"],
            sort_order=next_order(TaskList.objects.filter(project=project)),
        )
        log.info("lists create %s %s", lst.pk, user.pk)
        return Response(self._detail(lst.pk), status=status.HTTP_201_CREATED)

    def partial_update(self, request, pk=None):
        lst = owned(self.get_queryset(), _id(pk, "list"), "list")
        body = ListUpdateSerializer(data=request.data, partial=True)
        body.is_valid(raise_exception=True)
        data = body.validated_data
        user = request.user

        target = None
        if "project" in data and data["project"] != lst.project_id:
            target = owned(projects_of(user), data["project"], "project")

        if "name" in data:
            lst.name = data["name"]
        if "sort_order" in data:
            lst.sort_order = data["sort_order"]
        if target is not None:
            lst.project = target
            if "sort_order" not in data:
                # Moved without an explicit position: goes to the end of its new project.
                lst.sort_order = next_order(TaskList.objects.filter(project=target))
        lst.save()
        log.info("lists update %s %s", lst.pk, user.pk)
        return Response(self._detail(lst.pk))

    def destroy(self, request, pk=None):
        lst = owned(self.get_queryset(), _id(pk, "list"), "list")
        user = request.user
        stored = list(
            Attachment.objects.filter(task__list=lst, kind=Attachment.Kind.FILE)
            .exclude(file="")
            .values_list("file", flat=True)
        )
        with transaction.atomic():
            lst.delete()
            if stored:
                transaction.on_commit(lambda: _remove_files(stored))
        log.info("lists delete %s %s", pk, user.pk)
        return Response(status=status.HTTP_204_NO_CONTENT)

    @action(detail=True, methods=["post"], url_path="import")
    def import_tasks(self, request, pk=None):
        lst = owned(self.get_queryset(), _id(pk, "list"), "list")
        body = ImportSerializer(data=request.data)
        body.is_valid(raise_exception=True)
        user = request.user
        with transaction.atomic():
            created = _append_tree(lst, body.validated_data["tasks"], None, timezone.now())
        log.info("lists import %s %s", lst.pk, user.pk)
        return Response(TaskOutSerializer(created, many=True).data, status=status.HTTP_201_CREATED)

    @action(detail=True, methods=["get"])
    def export(self, request, pk=None):
        lst = owned(self.get_queryset(), _id(pk, "list"), "list")
        live = tasks_of(request.user).filter(list=lst).order_by("sort_order", "created_at", "id")
        return Response({"name": lst.name, "tasks": _export_tree(list(live))})
