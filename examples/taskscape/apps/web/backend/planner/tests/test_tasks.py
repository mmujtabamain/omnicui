"""Contract tests for the `tasks` resource (API.md, section "Tasks: `tasks`").

Written from the contract alone; the implementation was not consulted. Fixtures come from
backend/conftest.py: `ana` owns `home` (project "Home" with lists "errands" and "reading"),
`bob` owns nothing until a test asks for `bob_list`.
"""
import logging
from datetime import datetime, timedelta

import pytest
from django.utils import timezone
from rest_framework.test import APIClient

from planner.models import Attachment, Note, Project, Task, TaskList

pytestmark = pytest.mark.django_db

TASKS = "/api/tasks"
TASK_FIELDS = {
    "id", "list", "parent", "title", "notes", "done", "done_at", "sort_order",
    "created_at", "updated_at", "attachments", "note_items",
}


def make_task(lst, title, parent=None, sort_order=0, **extra):
    return Task.objects.create(list=lst, parent=parent, title=title, sort_order=sort_order, **extra)


def ids(resp):
    return [t["id"] for t in resp.json()]


def titles(resp):
    return [t["title"] for t in resp.json()]


def fresh(task):
    return Task.objects.get(pk=task.pk)


def assert_iso_utc(value):
    assert isinstance(value, str), f"expected an ISO 8601 string, got {value!r}"
    parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    assert parsed.tzinfo is not None, f"timestamp {value!r} has no offset"
    assert parsed.utcoffset() == timedelta(0), f"timestamp {value!r} is not UTC"


@pytest.fixture
def api(ana, client_for):
    """Client signed in as ana."""
    return client_for(ana)


@pytest.fixture
def bob_api(bob, client_for):
    """Client signed in as bob."""
    return client_for(bob)


@pytest.fixture
def bob_list(bob):
    """A list owned by bob, for cross-user checks."""
    project = Project.objects.create(owner=bob, name="Bob's project")
    return TaskList.objects.create(project=project, name="Bob's list", sort_order=0)


@pytest.fixture
def work_list(ana):
    """A list in a second project of ana's, for moves between her projects."""
    project = Project.objects.create(owner=ana, name="Work")
    return TaskList.objects.create(project=project, name="Work list", sort_order=0)


@pytest.fixture
def ana_task(home):
    """Ana's task with one child, in her errands list."""
    parent = make_task(home["errands"], "Ana's task")
    make_task(home["errands"], "Ana's child", parent=parent)
    return parent


# --- GET tasks?list= -------------------------------------------------------------------------

def test_list_scope_returns_only_live_tasks_of_that_list(api, home):
    mine = make_task(home["errands"], "in errands")
    make_task(home["reading"], "in reading")
    resp = api.get(TASKS, {"list": home["errands"].pk})
    assert resp.status_code == 200
    assert ids(resp) == [mine.pk]


def test_list_scope_includes_nested_tasks_at_every_depth(api, home):
    parent = make_task(home["errands"], "parent")
    child = make_task(home["errands"], "child", parent=parent)
    grandchild = make_task(home["errands"], "grandchild", parent=child)
    resp = api.get(TASKS, {"list": home["errands"].pk})
    assert set(ids(resp)) == {parent.pk, child.pk, grandchild.pk}


def test_list_scope_excludes_trashed_tasks(api, home):
    live = make_task(home["errands"], "live")
    make_task(home["errands"], "trashed", deleted_at=timezone.now())
    resp = api.get(TASKS, {"list": home["errands"].pk})
    assert ids(resp) == [live.pk]


def test_list_orders_tasks_by_sort_order(api, home):
    lst = home["errands"]
    make_task(lst, "sort 2", sort_order=2)
    make_task(lst, "sort 0 first", sort_order=0)
    make_task(lst, "sort 1", sort_order=1)
    make_task(lst, "sort 0 second", sort_order=0)
    resp = api.get(TASKS, {"list": lst.pk})
    assert titles(resp) == ["sort 0 first", "sort 0 second", "sort 1", "sort 2"]


def test_list_breaks_sort_order_ties_by_created_at(api, home):
    lst = home["errands"]
    first = make_task(lst, "first tie", sort_order=0)
    second = make_task(lst, "second tie", sort_order=0)
    third = make_task(lst, "third tie", sort_order=0)
    resp = api.get(TASKS, {"list": lst.pk})
    assert ids(resp) == [first.pk, second.pk, third.pk]


def test_nested_tasks_sort_together_with_top_level_tasks(api, home):
    lst = home["errands"]
    parent = make_task(lst, "parent", sort_order=1)
    child = make_task(lst, "child", parent=parent, sort_order=0)  # created first
    sibling = make_task(lst, "sibling", sort_order=0)             # created second
    resp = api.get(TASKS, {"list": lst.pk})
    assert ids(resp) == [child.pk, sibling.pk, parent.pk]


def test_list_is_returned_whole_as_a_bare_array(api, home):
    lst = home["errands"]
    for n in range(30):
        make_task(lst, f"task {n}", sort_order=n)
    resp = api.get(TASKS, {"list": lst.pk})
    assert resp.status_code == 200
    body = resp.json()
    assert isinstance(body, list)
    assert len(body) == 30


def test_list_requires_a_signed_in_user(home):
    resp = APIClient().get(TASKS, {"list": home["errands"].pk})
    assert resp.status_code in (401, 403)


# --- GET tasks?project= and GET tasks (no scope) ---------------------------------------------

def test_project_scope_returns_tasks_of_every_list_in_that_project(api, home):
    a = make_task(home["errands"], "errand")
    b = make_task(home["reading"], "book")
    resp = api.get(TASKS, {"project": home["project"].pk})
    assert resp.status_code == 200
    assert set(ids(resp)) == {a.pk, b.pk}


def test_project_scope_excludes_tasks_of_other_projects(api, home, work_list):
    make_task(home["errands"], "home task")
    work = make_task(work_list, "work task")
    resp = api.get(TASKS, {"project": work_list.project_id})
    assert ids(resp) == [work.pk]


def test_no_scope_returns_all_of_the_users_live_tasks(api, home, work_list):
    a = make_task(home["errands"], "home task")
    b = make_task(work_list, "work task")
    resp = api.get(TASKS)
    assert resp.status_code == 200
    assert set(ids(resp)) == {a.pk, b.pk}


def test_no_scope_never_includes_other_users_tasks(api, home, bob_list):
    mine = make_task(home["errands"], "mine")
    make_task(bob_list, "bob's")
    resp = api.get(TASKS)
    assert ids(resp) == [mine.pk]


def test_trashed_tasks_never_appear_without_a_scope(api, home):
    live = make_task(home["errands"], "live")
    make_task(home["errands"], "trashed", deleted_at=timezone.now())
    resp = api.get(TASKS)
    assert ids(resp) == [live.pk]


def test_trashed_tasks_never_appear_in_project_scope(api, home):
    live = make_task(home["errands"], "live")
    make_task(home["reading"], "trashed", deleted_at=timezone.now())
    resp = api.get(TASKS, {"project": home["project"].pk})
    assert ids(resp) == [live.pk]


# --- GET tasks?q= ----------------------------------------------------------------------------

def test_q_matches_title_case_insensitively(api, home):
    milk = make_task(home["errands"], "Buy MILK")
    make_task(home["errands"], "Call mum")
    resp = api.get(TASKS, {"q": "milk"})
    assert ids(resp) == [milk.pk]


def test_q_also_matches_notes(api, home):
    t = make_task(home["errands"], "Shopping", notes="get oat milk")
    make_task(home["errands"], "Unrelated", notes="nothing here")
    resp = api.get(TASKS, {"q": "oat milk"})
    assert ids(resp) == [t.pk]


def test_q_is_limited_to_the_list_scope(api, home):
    in_list = make_task(home["errands"], "milk run")
    make_task(home["reading"], "milk book")
    resp = api.get(TASKS, {"q": "milk", "list": home["errands"].pk})
    assert ids(resp) == [in_list.pk]


def test_q_is_limited_to_the_project_scope(api, home, work_list):
    in_project = make_task(home["errands"], "milk run")
    make_task(work_list, "milk work")
    resp = api.get(TASKS, {"q": "milk", "project": home["project"].pk})
    assert ids(resp) == [in_project.pk]


def test_q_without_a_scope_searches_all_of_the_users_tasks(api, home, work_list):
    a = make_task(home["errands"], "milk run")
    b = make_task(work_list, "milk work")
    make_task(home["reading"], "book")
    resp = api.get(TASKS, {"q": "milk"})
    assert set(ids(resp)) == {a.pk, b.pk}


def test_q_never_returns_trashed_tasks(api, home):
    make_task(home["errands"], "milk old", deleted_at=timezone.now())
    resp = api.get(TASKS, {"q": "milk"})
    assert resp.status_code == 200
    assert resp.json() == []


def test_q_with_no_match_answers_an_empty_array(api, home):
    make_task(home["errands"], "Call mum")
    resp = api.get(TASKS, {"q": "zzz-no-match"})
    assert resp.status_code == 200
    assert resp.json() == []


# --- Task body -------------------------------------------------------------------------------

def test_task_body_has_exactly_the_contract_fields(api, home):
    make_task(home["errands"], "Typed")
    body = api.get(TASKS, {"list": home["errands"].pk}).json()[0]
    assert set(body) == TASK_FIELDS


def test_task_body_field_values_and_types(api, home):
    t = make_task(home["errands"], "Typed", notes="a subtitle", sort_order=3)
    body = api.get(TASKS, {"list": home["errands"].pk}).json()[0]
    assert isinstance(body["id"], int) and body["id"] == t.pk
    assert isinstance(body["list"], int) and body["list"] == home["errands"].pk
    assert body["parent"] is None
    assert body["title"] == "Typed"
    assert body["notes"] == "a subtitle"
    assert body["done"] is False
    assert body["done_at"] is None
    assert isinstance(body["sort_order"], int) and body["sort_order"] == 3
    assert body["attachments"] == []
    assert body["note_items"] == []


def test_task_body_parent_is_the_parent_id_for_nested_tasks(api, home):
    parent = make_task(home["errands"], "parent")
    child = make_task(home["errands"], "child", parent=parent)
    resp = api.get(TASKS, {"list": home["errands"].pk})
    by_id = {t["id"]: t for t in resp.json()}
    assert by_id[child.pk]["parent"] == parent.pk
    assert by_id[parent.pk]["parent"] is None


def test_task_body_lists_its_attachments(api, home):
    t = make_task(home["errands"], "With link")
    Attachment.objects.create(task=t, kind=Attachment.Kind.LINK, name="Spec",
                              url="https://example.com/spec", size=0)
    body = api.get(TASKS, {"list": home["errands"].pk}).json()[0]
    (att,) = body["attachments"]
    assert set(att) == {"id", "task", "kind", "name", "url", "size", "mime", "created_at"}
    assert att["task"] == t.pk
    assert att["kind"] == "link"
    assert att["name"] == "Spec"
    assert att["url"] == "https://example.com/spec"


def test_task_body_lists_its_note_items(api, home):
    t = make_task(home["errands"], "With notes")
    Note.objects.create(task=t, content="<p>first</p>", sort_order=0)
    body = api.get(TASKS, {"list": home["errands"].pk}).json()[0]
    (note,) = body["note_items"]
    assert set(note) == {"id", "task", "content", "sort_order", "created_at", "updated_at"}
    assert note["task"] == t.pk
    assert note["content"] == "<p>first</p>"


def test_task_times_are_iso_8601_utc(api, home):
    make_task(home["errands"], "Timed", done=True, done_at=timezone.now())
    body = api.get(TASKS, {"list": home["errands"].pk}).json()[0]
    assert_iso_utc(body["created_at"])
    assert_iso_utc(body["updated_at"])
    assert_iso_utc(body["done_at"])


# --- POST tasks ------------------------------------------------------------------------------

def test_create_answers_201_with_the_new_task(api, home):
    resp = api.post(TASKS, {"list": home["errands"].pk, "title": "Milk"}, format="json")
    assert resp.status_code == 201
    body = resp.json()
    assert set(body) == TASK_FIELDS
    assert body["title"] == "Milk"
    assert body["list"] == home["errands"].pk
    assert body["parent"] is None
    assert body["done"] is False
    assert body["done_at"] is None


def test_create_defaults_notes_to_empty_string(api, home):
    resp = api.post(TASKS, {"list": home["errands"].pk, "title": "Bare"}, format="json")
    assert resp.status_code == 201
    assert resp.json()["notes"] == ""


def test_create_keeps_given_notes(api, home):
    resp = api.post(TASKS, {"list": home["errands"].pk, "title": "Noted", "notes": "2 litres"},
                    format="json")
    assert resp.status_code == 201
    assert resp.json()["notes"] == "2 litres"


def test_create_appends_at_the_end_of_top_level_siblings(api, home):
    lst = home["errands"]
    make_task(lst, "existing a", sort_order=0)
    make_task(lst, "existing b", sort_order=4)
    resp = api.post(TASKS, {"list": lst.pk, "title": "new"}, format="json")
    assert resp.status_code == 201
    assert resp.json()["sort_order"] > 4
    assert titles(api.get(TASKS, {"list": lst.pk}))[-1] == "new"


def test_create_with_parent_nests_under_it(api, home):
    parent = make_task(home["errands"], "parent")
    resp = api.post(TASKS, {"list": home["errands"].pk, "title": "child", "parent": parent.pk},
                    format="json")
    assert resp.status_code == 201
    assert resp.json()["parent"] == parent.pk


def test_create_with_parent_appends_at_the_end_of_the_parents_children(api, home):
    lst = home["errands"]
    parent = make_task(lst, "parent")
    make_task(lst, "child a", parent=parent, sort_order=0)
    make_task(lst, "child b", parent=parent, sort_order=1)
    make_task(lst, "other top level", sort_order=9)
    resp = api.post(TASKS, {"list": lst.pk, "title": "child c", "parent": parent.pk}, format="json")
    assert resp.status_code == 201
    children = [t["title"] for t in api.get(TASKS, {"list": lst.pk}).json() if t["parent"] == parent.pk]
    assert children == ["child a", "child b", "child c"]


def test_create_with_empty_title_is_400(api, home):
    resp = api.post(TASKS, {"list": home["errands"].pk, "title": ""}, format="json")
    assert resp.status_code == 400
    assert "title" in resp.json()
    assert not Task.objects.filter(list=home["errands"], title="").exists()


def test_create_with_parent_from_another_list_is_400(api, home):
    elsewhere = make_task(home["reading"], "a book")
    resp = api.post(TASKS, {"list": home["errands"].pk, "title": "child", "parent": elsewhere.pk},
                    format="json")
    assert resp.status_code == 400


def test_create_with_a_trashed_parent_is_400(api, home):
    parent = make_task(home["errands"], "old parent", deleted_at=timezone.now())
    resp = api.post(TASKS, {"list": home["errands"].pk, "title": "child", "parent": parent.pk},
                    format="json")
    assert resp.status_code == 400


def test_create_writes_one_info_log_line(api, ana, home, caplog):
    caplog.set_level(logging.INFO)
    resp = api.post(TASKS, {"list": home["errands"].pk, "title": "Logged"}, format="json")
    assert resp.status_code == 201
    lines = [r.getMessage() for r in caplog.records
             if r.levelno == logging.INFO and "tasks" in r.getMessage()]
    assert len(lines) == 1
    tokens = lines[0].split()
    assert str(resp.json()["id"]) in tokens
    assert str(ana.pk) in tokens


# --- PATCH tasks/{id} ------------------------------------------------------------------------

def test_patch_updates_title_and_notes_and_answers_200(api, home):
    t = make_task(home["errands"], "old")
    resp = api.patch(f"{TASKS}/{t.pk}", {"title": "new", "notes": "fresh"}, format="json")
    assert resp.status_code == 200
    assert resp.json()["title"] == "new"
    assert resp.json()["notes"] == "fresh"
    assert fresh(t).title == "new"


def test_patch_done_true_stamps_done_at(api, home):
    t = make_task(home["errands"], "finish me")
    resp = api.patch(f"{TASKS}/{t.pk}", {"done": True}, format="json")
    assert resp.status_code == 200
    assert resp.json()["done"] is True
    assert_iso_utc(resp.json()["done_at"])
    assert fresh(t).done_at is not None


def test_patch_done_false_clears_done_at(api, home):
    t = make_task(home["errands"], "toggle me")
    api.patch(f"{TASKS}/{t.pk}", {"done": True}, format="json")
    resp = api.patch(f"{TASKS}/{t.pk}", {"done": False}, format="json")
    assert resp.status_code == 200
    assert resp.json()["done"] is False
    assert resp.json()["done_at"] is None
    assert fresh(t).done_at is None


def test_completing_a_task_completes_its_live_descendants(api, home):
    parent = make_task(home["errands"], "parent")
    child = make_task(home["errands"], "child", parent=parent)
    grandchild = make_task(home["errands"], "grandchild", parent=child)
    resp = api.patch(f"{TASKS}/{parent.pk}", {"done": True}, format="json")
    assert resp.status_code == 200
    for t in (parent, child, grandchild):
        assert fresh(t).done is True
        assert fresh(t).done_at is not None


def test_completing_a_task_leaves_trashed_descendants_alone(api, home):
    parent = make_task(home["errands"], "parent")
    trashed = make_task(home["errands"], "trashed child", parent=parent, deleted_at=timezone.now())
    api.patch(f"{TASKS}/{parent.pk}", {"done": True}, format="json")
    assert fresh(trashed).done is False


def test_uncompleting_a_task_uncompletes_only_itself(api, home):
    parent = make_task(home["errands"], "parent", done=True, done_at=timezone.now())
    child = make_task(home["errands"], "child", parent=parent, done=True, done_at=timezone.now())
    resp = api.patch(f"{TASKS}/{parent.pk}", {"done": False}, format="json")
    assert resp.status_code == 200
    assert fresh(parent).done is False
    assert fresh(child).done is True


def test_completing_a_child_does_not_complete_its_parent(api, home):
    parent = make_task(home["errands"], "parent")
    child = make_task(home["errands"], "child", parent=parent)
    api.patch(f"{TASKS}/{child.pk}", {"done": True}, format="json")
    assert fresh(child).done is True
    assert fresh(parent).done is False


# --- POST tasks/{id}/move --------------------------------------------------------------------

def test_move_to_another_list_of_the_user(api, home):
    t = make_task(home["errands"], "move me")
    resp = api.post(f"{TASKS}/{t.pk}/move", {"list": home["reading"].pk}, format="json")
    assert resp.status_code == 200
    assert fresh(t).list_id == home["reading"].pk


def test_move_carries_the_subtree_to_the_new_list(api, home):
    parent = make_task(home["errands"], "parent")
    child = make_task(home["errands"], "child", parent=parent)
    grandchild = make_task(home["errands"], "grandchild", parent=child)
    api.post(f"{TASKS}/{parent.pk}/move", {"list": home["reading"].pk}, format="json")
    for t in (parent, child, grandchild):
        assert fresh(t).list_id == home["reading"].pk


def test_move_into_a_list_of_another_project_of_the_user(api, home, work_list):
    t = make_task(home["errands"], "cross project")
    resp = api.post(f"{TASKS}/{t.pk}/move", {"list": work_list.pk}, format="json")
    assert resp.status_code == 200
    assert fresh(t).list_id == work_list.pk


def test_move_under_a_new_parent(api, home):
    new_parent = make_task(home["errands"], "new parent")
    t = make_task(home["errands"], "wanderer")
    resp = api.post(f"{TASKS}/{t.pk}/move", {"parent": new_parent.pk}, format="json")
    assert resp.status_code == 200
    assert fresh(t).parent_id == new_parent.pk


def test_move_with_null_parent_makes_it_top_level(api, home):
    parent = make_task(home["errands"], "parent")
    child = make_task(home["errands"], "child", parent=parent)
    resp = api.post(f"{TASKS}/{child.pk}/move", {"parent": None}, format="json")
    assert resp.status_code == 200
    assert fresh(child).parent_id is None


def test_move_sets_the_sort_order(api, home):
    t = make_task(home["errands"], "positioned")
    resp = api.post(f"{TASKS}/{t.pk}/move", {"sort_order": 7}, format="json")
    assert resp.status_code == 200
    assert fresh(t).sort_order == 7


def test_move_under_itself_is_400(api, home):
    t = make_task(home["errands"], "loop")
    resp = api.post(f"{TASKS}/{t.pk}/move", {"parent": t.pk}, format="json")
    assert resp.status_code == 400
    assert fresh(t).parent_id is None


def test_move_under_its_own_descendant_is_400(api, home):
    top = make_task(home["errands"], "top")
    mid = make_task(home["errands"], "mid", parent=top)
    low = make_task(home["errands"], "low", parent=mid)
    resp = api.post(f"{TASKS}/{top.pk}/move", {"parent": low.pk}, format="json")
    assert resp.status_code == 400
    assert fresh(top).parent_id is None


def test_move_into_another_users_list_is_404(api, home, bob_list):
    t = make_task(home["errands"], "mine")
    resp = api.post(f"{TASKS}/{t.pk}/move", {"list": bob_list.pk}, format="json")
    assert resp.status_code == 404
    assert fresh(t).list_id == home["errands"].pk


# --- POST tasks/{id}/reorder -----------------------------------------------------------------

def test_reorder_answers_200_and_sets_sort_order(api, home):
    t = make_task(home["errands"], "reorder me")
    resp = api.post(f"{TASKS}/{t.pk}/reorder", {"sort_order": 7}, format="json")
    assert resp.status_code == 200
    assert fresh(t).sort_order == 7


def test_reorder_moves_the_task_among_its_siblings(api, home):
    lst = home["errands"]
    a = make_task(lst, "a", sort_order=10)
    b = make_task(lst, "b", sort_order=20)
    c = make_task(lst, "c", sort_order=30)
    api.post(f"{TASKS}/{c.pk}/reorder", {"sort_order": 5}, format="json")
    assert ids(api.get(TASKS, {"list": lst.pk})) == [c.pk, a.pk, b.pk]


# --- POST tasks/bulk -------------------------------------------------------------------------

def test_bulk_done_completes_the_listed_tasks(api, home):
    a = make_task(home["errands"], "a")
    b = make_task(home["errands"], "b")
    resp = api.post(f"{TASKS}/bulk", {"ids": [a.pk, b.pk], "action": "done"}, format="json")
    assert resp.status_code == 200
    assert fresh(a).done is True
    assert fresh(b).done is True


def test_bulk_undone_uncompletes_the_listed_tasks(api, home):
    a = make_task(home["errands"], "a", done=True, done_at=timezone.now())
    b = make_task(home["errands"], "b", done=True, done_at=timezone.now())
    resp = api.post(f"{TASKS}/bulk", {"ids": [a.pk, b.pk], "action": "undone"}, format="json")
    assert resp.status_code == 200
    assert fresh(a).done is False
    assert fresh(b).done is False


def test_bulk_trash_answers_the_trashed_ids_including_subtrees(api, home):
    parent = make_task(home["errands"], "parent")
    child = make_task(home["errands"], "child", parent=parent)
    resp = api.post(f"{TASKS}/bulk", {"ids": [parent.pk], "action": "trash"}, format="json")
    assert resp.status_code == 200
    assert set(resp.json()) == {"trashed"}
    assert set(resp.json()["trashed"]) == {parent.pk, child.pk}


def test_bulk_trash_moves_every_subtree_to_the_trash(api, home):
    parent = make_task(home["errands"], "parent")
    child = make_task(home["errands"], "child", parent=parent)
    api.post(f"{TASKS}/bulk", {"ids": [parent.pk], "action": "trash"}, format="json")
    assert fresh(parent).deleted_at is not None
    assert fresh(child).deleted_at is not None


def test_bulk_move_moves_the_listed_tasks_into_the_list(api, home):
    a = make_task(home["errands"], "a")
    b = make_task(home["errands"], "b")
    resp = api.post(f"{TASKS}/bulk",
                    {"ids": [a.pk, b.pk], "action": "move", "list": home["reading"].pk},
                    format="json")
    assert resp.status_code == 200
    assert fresh(a).list_id == home["reading"].pk
    assert fresh(b).list_id == home["reading"].pk


def test_bulk_move_without_a_list_is_400(api, home):
    a = make_task(home["errands"], "a")
    resp = api.post(f"{TASKS}/bulk", {"ids": [a.pk], "action": "move"}, format="json")
    assert resp.status_code == 400
    assert fresh(a).list_id == home["errands"].pk


def test_bulk_unknown_action_is_400(api, home):
    a = make_task(home["errands"], "a")
    resp = api.post(f"{TASKS}/bulk", {"ids": [a.pk], "action": "explode"}, format="json")
    assert resp.status_code == 400
    assert fresh(a).done is False


def test_bulk_move_into_another_users_list_is_404(api, home, bob_list):
    a = make_task(home["errands"], "a")
    resp = api.post(f"{TASKS}/bulk", {"ids": [a.pk], "action": "move", "list": bob_list.pk},
                    format="json")
    assert resp.status_code == 404
    assert fresh(a).list_id == home["errands"].pk


def test_bulk_with_an_unowned_id_is_404_and_completes_nothing(api, home, bob_list):
    own = make_task(home["errands"], "own")
    theirs = make_task(bob_list, "bob's")
    resp = api.post(f"{TASKS}/bulk", {"ids": [own.pk, theirs.pk], "action": "done"}, format="json")
    assert resp.status_code == 404
    assert fresh(own).done is False
    assert fresh(theirs).done is False


def test_bulk_with_an_unowned_id_is_404_and_trashes_nothing(api, home, bob_list):
    own = make_task(home["errands"], "own")
    theirs = make_task(bob_list, "bob's")
    resp = api.post(f"{TASKS}/bulk", {"ids": [own.pk, theirs.pk], "action": "trash"}, format="json")
    assert resp.status_code == 404
    assert fresh(own).deleted_at is None
    assert fresh(theirs).deleted_at is None


# --- DELETE tasks/{id} -----------------------------------------------------------------------

def test_delete_answers_200_with_the_trashed_ids(api, home):
    parent = make_task(home["errands"], "parent")
    child = make_task(home["errands"], "child", parent=parent)
    resp = api.delete(f"{TASKS}/{parent.pk}")
    assert resp.status_code == 200
    assert set(resp.json()) == {"trashed"}
    assert set(resp.json()["trashed"]) == {parent.pk, child.pk}


def test_delete_trashes_the_task_and_its_live_subtree(api, home):
    parent = make_task(home["errands"], "parent")
    child = make_task(home["errands"], "child", parent=parent)
    grandchild = make_task(home["errands"], "grandchild", parent=child)
    api.delete(f"{TASKS}/{parent.pk}")
    for t in (parent, child, grandchild):
        assert fresh(t).deleted_at is not None


def test_delete_stamps_one_instant_on_the_whole_subtree(api, home):
    parent = make_task(home["errands"], "parent")
    child = make_task(home["errands"], "child", parent=parent)
    grandchild = make_task(home["errands"], "grandchild", parent=child)
    api.delete(f"{TASKS}/{parent.pk}")
    stamps = {fresh(t).deleted_at for t in (parent, child, grandchild)}
    assert len(stamps) == 1
    assert None not in stamps


def test_delete_leaves_already_trashed_descendants_as_they_were(api, home):
    parent = make_task(home["errands"], "parent")
    earlier = timezone.now() - timedelta(days=3)
    old_trashed = make_task(home["errands"], "old trash", parent=parent, deleted_at=earlier)
    live_child = make_task(home["errands"], "live child", parent=parent)
    resp = api.delete(f"{TASKS}/{parent.pk}")
    assert resp.status_code == 200
    assert set(resp.json()["trashed"]) == {parent.pk, live_child.pk}
    assert fresh(old_trashed).deleted_at == earlier


def test_deleted_task_disappears_from_listings(api, home):
    t = make_task(home["errands"], "gone soon")
    api.delete(f"{TASKS}/{t.pk}")
    assert t.pk not in ids(api.get(TASKS, {"list": home["errands"].pk}))
    assert t.pk not in ids(api.get(TASKS))


# --- GET tasks/{id}/copy ---------------------------------------------------------------------

def test_copy_renders_the_subtree_as_a_markdown_checklist(api, home):
    parent = make_task(home["errands"], "Pack")
    child = make_task(home["errands"], "Boxes", parent=parent, done=True, done_at=timezone.now())
    make_task(home["errands"], "Tape", parent=child)
    resp = api.get(f"{TASKS}/{parent.pk}/copy")
    assert resp.status_code == 200
    assert set(resp.json()) == {"text"}
    assert resp.json()["text"].splitlines() == [
        "- [ ] Pack",
        "  - [x] Boxes",
        "    - [ ] Tape",
    ]


def test_copy_lists_siblings_in_sort_order(api, home):
    parent = make_task(home["errands"], "Pack")
    make_task(home["errands"], "late", parent=parent, sort_order=1)
    make_task(home["errands"], "early", parent=parent, sort_order=0)
    text = api.get(f"{TASKS}/{parent.pk}/copy").json()["text"]
    assert text.splitlines() == ["- [ ] Pack", "  - [ ] early", "  - [ ] late"]


def test_copy_omits_trashed_descendants(api, home):
    parent = make_task(home["errands"], "Pack")
    make_task(home["errands"], "Trashed", parent=parent, deleted_at=timezone.now())
    make_task(home["errands"], "Live", parent=parent)
    text = api.get(f"{TASKS}/{parent.pk}/copy").json()["text"]
    assert text.splitlines() == ["- [ ] Pack", "  - [ ] Live"]


# --- Another user's rows answer 404 ----------------------------------------------------------

def test_bob_gets_404_listing_ana_list(bob_api, home):
    resp = bob_api.get(TASKS, {"list": home["errands"].pk})
    assert resp.status_code == 404


def test_bob_gets_404_listing_ana_project(bob_api, home):
    resp = bob_api.get(TASKS, {"project": home["project"].pk})
    assert resp.status_code == 404


def test_bob_gets_404_patching_ana_task(bob_api, ana_task):
    resp = bob_api.patch(f"{TASKS}/{ana_task.pk}", {"title": "hijacked"}, format="json")
    assert resp.status_code == 404
    assert fresh(ana_task).title == "Ana's task"


def test_bob_gets_404_moving_ana_task_into_his_own_list(bob_api, ana_task, bob_list, home):
    resp = bob_api.post(f"{TASKS}/{ana_task.pk}/move", {"list": bob_list.pk}, format="json")
    assert resp.status_code == 404
    assert fresh(ana_task).list_id == home["errands"].pk


def test_bob_gets_404_reordering_ana_task(bob_api, ana_task):
    before = fresh(ana_task).sort_order
    resp = bob_api.post(f"{TASKS}/{ana_task.pk}/reorder", {"sort_order": 99}, format="json")
    assert resp.status_code == 404
    assert fresh(ana_task).sort_order == before


def test_bob_gets_404_deleting_ana_task(bob_api, ana_task):
    resp = bob_api.delete(f"{TASKS}/{ana_task.pk}")
    assert resp.status_code == 404
    assert fresh(ana_task).deleted_at is None


def test_bob_gets_404_copying_ana_task(bob_api, ana_task):
    resp = bob_api.get(f"{TASKS}/{ana_task.pk}/copy")
    assert resp.status_code == 404


def test_bob_gets_404_creating_a_task_in_ana_list(bob_api, home):
    before = Task.objects.filter(list=home["errands"]).count()
    resp = bob_api.post(TASKS, {"list": home["errands"].pk, "title": "intruder"}, format="json")
    assert resp.status_code == 404
    assert Task.objects.filter(list=home["errands"]).count() == before


@pytest.mark.parametrize("action", ["done", "undone", "trash", "move"])
def test_bob_gets_404_for_ana_rows_in_every_bulk_action(action, bob_api, bob_list, ana_task, home):
    body = {"ids": [ana_task.pk], "action": action}
    if action == "move":
        body["list"] = bob_list.pk
    resp = bob_api.post(f"{TASKS}/bulk", body, format="json")
    assert resp.status_code == 404
    parent = fresh(ana_task)
    assert parent.done is False
    assert parent.deleted_at is None
    assert parent.list_id == home["errands"].pk
    child = ana_task.children.get()
    assert child.deleted_at is None


def test_ana_gets_404_patching_a_task_of_bobs(api, bob_list):
    theirs = make_task(bob_list, "bob's task")
    resp = api.patch(f"{TASKS}/{theirs.pk}", {"done": True}, format="json")
    assert resp.status_code == 404
    assert fresh(theirs).done is False
