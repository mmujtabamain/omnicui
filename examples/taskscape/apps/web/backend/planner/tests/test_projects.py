"""Contract tests for the Projects section of API.md (`projects`).

Written from the contract alone. Every rule of the section gets its own test; the
shared rules that apply to every write (sign-in, CSRF, logging) are covered too.
"""
import logging
import os
from datetime import datetime, timedelta

import pytest
from django.core.files.base import ContentFile
from django.utils import timezone
from rest_framework.test import APIClient

from planner.models import Attachment, Note, Project, Task, TaskList

PROJECTS = "/api/projects"
PROJECT_FIELDS = {"id", "name", "sort_order", "created_at", "updated_at"}


def url(pk):
    return f"{PROJECTS}/{pk}"


def assert_iso_utc(value):
    assert isinstance(value, str), f"expected an ISO 8601 string, got {value!r}"
    parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    assert parsed.utcoffset() == timedelta(0), f"{value} is not a UTC time"


def make_project(owner, name, sort_order=0, created_at=None):
    project = Project.objects.create(owner=owner, name=name, sort_order=sort_order)
    if created_at is not None:
        # created_at is auto_now_add, so it is set directly to control ties and ordering.
        Project.objects.filter(pk=project.pk).update(created_at=created_at)
    return project


def names(response):
    return [item["name"] for item in response.json()]


# --- sign-in -----------------------------------------------------------------------


def test_every_projects_endpoint_needs_a_signed_in_user(ana):
    project = make_project(ana, "Home")
    anonymous = APIClient()
    responses = [
        anonymous.get(PROJECTS),
        anonymous.post(PROJECTS, {"name": "Work"}, format="json"),
        anonymous.patch(url(project.pk), {"name": "Renamed"}, format="json"),
        anonymous.delete(url(project.pk)),
    ]
    for response in responses:
        assert response.status_code in (401, 403)
    project.refresh_from_db()
    assert project.name == "Home"
    assert Project.objects.filter(owner=ana).count() == 1


# --- GET projects ------------------------------------------------------------------


def test_list_answers_200_with_a_json_list_of_the_users_projects(client_for, ana):
    make_project(ana, "Home")
    make_project(ana, "Work", sort_order=1)
    response = client_for(ana).get(PROJECTS)
    assert response.status_code == 200
    body = response.json()
    assert isinstance(body, list)
    assert len(body) == 2


def test_list_items_have_exactly_the_project_fields(client_for, ana):
    project = make_project(ana, "Home")
    [item] = client_for(ana).get(PROJECTS).json()
    assert set(item) == PROJECT_FIELDS
    assert isinstance(item["id"], int)
    assert item["id"] == project.pk
    assert item["name"] == "Home"
    assert item["sort_order"] == 0
    assert_iso_utc(item["created_at"])
    assert_iso_utc(item["updated_at"])


def test_list_orders_projects_by_sort_order_regardless_of_creation_order(client_for, ana):
    make_project(ana, "Third", sort_order=2)
    make_project(ana, "First", sort_order=0)
    make_project(ana, "Second", sort_order=1)
    assert names(client_for(ana).get(PROJECTS)) == ["First", "Second", "Third"]


def test_list_breaks_sort_order_ties_by_created_at(client_for, ana):
    now = timezone.now()
    # Created in this order, so the id order would put "Newer" first.
    make_project(ana, "Newer", sort_order=0, created_at=now)
    make_project(ana, "Older", sort_order=0, created_at=now - timedelta(days=1))
    assert names(client_for(ana).get(PROJECTS)) == ["Older", "Newer"]


def test_list_returns_every_project_without_pagination(client_for, ana):
    for i in range(30):
        make_project(ana, f"Project {i}", sort_order=i)
    assert len(client_for(ana).get(PROJECTS).json()) == 30


def test_list_only_shows_the_users_own_projects(client_for, ana, bob):
    make_project(ana, "Home")
    make_project(bob, "Bob's")
    assert names(client_for(ana).get(PROJECTS)) == ["Home"]
    assert names(client_for(bob).get(PROJECTS)) == ["Bob's"]


# --- POST projects -----------------------------------------------------------------


def test_create_answers_201_with_the_new_project(client_for, ana):
    response = client_for(ana).post(PROJECTS, {"name": "Work"}, format="json")
    assert response.status_code == 201
    body = response.json()
    assert set(body) == PROJECT_FIELDS
    assert isinstance(body["id"], int)
    assert body["name"] == "Work"
    assert_iso_utc(body["created_at"])
    assert_iso_utc(body["updated_at"])


def test_create_makes_the_project_owned_by_the_signed_in_user(client_for, ana, bob):
    body = client_for(ana).post(PROJECTS, {"name": "Work"}, format="json").json()
    assert Project.objects.get(pk=body["id"]).owner_id == ana.pk
    assert Project.objects.get(pk=body["id"]).owner_id != bob.pk


def test_create_trims_surrounding_whitespace_from_the_name(client_for, ana):
    response = client_for(ana).post(PROJECTS, {"name": "  Work  "}, format="json")
    assert response.status_code == 201
    assert response.json()["name"] == "Work"
    assert Project.objects.get(pk=response.json()["id"]).name == "Work"


@pytest.mark.parametrize("name", ["", "   ", "\t\n "])
def test_create_with_a_name_empty_after_trimming_is_400(client_for, ana, name):
    response = client_for(ana).post(PROJECTS, {"name": name}, format="json")
    assert response.status_code == 400
    assert "name" in response.json()
    assert not Project.objects.filter(owner=ana).exists()


def test_create_without_a_name_is_400(client_for, ana):
    response = client_for(ana).post(PROJECTS, {}, format="json")
    assert response.status_code == 400
    assert "name" in response.json()
    assert not Project.objects.filter(owner=ana).exists()


def test_create_places_the_new_project_at_the_end_with_last_sort_order_plus_one(client_for, ana):
    make_project(ana, "A", sort_order=0)
    make_project(ana, "B", sort_order=1)
    make_project(ana, "C", sort_order=5)
    response = client_for(ana).post(PROJECTS, {"name": "D"}, format="json")
    assert response.json()["sort_order"] == 6
    assert names(client_for(ana).get(PROJECTS)) == ["A", "B", "C", "D"]


def test_create_computes_the_position_from_the_users_own_projects_only(client_for, ana, bob):
    make_project(ana, "A", sort_order=0)
    make_project(bob, "Bob's", sort_order=9)
    response = client_for(ana).post(PROJECTS, {"name": "B"}, format="json")
    assert response.json()["sort_order"] == 1


def test_create_adds_one_list_named_inbox_to_the_new_project(client_for, ana):
    body = client_for(ana).post(PROJECTS, {"name": "Work"}, format="json").json()
    lists = list(TaskList.objects.filter(project_id=body["id"]))
    assert [task_list.name for task_list in lists] == ["Inbox"]


# --- PATCH projects/{id} -----------------------------------------------------------


def test_rename_answers_200_with_the_project_and_saves_the_new_name(client_for, ana):
    project = make_project(ana, "Home")
    response = client_for(ana).patch(url(project.pk), {"name": "Household"}, format="json")
    assert response.status_code == 200
    body = response.json()
    assert set(body) == PROJECT_FIELDS
    assert body["id"] == project.pk
    assert body["name"] == "Household"
    project.refresh_from_db()
    assert project.name == "Household"


def test_rename_keeps_the_sort_order(client_for, ana):
    project = make_project(ana, "Home", sort_order=3)
    response = client_for(ana).patch(url(project.pk), {"name": "Household"}, format="json")
    assert response.json()["sort_order"] == 3


def test_rename_to_a_name_empty_after_trimming_is_400(client_for, ana):
    project = make_project(ana, "Home")
    response = client_for(ana).patch(url(project.pk), {"name": "   "}, format="json")
    assert response.status_code == 400
    assert "name" in response.json()
    project.refresh_from_db()
    assert project.name == "Home"


def test_patching_a_non_integer_sort_order_is_400(client_for, ana):
    project = make_project(ana, "Home", sort_order=2)
    response = client_for(ana).patch(url(project.pk), {"sort_order": "soon"}, format="json")
    assert response.status_code == 400
    assert "sort_order" in response.json()
    project.refresh_from_db()
    assert project.sort_order == 2


def test_reorder_changes_where_the_project_appears_in_the_list(client_for, ana):
    make_project(ana, "A", sort_order=1)
    make_project(ana, "B", sort_order=2)
    c = make_project(ana, "C", sort_order=3)
    response = client_for(ana).patch(url(c.pk), {"sort_order": 0}, format="json")
    assert response.status_code == 200
    assert response.json()["sort_order"] == 0
    assert names(client_for(ana).get(PROJECTS)) == ["C", "A", "B"]


def test_rename_and_reorder_can_happen_in_one_call(client_for, ana):
    project = make_project(ana, "Home", sort_order=4)
    response = client_for(ana).patch(
        url(project.pk), {"name": "Household", "sort_order": 0}, format="json"
    )
    assert response.status_code == 200
    body = response.json()
    assert body["name"] == "Household"
    assert body["sort_order"] == 0


@pytest.mark.parametrize("payload", [{"name": "Hijacked"}, {"sort_order": 9}])
def test_bob_gets_404_patching_each_of_anas_projects(client_for, ana, bob, payload):
    projects = [make_project(ana, "Home", sort_order=0), make_project(ana, "Work", sort_order=1)]
    for project in projects:
        response = client_for(bob).patch(url(project.pk), payload, format="json")
        assert response.status_code == 404
    for project in projects:
        project.refresh_from_db()
    assert [project.name for project in projects] == ["Home", "Work"]
    assert [project.sort_order for project in projects] == [0, 1]


# --- DELETE projects/{id} ----------------------------------------------------------


def test_delete_answers_204_and_removes_the_project(client_for, ana):
    make_project(ana, "Home")
    other = make_project(ana, "Work", sort_order=1)
    response = client_for(ana).delete(url(other.pk))
    assert response.status_code == 204
    assert response.content == b""
    assert not Project.objects.filter(pk=other.pk).exists()
    assert Project.objects.filter(owner=ana).count() == 1


def test_delete_removes_the_projects_lists(client_for, ana, home):
    make_project(ana, "Work", sort_order=1)
    response = client_for(ana).delete(url(home["project"].pk))
    assert response.status_code == 204
    assert not TaskList.objects.filter(
        pk__in=[home["errands"].pk, home["reading"].pk]
    ).exists()


def test_delete_removes_the_projects_live_tasks_and_their_subtrees(client_for, ana, home):
    make_project(ana, "Work", sort_order=1)
    parent = Task.objects.create(list=home["errands"], title="Parent")
    child = Task.objects.create(list=home["errands"], parent=parent, title="Child")
    grandchild = Task.objects.create(list=home["reading"], parent=child, title="Grandchild")
    response = client_for(ana).delete(url(home["project"].pk))
    assert response.status_code == 204
    assert not Task.objects.filter(pk__in=[parent.pk, child.pk, grandchild.pk]).exists()


def test_delete_removes_the_projects_trashed_tasks_too(client_for, ana, home):
    make_project(ana, "Work", sort_order=1)
    trashed = Task.objects.create(
        list=home["errands"], title="In the Trash", deleted_at=timezone.now()
    )
    response = client_for(ana).delete(url(home["project"].pk))
    assert response.status_code == 204
    assert not Task.objects.filter(pk=trashed.pk).exists()


def test_delete_removes_notes_on_the_projects_tasks(client_for, ana, home):
    make_project(ana, "Work", sort_order=1)
    task = Task.objects.create(list=home["errands"], title="Task")
    note = Note.objects.create(task=task, content="<p>A note</p>")
    response = client_for(ana).delete(url(home["project"].pk))
    assert response.status_code == 204
    assert not Note.objects.filter(pk=note.pk).exists()


def test_delete_removes_attachments_and_deletes_their_stored_files(
    client_for, ana, home, settings, tmp_path
):
    settings.MEDIA_ROOT = str(tmp_path)
    make_project(ana, "Work", sort_order=1)
    task = Task.objects.create(list=home["errands"], title="Task")
    uploaded = Attachment(task=task, kind=Attachment.Kind.FILE, name="receipt.txt", size=5, mime="text/plain")
    uploaded.file.save("receipt.txt", ContentFile(b"hello"), save=True)
    link = Attachment.objects.create(
        task=task, kind=Attachment.Kind.LINK, name="Site", url="https://example.com"
    )
    path = uploaded.file.path
    assert os.path.exists(path)

    response = client_for(ana).delete(url(home["project"].pk))

    assert response.status_code == 204
    assert not Attachment.objects.filter(pk__in=[uploaded.pk, link.pk]).exists()
    assert not os.path.exists(path)


def test_delete_leaves_the_users_other_projects_untouched(client_for, ana, home):
    doomed = make_project(ana, "Work", sort_order=1)
    Task.objects.create(list=home["errands"], title="Keep me")
    response = client_for(ana).delete(url(doomed.pk))
    assert response.status_code == 204
    assert Project.objects.filter(pk=home["project"].pk).exists()
    assert TaskList.objects.filter(project=home["project"]).count() == 2
    assert Task.objects.filter(title="Keep me").exists()


def test_deleting_the_users_last_project_is_400_with_a_detail_message(client_for, ana, home):
    response = client_for(ana).delete(url(home["project"].pk))
    assert response.status_code == 400
    assert response.json() == {"detail": "A project is needed."}
    assert Project.objects.filter(pk=home["project"].pk).exists()
    assert TaskList.objects.filter(project=home["project"]).count() == 2


def test_last_project_rule_ignores_other_users_projects(client_for, ana, bob, home):
    make_project(bob, "Bob's")
    make_project(bob, "Bob's second")
    response = client_for(ana).delete(url(home["project"].pk))
    assert response.status_code == 400
    assert response.json() == {"detail": "A project is needed."}
    assert Project.objects.filter(pk=home["project"].pk).exists()


def test_bob_gets_404_deleting_each_of_anas_projects(client_for, ana, bob):
    projects = [make_project(ana, "Home", sort_order=0), make_project(ana, "Work", sort_order=1)]
    for project in projects:
        response = client_for(bob).delete(url(project.pk))
        assert response.status_code == 404
    assert Project.objects.filter(owner=ana).count() == 2


# --- shared rules for writes -------------------------------------------------------


def test_session_write_without_a_csrf_token_is_refused(ana):
    client = APIClient(enforce_csrf_checks=True)
    client.force_login(ana)
    response = client.post(PROJECTS, {"name": "Work"}, format="json")
    assert response.status_code == 403
    assert not Project.objects.filter(owner=ana).exists()


def test_session_write_with_the_csrf_token_from_auth_csrf_succeeds(ana):
    client = APIClient(enforce_csrf_checks=True)
    client.force_login(ana)
    token = client.get("/api/auth/csrf").json()["csrfToken"]
    response = client.post(PROJECTS, {"name": "Work"}, format="json", HTTP_X_CSRFTOKEN=token)
    assert response.status_code == 201


@pytest.mark.parametrize("write", ["create", "rename", "delete"])
def test_every_project_write_logs_a_line_at_info_with_resource_id_and_user(
    client_for, ana, caplog, write
):
    project = make_project(ana, "Home")
    make_project(ana, "Work", sort_order=1)
    client = client_for(ana)
    with caplog.at_level(logging.INFO):
        if write == "create":
            response = client.post(PROJECTS, {"name": "Newest"}, format="json")
        elif write == "rename":
            response = client.patch(url(project.pk), {"name": "Renamed"}, format="json")
        else:
            response = client.delete(url(project.pk))

    expected_status = {"create": 201, "rename": 200, "delete": 204}[write]
    assert response.status_code == expected_status
    written_id = response.json()["id"] if write == "create" else project.pk

    info_tokens = [
        record.getMessage().split() for record in caplog.records if record.levelno == logging.INFO
    ]
    assert any(
        str(written_id) in tokens and str(ana.pk) in tokens and any("project" in t for t in tokens)
        for tokens in info_tokens
    ), f"no INFO log line naming the project {written_id} and user {ana.pk}: {info_tokens}"
