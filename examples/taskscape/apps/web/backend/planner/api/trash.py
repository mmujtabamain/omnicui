"""The Trash: soft-deleted tasks (deleted_at set). A subtree is trashed as one unit, so restoring a
task brings back the descendants that share its deleted_at, and the listing shows only the
top-most trashed task of each subtree."""
import logging

from django.db import transaction
from django.db.models import Q
from rest_framework import permissions, serializers, viewsets
from rest_framework.decorators import action
from rest_framework.exceptions import NotFound
from rest_framework.response import Response

from ..models import Attachment, Task
from .common import tasks_of

log = logging.getLogger("taskscape.trash")


class TrashedTaskSerializer(serializers.ModelSerializer):
    list_name = serializers.CharField(source="list.name", read_only=True)

    class Meta:
        model = Task
        fields = ["id", "title", "list", "list_name", "deleted_at"]


class TrashIdsSerializer(serializers.Serializer):
    ids = serializers.ListField(child=serializers.IntegerField(min_value=1), allow_empty=False)


def _subtree_ids(ids):
    """The given task ids and every descendant, alive or trashed."""
    seen = set(ids)
    frontier = list(seen)
    while frontier:
        frontier = [i for i in Task.objects.filter(parent_id__in=frontier).values_list("id", flat=True)
                    if i not in seen]
        seen.update(frontier)
    return seen


def _ids(request):
    s = TrashIdsSerializer(data=request.data)
    s.is_valid(raise_exception=True)
    return list(dict.fromkeys(s.validated_data["ids"]))


def _resolve(user, ids):
    """Returns (trashed tasks in request order, stale ids). A task that exists but belongs to
    another user is 404. An id that is gone, or whose task is no longer in the Trash, is stale.
    Must run inside a transaction so the row locks hold."""
    existing = set(Task.objects.filter(id__in=ids).values_list("id", flat=True))
    owned = {t.id: t for t in
             tasks_of(user, include_trashed=True).filter(id__in=ids).select_for_update(of=("self",))}
    if existing - set(owned):
        raise NotFound("No such task.")
    stale = [i for i in ids if i not in owned or owned[i].deleted_at is None]
    roots = [owned[i] for i in ids if i in owned and owned[i].deleted_at is not None]
    return roots, stale


def _delete_tasks(ids):
    """Deletes these tasks for good; notes and attachments cascade. Stored attachment files are
    removed once the delete has committed."""
    ids = list(ids)
    if not ids:
        return
    files = [(a.file.storage, a.file.name)
             for a in Attachment.objects.filter(task_id__in=ids, kind=Attachment.Kind.FILE) if a.file]
    Task.objects.filter(id__in=ids).delete()

    def remove_files():
        for storage, name in files:
            storage.delete(name)

    transaction.on_commit(remove_files)


class TrashViewSet(viewsets.ViewSet):
    permission_classes = [permissions.IsAuthenticated]

    def list(self, request):
        qs = (
            tasks_of(request.user, include_trashed=True)
            .filter(deleted_at__isnull=False)
            .filter(Q(parent__isnull=True) | Q(parent__deleted_at__isnull=True))
            .select_related("list")
            .order_by("-deleted_at", "-id")
        )
        return Response(TrashedTaskSerializer(qs, many=True).data)

    @action(detail=False, methods=["post"])
    def restore(self, request):
        ids = _ids(request)
        with transaction.atomic():
            roots, stale = _resolve(request.user, ids)
            if stale:
                return Response({"detail": "Some tasks are no longer in the Trash.", "ids": stale},
                                status=409)
            # Each root brings back its subtree's tasks that were trashed at the same moment.
            selected = {}
            for root in roots:
                for t in Task.objects.filter(id__in=_subtree_ids([root.id]), deleted_at=root.deleted_at):
                    selected[t.id] = t
            # A task whose parent is neither being restored nor live (still in the Trash, or gone)
            # comes back at the top level of its list.
            for t in selected.values():
                if (t.parent_id is not None and t.parent_id not in selected
                        and not Task.objects.filter(pk=t.parent_id, deleted_at__isnull=True).exists()):
                    t.parent = None
                t.deleted_at = None
                t.save(update_fields=["parent", "deleted_at", "updated_at"])
        restored = sorted(selected)
        log.info("trash restore %s %s", ",".join(map(str, ids)), request.user.pk)
        return Response({"restored": restored})

    @action(detail=False, methods=["post"])
    def purge(self, request):
        ids = _ids(request)
        with transaction.atomic():
            roots, stale = _resolve(request.user, ids)
            if stale:
                return Response({"detail": "Some tasks are no longer in the Trash.", "ids": stale},
                                status=409)
            _delete_tasks(_subtree_ids([r.id for r in roots]))
        log.info("trash purge %s %s", ",".join(map(str, ids)), request.user.pk)
        return Response(status=204)

    @action(detail=False, methods=["post"])
    def empty(self, request):
        with transaction.atomic():
            trashed = (tasks_of(request.user, include_trashed=True)
                       .filter(deleted_at__isnull=False)
                       .values_list("id", flat=True))
            _delete_tasks(_subtree_ids(list(trashed)))
        log.info("trash empty - %s", request.user.pk)
        return Response(status=204)
