"""The attachments resource: a file uploaded to MEDIA_ROOT, or a link, pinned to one of the user's tasks.

Everything is reached through tasks_of/attachments_of, so another user's task or attachment answers 404.
"""
import logging
from urllib.parse import urlsplit

from rest_framework import serializers, status, viewsets
from rest_framework.exceptions import NotFound, ValidationError
from rest_framework.parsers import FormParser, JSONParser, MultiPartParser
from rest_framework.permissions import IsAuthenticated
from rest_framework.response import Response

from ..models import Attachment
from .common import attachments_of, owned, tasks_of

log = logging.getLogger("taskscape.attachments")

MAX_UPLOAD_BYTES = 25 * 1024 * 1024
MAX_ID = 2**31 - 1  # ids past this cannot exist; treat them as not found instead of overflowing the database


def _owned_or_404(qs, raw, what):
    """owned() for an id from a URL or query string; something that is not an id is simply not found."""
    try:
        pk = int(raw)
    except (TypeError, ValueError):
        pk = None
    if pk is None or not 0 < pk <= MAX_ID:
        raise NotFound(f"No such {what}.")
    return owned(qs, pk, what)


def _log_write(action, attachment_id, user):
    log.info("attachment %s %s %s", action, attachment_id, user.pk)


class AttachmentSerializer(serializers.ModelSerializer):
    url = serializers.SerializerMethodField()

    class Meta:
        model = Attachment
        fields = ["id", "task", "kind", "name", "url", "size", "mime", "created_at"]
        read_only_fields = ["id", "task", "kind", "name", "size", "mime", "created_at"]

    def get_url(self, obj):
        if obj.kind == Attachment.Kind.FILE:
            return obj.file.url  # under /media/
        return obj.url


class AttachmentRenameSerializer(serializers.ModelSerializer):
    class Meta:
        model = Attachment
        fields = ["name"]


class AttachmentCreateSerializer(serializers.Serializer):
    """Either a multipart `file` or a JSON/form `url`, on a live task of the user's."""

    task = serializers.IntegerField()
    file = serializers.FileField(required=False)
    url = serializers.URLField(max_length=2000, required=False)
    name = serializers.CharField(max_length=255, required=False)

    def validate_task(self, value):
        return _owned_or_404(tasks_of(self.context["request"].user), value, "task")

    def validate_file(self, value):
        if value.size > MAX_UPLOAD_BYTES:
            raise serializers.ValidationError("Files are limited to 25 MB.")
        return value

    def validate_url(self, value):
        if urlsplit(value).scheme.lower() not in ("http", "https"):
            raise serializers.ValidationError("Only http and https links are allowed.")
        return value

    def validate(self, attrs):
        has_file = "file" in attrs
        has_url = "url" in attrs
        if has_file and has_url:
            message = ["Send a file or a url, not both."]
            raise serializers.ValidationError({"file": message, "url": message})
        if not (has_file or has_url):
            message = ["Send a file or a url."]
            raise serializers.ValidationError({"file": message, "url": message})
        return attrs

    def create(self, validated_data):
        task = validated_data["task"]
        name = validated_data.get("name")
        upload = validated_data.get("file")
        if upload is not None:
            return Attachment.objects.create(
                task=task,
                kind=Attachment.Kind.FILE,
                file=upload,
                name=((name or upload.name) or "file")[:255],
                size=upload.size,
                mime=(upload.content_type or "")[:120],
            )
        link = validated_data["url"]
        return Attachment.objects.create(
            task=task,
            kind=Attachment.Kind.LINK,
            url=link,
            name=(name or link)[:255],
        )


class AttachmentViewSet(viewsets.GenericViewSet):
    """GET attachments?task={id}, POST attachments, PATCH attachments/{id}, DELETE attachments/{id}.

    Lists are returned whole (no pagination)."""

    permission_classes = [IsAuthenticated]
    pagination_class = None
    parser_classes = [JSONParser, FormParser, MultiPartParser]
    lookup_value_regex = r"\d+"

    def list(self, request):
        raw = request.query_params.get("task")
        if raw is None:
            raise ValidationError({"task": ["This query parameter is required."]})
        task = _owned_or_404(tasks_of(request.user), raw, "task")
        qs = attachments_of(request.user).filter(task=task).order_by("created_at", "id")
        return Response(AttachmentSerializer(qs, many=True).data)

    def create(self, request):
        serializer = AttachmentCreateSerializer(data=request.data, context={"request": request})
        serializer.is_valid(raise_exception=True)
        attachment = serializer.save()
        _log_write("create", attachment.pk, request.user)
        return Response(AttachmentSerializer(attachment).data, status=status.HTTP_201_CREATED)

    def partial_update(self, request, pk=None):
        attachment = _owned_or_404(attachments_of(request.user), pk, "attachment")
        serializer = AttachmentRenameSerializer(attachment, data=request.data, partial=True)
        serializer.is_valid(raise_exception=True)
        serializer.save()
        _log_write("update", attachment.pk, request.user)
        return Response(AttachmentSerializer(attachment).data)

    def destroy(self, request, pk=None):
        attachment = _owned_or_404(attachments_of(request.user), pk, "attachment")
        attachment_id = attachment.pk
        storage = attachment.file.storage
        stored = attachment.file.name if attachment.kind == Attachment.Kind.FILE and attachment.file else ""
        attachment.delete()
        if stored:
            storage.delete(stored)
        _log_write("delete", attachment_id, request.user)
        return Response(status=status.HTTP_204_NO_CONTENT)
