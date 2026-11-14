"""The projects resource (API.md, "Projects: `projects`"): list, create, rename or reorder, delete.

A project is the owner's; anything of another user's answers 404 (see common.owned)."""
import logging

from django.contrib.auth import get_user_model
from django.db import transaction
from rest_framework import serializers, viewsets
from rest_framework.exceptions import NotFound
from rest_framework.permissions import IsAuthenticated
from rest_framework.response import Response

from ..models import Attachment, Project, TaskList
from .common import next_order, owned, projects_of

log = logging.getLogger("taskscape.projects")


class ProjectSerializer(serializers.ModelSerializer):
    # CharField trims surrounding whitespace, so a blank or whitespace-only name is a 400.
    name = serializers.CharField(max_length=200)

    class Meta:
        model = Project
        fields = ["id", "name", "sort_order", "created_at", "updated_at"]
        read_only_fields = ["id", "created_at", "updated_at"]


class ProjectCreateSerializer(serializers.Serializer):
    name = serializers.CharField(max_length=200)


def _owned_project(request, pk):
    """The user's project with this id; a non-integer id is simply not found."""
    try:
        pk = int(pk)
    except (TypeError, ValueError):
        raise NotFound("No such project.")
    return owned(projects_of(request.user), pk, "project")


class ProjectViewSet(viewsets.GenericViewSet):
    permission_classes = [IsAuthenticated]
    serializer_class = ProjectSerializer

    def get_serializer_class(self):
        return ProjectCreateSerializer if self.action == "create" else ProjectSerializer

    def list(self, request):
        projects = projects_of(request.user).order_by("sort_order", "created_at")
        return Response(ProjectSerializer(projects, many=True).data)

    def create(self, request):
        ser = ProjectCreateSerializer(data=request.data)
        ser.is_valid(raise_exception=True)
        user = request.user
        with transaction.atomic():
            project = Project.objects.create(
                owner=user,
                name=ser.validated_data["name"],
                sort_order=next_order(projects_of(user)),
            )
            TaskList.objects.create(project=project, name="Inbox", sort_order=0)
        log.info("projects create %s %s", project.pk, user.pk)
        return Response(ProjectSerializer(project).data, status=201)

    def partial_update(self, request, pk=None):
        project = _owned_project(request, pk)
        ser = ProjectSerializer(project, data=request.data, partial=True)
        ser.is_valid(raise_exception=True)
        ser.save()
        log.info("projects update %s %s", project.pk, request.user.pk)
        return Response(ser.data)

    def destroy(self, request, pk=None):
        user = request.user
        project = _owned_project(request, pk)
        project_id = project.pk
        with transaction.atomic():
            # Serialise deletes per user so two of them cannot both leave the user with no project.
            get_user_model().objects.select_for_update().get(pk=user.pk)
            if projects_of(user).count() <= 1:
                return Response({"detail": "A project is needed."}, status=400)
            files = list(
                Attachment.objects.filter(task__list__project=project)
                .exclude(file="")
                .values_list("file", flat=True)
            )
            project.delete()
        storage = Attachment._meta.get_field("file").storage
        for name in files:
            storage.delete(name)
        log.info("projects delete %s %s", project_id, user.pk)
        return Response(status=204)
