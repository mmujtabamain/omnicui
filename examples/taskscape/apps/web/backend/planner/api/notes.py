"""Notes: rich-text blocks on a task. Every row is reached through its owner's projects (common.py);
other users' rows answer 404. Content is sanitized to a small tag set on save."""
import logging
import re
from html import escape
from html.parser import HTMLParser
from urllib.parse import urlsplit

from rest_framework import mixins, serializers, viewsets
from rest_framework.exceptions import NotFound, ValidationError
from rest_framework.permissions import IsAuthenticated
from rest_framework.response import Response

from ..models import Note
from .common import next_order, notes_of, owned, tasks_of

log = logging.getLogger("taskscape.notes")

ALLOWED_TAGS = {"p", "br", "strong", "b", "em", "i", "u", "s", "ul", "ol", "li", "a", "code", "pre",
                "blockquote", "h1", "h2", "h3"}
VOID_TAGS = {"br"}
SAFE_SCHEMES = {"http", "https", "mailto"}
_CONTROL_CHARS = re.compile(r"[\x00-\x1f\x7f]")


def _safe_href(value):
    """The href if it is an absolute http, https or mailto URL; otherwise None."""
    if value is None:
        return None
    cleaned = _CONTROL_CHARS.sub("", value).strip(" ")
    try:
        scheme = urlsplit(cleaned).scheme.lower()
    except ValueError:
        return None
    return cleaned if scheme in SAFE_SCHEMES else None


class _Sanitizer(HTMLParser):
    """Keeps the allowed tags (re-emitted bare, except a's safe href) and escapes all text.
    Disallowed tags are dropped; their text is kept. Comments and declarations are dropped."""

    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.out = []
        self.open_tags = []

    def handle_starttag(self, tag, attrs):
        if tag not in ALLOWED_TAGS:
            return
        if tag in VOID_TAGS:
            self.out.append(f"<{tag}>")
            return
        if tag == "a":
            href = _safe_href(next((v for k, v in attrs if k == "href"), None))
            self.out.append(f'<a href="{escape(href, quote=True)}">' if href else "<a>")
        else:
            self.out.append(f"<{tag}>")
        self.open_tags.append(tag)

    def handle_endtag(self, tag):
        if tag not in ALLOWED_TAGS or tag in VOID_TAGS or tag not in self.open_tags:
            return
        while self.open_tags:
            top = self.open_tags.pop()
            self.out.append(f"</{top}>")
            if top == tag:
                break

    def handle_data(self, data):
        self.out.append(escape(data, quote=False))

    def result(self):
        while self.open_tags:
            self.out.append(f"</{self.open_tags.pop()}>")
        return "".join(self.out)


def sanitize_html(html):
    parser = _Sanitizer()
    parser.feed(html)
    parser.close()
    return parser.result()


class NoteSerializer(serializers.ModelSerializer):
    task = serializers.IntegerField(source="task_id")
    content = serializers.CharField(allow_blank=True, trim_whitespace=False)
    sort_order = serializers.IntegerField(required=False)

    class Meta:
        model = Note
        fields = ["id", "task", "content", "sort_order", "created_at", "updated_at"]
        read_only_fields = ["id", "created_at", "updated_at"]

    def get_fields(self):
        fields = super().get_fields()
        if self.instance is None:
            # create: the note goes at the end of its task and its task is named in the body
            fields["sort_order"].read_only = True
        else:
            # update: the body is {content?, sort_order?}; a note never changes task
            fields["task"].read_only = True
            fields["task"].required = False
        return fields

    def validate_task(self, value):
        owned(tasks_of(self.context["request"].user), value, "task")
        return value

    def validate_content(self, value):
        return sanitize_html(value)

    def create(self, validated_data):
        task_id = validated_data["task_id"]
        validated_data["sort_order"] = next_order(Note.objects.filter(task_id=task_id))
        return super().create(validated_data)


def _as_id(value, what):
    try:
        return int(value)
    except (TypeError, ValueError):
        raise NotFound(f"No such {what}.")


class NoteViewSet(mixins.ListModelMixin, mixins.CreateModelMixin, mixins.DestroyModelMixin,
                  viewsets.GenericViewSet):
    serializer_class = NoteSerializer
    permission_classes = [IsAuthenticated]
    pagination_class = None  # lists are returned whole

    def list(self, request, *args, **kwargs):
        task_param = request.query_params.get("task")
        if task_param is None:
            raise ValidationError({"task": ["This query parameter is required."]})
        task = owned(tasks_of(request.user), _as_id(task_param, "task"), "task")
        notes = notes_of(request.user).filter(task_id=task.id).order_by("sort_order", "created_at")
        return Response(self.get_serializer(notes, many=True).data)

    def get_object(self):
        note = owned(notes_of(self.request.user), _as_id(self.kwargs["pk"], "note"), "note")
        self.check_object_permissions(self.request, note)
        return note

    def perform_create(self, serializer):
        note = serializer.save()
        log.info("notes create %s %s", note.id, self.request.user.id)

    def partial_update(self, request, *args, **kwargs):
        note = self.get_object()
        serializer = self.get_serializer(note, data=request.data, partial=True)
        serializer.is_valid(raise_exception=True)
        note = serializer.save()
        log.info("notes update %s %s", note.id, request.user.id)
        return Response(serializer.data)

    def perform_destroy(self, instance):
        note_id = instance.id
        instance.delete()
        log.info("notes delete %s %s", note_id, self.request.user.id)
