"""Contract tests for the `attachments` resource (API.md, section "Attachments").

Written from the contract alone. Starting rows are made through the ORM so that each test
exercises only the endpoint it names. Stored files go to a throwaway MEDIA_ROOT.
"""
import datetime as dt
import logging
import os
import re
from urllib.parse import urlsplit

import pytest
from django.conf import settings as django_settings
from django.core.files.uploadedfile import SimpleUploadedFile
from django.urls import include, path, re_path
from django.utils import timezone
from django.views.static import serve
from rest_framework.test import APIClient

from planner.models import Attachment, Project, Task, TaskList

ATTACHMENTS = "/api/attachments"
MB = 1024 * 1024
FIELDS = {"id", "task", "kind", "name", "url", "size", "mime", "created_at"}
PDF = b"%PDF-1.4 test document"

# Routing for the download test. The project mounts /media/ only under DEBUG, and pytest-django runs
# with DEBUG off, so media_routes() points ROOT_URLCONF at this module. It serves MEDIA_URL from the
# throwaway MEDIA_ROOT and hands every other path to the project's own urlconf.
PROJECT_URLCONF = django_settings.ROOT_URLCONF
MEDIA_PREFIX = urlsplit(django_settings.MEDIA_URL).path.lstrip("/")


def _serve_media(request, name):
    return serve(request, name, document_root=django_settings.MEDIA_ROOT)


urlpatterns = [
    re_path(rf"^{re.escape(MEDIA_PREFIX)}(?P<name>.*)$", _serve_media),
    path("", include(PROJECT_URLCONF)),
]


@pytest.fixture(autouse=True)
def media_root(settings, tmp_path):
    """Uploaded files are written under a temporary directory, not the real MEDIA_ROOT."""
    settings.MEDIA_ROOT = str(tmp_path / "media")
    return settings.MEDIA_ROOT


def make_task(task_list, title="Buy milk", parent=None):
    return Task.objects.create(list=task_list, title=title, parent=parent)


def make_file_attachment(task, name="report.pdf", content=PDF, mime="application/pdf"):
    return Attachment.objects.create(
        task=task,
        kind=Attachment.Kind.FILE,
        name=name,
        file=SimpleUploadedFile(name, content, content_type=mime),
        size=len(content),
        mime=mime,
    )


def make_link_attachment(task, url="https://example.com/plan", name=None):
    return Attachment.objects.create(task=task, kind=Attachment.Kind.LINK, name=name or url, url=url)


def upload(client, task, content=PDF, content_type="application/pdf", filename="report.pdf", **extra):
    """POST multipart {task, file, name?}."""
    data = {"task": task.pk, "file": SimpleUploadedFile(filename, content, content_type=content_type), **extra}
    return client.post(ATTACHMENTS, data, format="multipart")


def link(client, task, url="https://example.com/plan", **extra):
    """POST JSON {task, url, name?}."""
    return client.post(ATTACHMENTS, {"task": task.pk, "url": url, **extra}, format="json")


def info_lines(caplog):
    return [
        r.getMessage()
        for r in caplog.records
        if r.levelno == logging.INFO and r.name == "taskscape.attachments"
    ]


@pytest.fixture
def media_routes(settings):
    """Serve MEDIA_URL from MEDIA_ROOT for the test that downloads a stored file (see urlpatterns above)."""
    settings.ROOT_URLCONF = __name__


@pytest.fixture
def milk(home):
    """A live task in Ana's Errands list."""
    return make_task(home["errands"], "Buy milk")


@pytest.fixture
def bob_task(bob):
    """A live task in Bob's own project."""
    project = Project.objects.create(owner=bob, name="Bob's")
    return make_task(TaskList.objects.create(project=project, name="Chores"), "Mow lawn")


@pytest.fixture
def ana_file(milk):
    return make_file_attachment(milk)


@pytest.fixture
def ana_link(milk):
    return make_link_attachment(milk)


# Authentication

def test_anonymous_request_to_list_attachments_is_refused(milk):
    resp = APIClient().get(ATTACHMENTS, {"task": milk.pk})
    assert resp.status_code in (401, 403)


def test_anonymous_upload_is_refused(milk):
    resp = upload(APIClient(), milk)
    assert resp.status_code in (401, 403)
    assert not Attachment.objects.exists()


# POST multipart: file upload

def test_upload_answers_201_with_exactly_the_attachment_fields(client_for, ana, milk):
    resp = upload(client_for(ana), milk)
    assert resp.status_code == 201
    assert set(resp.json()) == FIELDS


def test_upload_is_kind_file(client_for, ana, milk):
    resp = upload(client_for(ana), milk)
    assert resp.json()["kind"] == "file"


def test_upload_records_the_task_it_was_sent_to(client_for, ana, milk):
    resp = upload(client_for(ana), milk)
    assert resp.json()["task"] == milk.pk


def test_upload_name_defaults_to_the_file_name(client_for, ana, milk):
    resp = upload(client_for(ana), milk, filename="report.pdf")
    assert resp.json()["name"] == "report.pdf"


def test_upload_explicit_name_overrides_the_file_name(client_for, ana, milk):
    resp = upload(client_for(ana), milk, filename="report.pdf", name="Quarterly report")
    assert resp.status_code == 201
    assert resp.json()["name"] == "Quarterly report"


def test_upload_size_is_the_byte_count_of_the_file(client_for, ana, milk):
    resp = upload(client_for(ana), milk, content=PDF)
    assert resp.json()["size"] == len(PDF)


def test_upload_mime_is_taken_from_the_upload(client_for, ana, milk):
    resp = upload(client_for(ana), milk, content=b"\x89PNG\r\n\x1a\n", content_type="image/png", filename="x.png")
    assert resp.json()["mime"] == "image/png"


def test_upload_url_points_under_media(client_for, ana, milk):
    resp = upload(client_for(ana), milk)
    assert "/media/" in resp.json()["url"]


def test_uploaded_file_downloads_its_content_from_the_url(client_for, ana, milk, media_routes):
    body = upload(client_for(ana), milk).json()
    download = client_for(ana).get(body["url"])
    assert download.status_code == 200
    data = b"".join(download.streaming_content) if download.streaming else download.content
    assert data == PDF


def test_upload_ids_are_integers(client_for, ana, milk):
    body = upload(client_for(ana), milk).json()
    assert isinstance(body["id"], int)
    assert isinstance(body["task"], int)
    assert isinstance(body["size"], int)


def test_upload_created_at_is_iso_8601_utc(client_for, ana, milk):
    raw = upload(client_for(ana), milk).json()["created_at"]
    parsed = dt.datetime.fromisoformat(raw.replace("Z", "+00:00"))
    assert parsed.tzinfo is not None
    assert parsed.utcoffset() == dt.timedelta(0)


def test_upload_of_25_mb_is_accepted(client_for, ana, milk):
    resp = upload(client_for(ana), milk, content=b"0" * 25_000_000)
    assert resp.status_code == 201


def test_upload_larger_than_25_mb_is_400_and_stores_nothing(client_for, ana, milk):
    resp = upload(client_for(ana), milk, content=b"0" * (25 * MB + 1))
    assert resp.status_code == 400
    assert "file" in resp.json()
    assert not Attachment.objects.exists()


def test_upload_without_a_file_is_400(client_for, ana, milk):
    resp = client_for(ana).post(ATTACHMENTS, {"task": milk.pk}, format="multipart")
    assert resp.status_code == 400
    assert "file" in resp.json()


def test_upload_without_a_task_is_400(client_for, ana):
    resp = client_for(ana).post(
        ATTACHMENTS, {"file": SimpleUploadedFile("a.txt", PDF, content_type="text/plain")}, format="multipart"
    )
    assert resp.status_code == 400
    assert "task" in resp.json()


# POST JSON: link

def test_link_answers_201_with_exactly_the_attachment_fields(client_for, ana, milk):
    resp = link(client_for(ana), milk)
    assert resp.status_code == 201
    assert set(resp.json()) == FIELDS


def test_link_is_kind_link(client_for, ana, milk):
    resp = link(client_for(ana), milk)
    assert resp.json()["kind"] == "link"


def test_link_stores_the_url_as_given(client_for, ana, milk):
    resp = link(client_for(ana), milk, url="https://example.com/docs/plan?x=1")
    assert resp.json()["url"] == "https://example.com/docs/plan?x=1"


def test_link_name_defaults_to_the_url(client_for, ana, milk):
    url = "https://example.com/docs/plan"
    resp = link(client_for(ana), milk, url=url)
    assert resp.json()["name"] == url


def test_link_explicit_name_overrides_the_url(client_for, ana, milk):
    resp = link(client_for(ana), milk, url="https://example.com/plan", name="The plan")
    assert resp.status_code == 201
    assert resp.json()["name"] == "The plan"


def test_http_link_is_accepted(client_for, ana, milk):
    resp = link(client_for(ana), milk, url="http://example.com/plan")
    assert resp.status_code == 201


def test_https_link_is_accepted(client_for, ana, milk):
    resp = link(client_for(ana), milk, url="https://example.com/plan")
    assert resp.status_code == 201


@pytest.mark.parametrize(
    "url",
    [
        pytest.param("ftp://example.com/file.zip", id="ftp"),
        pytest.param("javascript:alert(1)", id="javascript"),
        pytest.param("file:///etc/passwd", id="file"),
        pytest.param("mailto:ana@example.com", id="mailto"),
        pytest.param("example.com/no-scheme", id="no-scheme"),
    ],
)
def test_link_with_a_scheme_other_than_http_or_https_is_400(client_for, ana, milk, url):
    resp = link(client_for(ana), milk, url=url)
    assert resp.status_code == 400
    assert "url" in resp.json()
    assert not Attachment.objects.exists()


def test_link_without_a_url_is_400(client_for, ana, milk):
    resp = client_for(ana).post(ATTACHMENTS, {"task": milk.pk}, format="json")
    assert resp.status_code == 400
    assert "url" in resp.json()


# Creating on another user's task

def test_bob_cannot_upload_to_anas_task_and_gets_404(client_for, bob, milk):
    resp = upload(client_for(bob), milk)
    assert resp.status_code == 404
    assert not Attachment.objects.exists()


def test_bob_cannot_add_a_link_to_anas_task_and_gets_404(client_for, bob, milk):
    resp = link(client_for(bob), milk)
    assert resp.status_code == 404
    assert not Attachment.objects.exists()


def test_bob_can_add_a_link_to_his_own_task(client_for, bob, bob_task):
    resp = link(client_for(bob), bob_task)
    assert resp.status_code == 201
    assert resp.json()["task"] == bob_task.pk


# GET attachments?task={id}

def test_list_answers_200_with_a_plain_array_not_a_paginated_object(client_for, ana, milk, ana_file):
    resp = client_for(ana).get(ATTACHMENTS, {"task": milk.pk})
    assert resp.status_code == 200
    assert isinstance(resp.json(), list)
    assert len(resp.json()) == 1


def test_list_items_have_exactly_the_attachment_fields(client_for, ana, milk, ana_file, ana_link):
    items = client_for(ana).get(ATTACHMENTS, {"task": milk.pk}).json()
    assert len(items) == 2
    for item in items:
        assert set(item) == FIELDS


def test_list_returns_only_the_attachments_of_the_given_task(client_for, ana, home, milk, ana_file):
    other = make_task(home["reading"], "Read book")
    make_link_attachment(other, url="https://example.com/other")
    items = client_for(ana).get(ATTACHMENTS, {"task": milk.pk}).json()
    assert [item["id"] for item in items] == [ana_file.pk]


def test_list_is_empty_for_a_task_without_attachments(client_for, ana, milk):
    resp = client_for(ana).get(ATTACHMENTS, {"task": milk.pk})
    assert resp.status_code == 200
    assert resp.json() == []


def test_list_is_oldest_first_by_created_at_not_by_id(client_for, ana, milk):
    first = make_link_attachment(milk, url="https://example.com/1")
    second = make_link_attachment(milk, url="https://example.com/2")
    now = timezone.now()
    # The row with the higher id is the older one, so id order and created_at order differ.
    Attachment.objects.filter(pk=first.pk).update(created_at=now)
    Attachment.objects.filter(pk=second.pk).update(created_at=now - dt.timedelta(hours=1))
    items = client_for(ana).get(ATTACHMENTS, {"task": milk.pk}).json()
    assert [item["id"] for item in items] == [second.pk, first.pk]


def test_bob_gets_404_listing_attachments_of_anas_task(client_for, bob, milk, ana_file):
    resp = client_for(bob).get(ATTACHMENTS, {"task": milk.pk})
    assert resp.status_code == 404


# PATCH attachments/{id} {name}

def test_patch_name_answers_200_with_the_new_name(client_for, ana, ana_file):
    resp = client_for(ana).patch(f"{ATTACHMENTS}/{ana_file.pk}", {"name": "Renamed"}, format="json")
    assert resp.status_code == 200
    body = resp.json()
    assert body["id"] == ana_file.pk
    assert body["name"] == "Renamed"


def test_patch_name_is_saved(client_for, ana, ana_file):
    client_for(ana).patch(f"{ATTACHMENTS}/{ana_file.pk}", {"name": "Renamed"}, format="json")
    ana_file.refresh_from_db()
    assert ana_file.name == "Renamed"


def test_patch_changes_only_the_name(client_for, ana, milk, ana_file):
    client = client_for(ana)
    before = client.get(ATTACHMENTS, {"task": milk.pk}).json()[0]
    after = client.patch(f"{ATTACHMENTS}/{ana_file.pk}", {"name": "Renamed"}, format="json").json()
    for field in ("id", "task", "kind", "url", "size", "mime", "created_at"):
        assert after[field] == before[field], field


def test_patch_with_an_empty_name_is_400(client_for, ana, ana_file):
    resp = client_for(ana).patch(f"{ATTACHMENTS}/{ana_file.pk}", {"name": ""}, format="json")
    assert resp.status_code == 400
    assert "name" in resp.json()
    ana_file.refresh_from_db()
    assert ana_file.name == "report.pdf"


def test_patch_with_a_name_longer_than_255_characters_is_400(client_for, ana, ana_file):
    resp = client_for(ana).patch(f"{ATTACHMENTS}/{ana_file.pk}", {"name": "x" * 256}, format="json")
    assert resp.status_code == 400
    assert "name" in resp.json()


def test_bob_cannot_rename_anas_attachment_and_gets_404(client_for, bob, ana_file):
    resp = client_for(bob).patch(f"{ATTACHMENTS}/{ana_file.pk}", {"name": "Stolen"}, format="json")
    assert resp.status_code == 404
    ana_file.refresh_from_db()
    assert ana_file.name == "report.pdf"


# DELETE attachments/{id}

def test_delete_answers_204(client_for, ana, ana_file):
    resp = client_for(ana).delete(f"{ATTACHMENTS}/{ana_file.pk}")
    assert resp.status_code == 204


def test_delete_removes_the_attachment(client_for, ana, milk, ana_file):
    client = client_for(ana)
    client.delete(f"{ATTACHMENTS}/{ana_file.pk}")
    assert not Attachment.objects.filter(pk=ana_file.pk).exists()
    assert client.get(ATTACHMENTS, {"task": milk.pk}).json() == []


def test_delete_removes_the_stored_file(client_for, ana, ana_file):
    path = ana_file.file.path
    assert os.path.exists(path)
    client_for(ana).delete(f"{ATTACHMENTS}/{ana_file.pk}")
    assert not os.path.exists(path)


def test_delete_of_a_link_answers_204_and_removes_it(client_for, ana, ana_link):
    resp = client_for(ana).delete(f"{ATTACHMENTS}/{ana_link.pk}")
    assert resp.status_code == 204
    assert not Attachment.objects.filter(pk=ana_link.pk).exists()


def test_bob_cannot_delete_anas_attachment_and_gets_404(client_for, bob, ana_file):
    path = ana_file.file.path
    resp = client_for(bob).delete(f"{ATTACHMENTS}/{ana_file.pk}")
    assert resp.status_code == 404
    assert Attachment.objects.filter(pk=ana_file.pk).exists()
    assert os.path.exists(path)


# Logging: every write logs one INFO line (resource action id user)

def test_upload_writes_one_info_line_naming_the_resource_id_and_user(client_for, ana, milk, caplog):
    caplog.set_level(logging.INFO)
    resp = link(client_for(ana), milk)
    assert resp.status_code == 201
    lines = info_lines(caplog)
    assert len(lines) == 1
    assert re.search(rf"\b{resp.json()['id']}\b", lines[0])
    assert re.search(rf"\b{ana.pk}\b", lines[0])


def test_rename_writes_one_info_line(client_for, ana, ana_file, caplog):
    caplog.set_level(logging.INFO)
    resp = client_for(ana).patch(f"{ATTACHMENTS}/{ana_file.pk}", {"name": "Renamed"}, format="json")
    assert resp.status_code == 200
    assert len(info_lines(caplog)) == 1


def test_delete_writes_one_info_line(client_for, ana, ana_file, caplog):
    caplog.set_level(logging.INFO)
    resp = client_for(ana).delete(f"{ATTACHMENTS}/{ana_file.pk}")
    assert resp.status_code == 204
    assert len(info_lines(caplog)) == 1


# Cascades from other resources

def test_deleting_a_project_removes_its_attachments_and_their_files(client_for, ana, home, milk, ana_file):
    work = Project.objects.create(owner=ana, name="Work")
    work_task = make_task(TaskList.objects.create(project=work, name="Inbox"), "Draft")
    kept = make_file_attachment(work_task, name="draft.pdf")
    kept_path = kept.file.path
    gone_path = ana_file.file.path

    resp = client_for(ana).delete(f"/api/projects/{home['project'].pk}")

    assert resp.status_code == 204
    assert not Attachment.objects.filter(pk=ana_file.pk).exists()
    assert not os.path.exists(gone_path)
    assert Attachment.objects.filter(pk=kept.pk).exists()
    assert os.path.exists(kept_path)


def test_deleting_a_list_removes_the_attachments_of_its_tasks(client_for, ana, home, milk, ana_file):
    reading_task = make_task(home["reading"], "Read book")
    kept = make_link_attachment(reading_task, url="https://example.com/book")

    resp = client_for(ana).delete(f"/api/lists/{home['errands'].pk}")

    assert resp.status_code == 204
    assert not Attachment.objects.filter(pk=ana_file.pk).exists()
    assert Attachment.objects.filter(pk=kept.pk).exists()


def test_purging_a_trashed_task_removes_its_attachments_and_its_subtree(client_for, ana, home):
    parent = make_task(home["errands"], "Parent")
    child = make_task(home["errands"], "Child", parent=parent)
    parent_att = make_link_attachment(parent, url="https://example.com/parent")
    child_att = make_file_attachment(child, name="child.pdf")
    Task.objects.filter(pk__in=[parent.pk, child.pk]).update(deleted_at=timezone.now())

    resp = client_for(ana).post("/api/trash/purge", {"ids": [parent.pk]}, format="json")

    assert resp.status_code == 204
    assert not Attachment.objects.filter(pk=parent_att.pk).exists()
    assert not Attachment.objects.filter(pk=child_att.pk).exists()


def test_emptying_the_trash_removes_attachments_of_trashed_tasks_only(client_for, ana, home, milk):
    trashed = make_task(home["errands"], "Old")
    trashed_att = make_link_attachment(trashed, url="https://example.com/old")
    live_att = make_link_attachment(milk, url="https://example.com/live")
    Task.objects.filter(pk=trashed.pk).update(deleted_at=timezone.now())

    resp = client_for(ana).post("/api/trash/empty")

    assert resp.status_code == 204
    assert not Attachment.objects.filter(pk=trashed_att.pk).exists()
    assert Attachment.objects.filter(pk=live_att.pk).exists()
