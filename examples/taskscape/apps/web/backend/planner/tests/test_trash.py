"""Contract tests for the Trash resource (API.md, "Trash: `trash`").

Written from the contract alone. Trashed state is set up through the ORM so each test controls
deleted_at exactly; the endpoints under test are the /api/trash routes.
"""
from datetime import timedelta

import pytest
from django.utils import timezone
from django.utils.dateparse import parse_datetime
from rest_framework.test import APIClient

from planner.models import Project, Task, TaskList

pytestmark = pytest.mark.django_db

TRASH = "/api/trash"
RESTORE = "/api/trash/restore"
PURGE = "/api/trash/purge"
EMPTY = "/api/trash/empty"


@pytest.fixture
def bobs_list(bob):
    project = Project.objects.create(owner=bob, name="Bob's project")
    return TaskList.objects.create(project=project, name="Bob's list")


def make_task(task_list, title, *, parent=None, deleted_at=None, sort_order=0):
    return Task.objects.create(
        list=task_list, parent=parent, title=title, deleted_at=deleted_at, sort_order=sort_order
    )


def trashed_subtree(task_list, when, title="Parent"):
    """A task with a child and a grandchild, all trashed at the same instant."""
    parent = make_task(task_list, title, deleted_at=when)
    child = make_task(task_list, f"{title} child", parent=parent, deleted_at=when)
    grandchild = make_task(task_list, f"{title} grandchild", parent=child, deleted_at=when)
    return parent, child, grandchild


def ana_rows_snapshot(user):
    """Every task of the user with its deleted_at, to prove a request changed nothing."""
    return sorted(Task.objects.filter(list__project__owner=user).values_list("id", "deleted_at"))


def listed_ids(client):
    response = client.get(TRASH)
    assert response.status_code == 200
    return [item["id"] for item in response.json()]


# GET trash


def test_get_trash_answers_200_with_a_list_of_exactly_the_documented_fields(ana, home, client_for):
    task = make_task(home["reading"], "Read chapter 3", deleted_at=timezone.now() - timedelta(hours=1))

    response = client_for(ana).get(TRASH)

    assert response.status_code == 200
    body = response.json()
    assert isinstance(body, list)
    assert len(body) == 1
    item = body[0]
    assert set(item) == {"id", "title", "list", "list_name", "deleted_at"}
    assert item["id"] == task.id
    assert item["title"] == "Read chapter 3"
    assert item["list"] == home["reading"].id
    assert item["list_name"] == "Reading"
    assert parse_datetime(item["deleted_at"]) == task.deleted_at


def test_get_trash_lists_most_recently_trashed_first(ana, home, client_for):
    now = timezone.now()
    # Created in a different order from the one they were trashed in.
    trashed_hour_ago = make_task(home["errands"], "Trashed an hour ago", deleted_at=now - timedelta(hours=1))
    trashed_long_ago = make_task(home["errands"], "Trashed three hours ago", deleted_at=now - timedelta(hours=3))
    trashed_two_ago = make_task(home["errands"], "Trashed two hours ago", deleted_at=now - timedelta(hours=2))

    assert listed_ids(client_for(ana)) == [trashed_hour_ago.id, trashed_two_ago.id, trashed_long_ago.id]


def test_get_trash_lists_only_the_top_most_task_of_a_trashed_subtree(ana, home, client_for):
    parent, _child, _grandchild = trashed_subtree(home["errands"], timezone.now() - timedelta(hours=2))

    assert listed_ids(client_for(ana)) == [parent.id]


def test_get_trash_lists_a_trashed_child_whose_parent_is_live(ana, home, client_for):
    live_parent = make_task(home["errands"], "Live parent")
    child = make_task(home["errands"], "Trashed child", parent=live_parent, deleted_at=timezone.now())

    assert listed_ids(client_for(ana)) == [child.id]


def test_get_trash_does_not_list_a_child_trashed_earlier_under_a_trashed_parent(ana, home, client_for):
    now = timezone.now()
    parent = make_task(home["errands"], "Parent", deleted_at=now - timedelta(hours=1))
    make_task(home["errands"], "Child trashed earlier", parent=parent, deleted_at=now - timedelta(hours=3))

    assert listed_ids(client_for(ana)) == [parent.id]


def test_get_trash_excludes_live_tasks(ana, home, client_for):
    make_task(home["errands"], "Still live")
    trashed = make_task(home["errands"], "In the trash", deleted_at=timezone.now())

    assert listed_ids(client_for(ana)) == [trashed.id]


def test_get_trash_is_an_empty_list_when_nothing_is_trashed(ana, home, client_for):
    make_task(home["errands"], "Live")

    response = client_for(ana).get(TRASH)

    assert response.status_code == 200
    assert response.json() == []


def test_get_trash_does_not_show_other_users_trashed_tasks(ana, bob, home, bobs_list, client_for):
    now = timezone.now()
    ana_task = make_task(home["errands"], "Ana's trashed task", deleted_at=now)
    bob_task = make_task(bobs_list, "Bob's trashed task", deleted_at=now)

    assert listed_ids(client_for(ana)) == [ana_task.id]
    assert listed_ids(client_for(bob)) == [bob_task.id]


# POST trash/restore


def test_restore_answers_200_with_the_restored_ids_of_the_subtree(ana, home, client_for):
    parent, child, grandchild = trashed_subtree(home["errands"], timezone.now() - timedelta(hours=1))

    response = client_for(ana).post(RESTORE, {"ids": [parent.id]}, format="json")

    assert response.status_code == 200
    assert "restored" in response.json()
    assert sorted(response.json()["restored"]) == sorted([parent.id, child.id, grandchild.id])


def test_restore_clears_deleted_at_on_the_task_and_its_subtree(ana, home, client_for):
    parent, child, grandchild = trashed_subtree(home["errands"], timezone.now() - timedelta(hours=1))

    client_for(ana).post(RESTORE, {"ids": [parent.id]}, format="json")

    for task in (parent, child, grandchild):
        task.refresh_from_db()
        assert task.deleted_at is None


def test_restore_leaves_a_child_trashed_at_another_moment_in_the_trash(ana, home, client_for):
    when = timezone.now() - timedelta(hours=2)
    parent = make_task(home["errands"], "Parent", deleted_at=when)
    earlier_child = make_task(
        home["errands"], "Child trashed a day earlier", parent=parent, deleted_at=when - timedelta(days=1)
    )

    response = client_for(ana).post(RESTORE, {"ids": [parent.id]}, format="json")

    assert response.status_code == 200
    assert parent.id in response.json()["restored"]
    assert earlier_child.id not in response.json()["restored"]
    parent.refresh_from_db()
    earlier_child.refresh_from_db()
    assert parent.deleted_at is None
    assert earlier_child.deleted_at is not None
    # The parent is live again, so the child that stayed behind is now the top-most trashed task.
    assert listed_ids(client_for(ana)) == [earlier_child.id]


def test_restore_of_a_child_whose_parent_is_still_trashed_moves_it_to_top_level(ana, home, client_for):
    parent, child, grandchild = trashed_subtree(home["errands"], timezone.now() - timedelta(hours=1))

    response = client_for(ana).post(RESTORE, {"ids": [child.id]}, format="json")

    assert response.status_code == 200
    child.refresh_from_db()
    assert child.deleted_at is None
    assert child.parent_id is None
    assert child.list_id == home["errands"].id
    # The child's own subtree (trashed at the same moment) comes back with it.
    grandchild.refresh_from_db()
    assert grandchild.deleted_at is None
    assert grandchild.parent_id == child.id
    # The parent is still in the Trash.
    parent.refresh_from_db()
    assert parent.deleted_at is not None


def test_restore_of_a_child_whose_parent_is_live_keeps_it_under_that_parent(ana, home, client_for):
    live_parent = make_task(home["errands"], "Live parent")
    child = make_task(home["errands"], "Trashed child", parent=live_parent, deleted_at=timezone.now())

    response = client_for(ana).post(RESTORE, {"ids": [child.id]}, format="json")

    assert response.status_code == 200
    child.refresh_from_db()
    assert child.deleted_at is None
    assert child.parent_id == live_parent.id


def test_restore_of_a_live_task_is_409_naming_it_and_changes_nothing(ana, home, client_for):
    live = make_task(home["errands"], "Live")
    before = ana_rows_snapshot(ana)

    response = client_for(ana).post(RESTORE, {"ids": [live.id]}, format="json")

    assert response.status_code == 409
    body = response.json()
    assert "detail" in body
    assert body["ids"] == [live.id]
    assert ana_rows_snapshot(ana) == before


def test_restore_mixing_a_stale_id_with_a_trashed_one_changes_nothing(ana, home, client_for):
    now = timezone.now()
    trashed = make_task(home["errands"], "Trashed", deleted_at=now)
    live = make_task(home["errands"], "Live")
    before = ana_rows_snapshot(ana)

    response = client_for(ana).post(RESTORE, {"ids": [trashed.id, live.id]}, format="json")

    assert response.status_code == 409
    assert response.json()["ids"] == [live.id]
    assert ana_rows_snapshot(ana) == before
    trashed.refresh_from_db()
    assert trashed.deleted_at is not None


def test_restore_of_an_id_already_restored_elsewhere_is_409(ana, home, client_for):
    trashed = make_task(home["errands"], "Trashed", deleted_at=timezone.now())
    client = client_for(ana)
    first = client.post(RESTORE, {"ids": [trashed.id]}, format="json")
    assert first.status_code == 200

    second = client.post(RESTORE, {"ids": [trashed.id]}, format="json")

    assert second.status_code == 409
    assert second.json()["ids"] == [trashed.id]


def test_restore_of_an_id_already_purged_elsewhere_is_409(ana, home, client_for):
    trashed = make_task(home["errands"], "Trashed", deleted_at=timezone.now())
    client = client_for(ana)
    purged = client.post(PURGE, {"ids": [trashed.id]}, format="json")
    assert purged.status_code == 204

    response = client.post(RESTORE, {"ids": [trashed.id]}, format="json")

    assert response.status_code == 409
    assert response.json()["ids"] == [trashed.id]


# POST trash/purge


def test_purge_answers_204_and_deletes_the_task_and_its_subtree(ana, home, client_for):
    parent, child, grandchild = trashed_subtree(home["errands"], timezone.now() - timedelta(hours=1))

    response = client_for(ana).post(PURGE, {"ids": [parent.id]}, format="json")

    assert response.status_code == 204
    assert response.content == b""
    assert not Task.objects.filter(id__in=[parent.id, child.id, grandchild.id]).exists()


def test_purge_leaves_other_trashed_tasks_alone(ana, home, client_for):
    now = timezone.now()
    purged = make_task(home["errands"], "Purged", deleted_at=now)
    kept = make_task(home["errands"], "Kept in the trash", deleted_at=now - timedelta(hours=1))

    response = client_for(ana).post(PURGE, {"ids": [purged.id]}, format="json")

    assert response.status_code == 204
    kept.refresh_from_db()
    assert kept.deleted_at is not None
    assert listed_ids(client_for(ana)) == [kept.id]


def test_purge_of_a_live_task_is_409_naming_it_and_changes_nothing(ana, home, client_for):
    live = make_task(home["errands"], "Live")
    before = ana_rows_snapshot(ana)

    response = client_for(ana).post(PURGE, {"ids": [live.id]}, format="json")

    assert response.status_code == 409
    body = response.json()
    assert "detail" in body
    assert body["ids"] == [live.id]
    assert ana_rows_snapshot(ana) == before


def test_purge_of_an_id_already_purged_is_409(ana, home, client_for):
    trashed = make_task(home["errands"], "Trashed", deleted_at=timezone.now())
    client = client_for(ana)
    assert client.post(PURGE, {"ids": [trashed.id]}, format="json").status_code == 204

    response = client.post(PURGE, {"ids": [trashed.id]}, format="json")

    assert response.status_code == 409
    assert response.json()["ids"] == [trashed.id]


# POST trash/empty


def test_empty_answers_204_and_deletes_everything_in_the_trash(ana, home, client_for):
    now = timezone.now()
    parent, child, grandchild = trashed_subtree(home["errands"], now - timedelta(hours=1))
    separately = make_task(home["reading"], "Trashed long ago", deleted_at=now - timedelta(days=5))

    response = client_for(ana).post(EMPTY)

    assert response.status_code == 204
    assert response.content == b""
    assert not Task.objects.filter(
        id__in=[parent.id, child.id, grandchild.id, separately.id]
    ).exists()
    assert listed_ids(client_for(ana)) == []


def test_empty_keeps_live_tasks(ana, home, client_for):
    live = make_task(home["errands"], "Live")
    make_task(home["errands"], "Trashed", deleted_at=timezone.now())

    response = client_for(ana).post(EMPTY)

    assert response.status_code == 204
    live.refresh_from_db()
    assert live.deleted_at is None


def test_empty_does_not_touch_other_users_trash(ana, bob, home, bobs_list, client_for):
    now = timezone.now()
    bob_trashed = make_task(bobs_list, "Bob's trashed task", deleted_at=now)
    make_task(home["errands"], "Ana's trashed task", deleted_at=now)

    response = client_for(ana).post(EMPTY)

    assert response.status_code == 204
    assert listed_ids(client_for(bob)) == [bob_trashed.id]


# Sign-in and ownership


@pytest.mark.parametrize("method,url", [("get", TRASH), ("post", RESTORE), ("post", PURGE), ("post", EMPTY)])
def test_trash_endpoints_need_a_signed_in_user(ana, home, method, url):
    trashed = make_task(home["errands"], "Trashed", deleted_at=timezone.now())
    client = APIClient()

    if method == "get":
        response = client.get(url)
    else:
        response = client.post(url, {"ids": [trashed.id]}, format="json")

    assert response.status_code in (401, 403)
    trashed.refresh_from_db()
    assert trashed.deleted_at is not None


@pytest.mark.parametrize("action", ["restore", "purge"])
@pytest.mark.parametrize("row", ["trashed_task", "trashed_child", "live_task"])
def test_bob_gets_404_for_each_of_anas_rows(ana, bob, home, client_for, action, row):
    when = timezone.now() - timedelta(hours=1)
    parent = make_task(home["errands"], "Ana's trashed task", deleted_at=when)
    child = make_task(
        home["errands"], "Ana's trashed child", parent=parent, deleted_at=when - timedelta(days=1)
    )
    live = make_task(home["reading"], "Ana's live task")
    target = {"trashed_task": parent, "trashed_child": child, "live_task": live}[row]
    before = ana_rows_snapshot(ana)

    response = client_for(bob).post(f"/api/trash/{action}", {"ids": [target.id]}, format="json")

    assert response.status_code == 404
    assert ana_rows_snapshot(ana) == before


# Validation (400)


@pytest.mark.parametrize("url", [RESTORE, PURGE])
def test_missing_ids_is_a_400_with_a_field_error(ana, home, client_for, url):
    trashed = make_task(home["errands"], "Trashed", deleted_at=timezone.now())
    before = ana_rows_snapshot(ana)

    response = client_for(ana).post(url, {}, format="json")

    assert response.status_code == 400
    assert "ids" in response.json()
    assert ana_rows_snapshot(ana) == before
    trashed.refresh_from_db()
    assert trashed.deleted_at is not None


@pytest.mark.parametrize("url", [RESTORE, PURGE])
@pytest.mark.parametrize("ids", ["5", 5, ["not-an-id"]])
def test_ids_must_be_a_list_of_integers(ana, home, client_for, url, ids):
    make_task(home["errands"], "Trashed", deleted_at=timezone.now())
    before = ana_rows_snapshot(ana)

    response = client_for(ana).post(url, {"ids": ids}, format="json")

    assert response.status_code == 400
    assert "ids" in response.json()
    assert ana_rows_snapshot(ana) == before
