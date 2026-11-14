"""Contract tests for the `lists` resource (API.md, section "Lists: `lists`").

Written from the contract alone. Each rule of the contract has its own test.
"""
from datetime import datetime, timezone as dt_timezone

import pytest
from django.utils import timezone
from rest_framework.test import APIClient

from planner.models import Note, Project, Task, TaskList

LIST_FIELDS = {"id", "project", "name", "sort_order", "created_at", "updated_at", "task_count", "done_count"}


# ---- helpers -------------------------------------------------------------------------------------


def at(year, month, day):
    return datetime(year, month, day, tzinfo=dt_timezone.utc)


def make_list(project, name, sort_order=0, created_at=None):
    task_list = TaskList.objects.create(project=project, name=name, sort_order=sort_order)
    if created_at is not None:
        TaskList.objects.filter(pk=task_list.pk).update(created_at=created_at)
    return task_list


def make_task(task_list, title, parent=None, done=False, trashed=False, sort_order=0, notes="", created_at=None):
    task = Task.objects.create(
        list=task_list,
        parent=parent,
        title=title,
        notes=notes,
        done=done,
        done_at=timezone.now() if done else None,
        sort_order=sort_order,
        deleted_at=timezone.now() if trashed else None,
    )
    if created_at is not None:
        Task.objects.filter(pk=task.pk).update(created_at=created_at)
    return task


def get_lists(client, project=None):
    params = {} if project is None else {"project": project.id}
    resp = client.get("/api/lists", params)
    assert resp.status_code == 200
    return resp.json()


def ids(rows):
    return [row["id"] for row in rows]


def row_for(rows, list_id):
    return next(row for row in rows if row["id"] == list_id)


def export_of(client, list_id):
    resp = client.get(f"/api/lists/{list_id}/export")
    assert resp.status_code == 200
    return resp.json()


def titles(nodes):
    return [node["title"] for node in nodes]


def parse_iso_utc(value):
    parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    assert parsed.utcoffset() is not None, f"{value!r} has no UTC offset"
    assert parsed.utcoffset().total_seconds() == 0, f"{value!r} is not UTC"
    return parsed


@pytest.fixture
def work(ana):
    """Another project of Ana's, with no lists yet."""
    return Project.objects.create(owner=ana, name="Work")


@pytest.fixture
def bobs_list(bob):
    """One of Bob's lists, in a project of his own."""
    project = Project.objects.create(owner=bob, name="Bob's")
    return make_list(project, "Bob's list")


# ---- GET lists ------------------------------------------------------------------------------------


def test_get_with_project_returns_only_that_projects_lists(home, work, ana, client_for):
    make_list(work, "Elsewhere")
    rows = get_lists(client_for(ana), home["project"])
    assert sorted(ids(rows)) == sorted([home["errands"].id, home["reading"].id])


def test_get_with_project_answers_an_empty_array_for_a_project_without_lists(work, ana, client_for):
    assert get_lists(client_for(ana), work) == []


def test_get_with_project_orders_lists_by_sort_order(home, ana, client_for):
    TaskList.objects.filter(pk=home["errands"].pk).update(sort_order=5)
    TaskList.objects.filter(pk=home["reading"].pk).update(sort_order=2)
    rows = get_lists(client_for(ana), home["project"])
    assert ids(rows) == [home["reading"].id, home["errands"].id]


def test_get_breaks_sort_order_ties_by_created_at(home, ana, client_for):
    project = home["project"]
    # Created in one order, but the earlier created_at must come first.
    later = make_list(project, "Created first, sorts later", sort_order=9, created_at=at(2026, 3, 2))
    earlier = make_list(project, "Created second, sorts first", sort_order=9, created_at=at(2026, 3, 1))
    rows = get_lists(client_for(ana), project)
    tied = [list_id for list_id in ids(rows) if list_id in (later.id, earlier.id)]
    assert tied == [earlier.id, later.id]


def test_get_without_project_returns_every_list_of_the_user(home, work, ana, client_for):
    shipping = make_list(work, "Shipping")
    rows = get_lists(client_for(ana))
    assert sorted(ids(rows)) == sorted([home["errands"].id, home["reading"].id, shipping.id])


def test_get_without_project_never_returns_other_users_lists(home, bobs_list, bob, client_for):
    rows = get_lists(client_for(bob))
    assert ids(rows) == [bobs_list.id]


def test_get_returns_the_whole_list_without_pagination(home, ana, client_for):
    for n in range(60):
        make_list(home["project"], f"Extra {n}", sort_order=10 + n)
    rows = get_lists(client_for(ana), home["project"])
    assert isinstance(rows, list)
    assert len(rows) == 62


def test_get_rows_have_exactly_the_contract_fields(home, ana, client_for):
    rows = get_lists(client_for(ana), home["project"])
    for row in rows:
        assert set(row) == LIST_FIELDS
        assert isinstance(row["id"], int)
    row = row_for(rows, home["errands"].id)
    assert row["project"] == home["project"].id
    assert row["name"] == "Errands"
    assert row["sort_order"] == 0


def test_list_times_are_iso_8601_utc(home, ana, client_for):
    row = row_for(get_lists(client_for(ana), home["project"]), home["errands"].id)
    parse_iso_utc(row["created_at"])
    parse_iso_utc(row["updated_at"])


def test_get_requires_a_signed_in_user(home):
    resp = APIClient().get("/api/lists", {"project": home["project"].id})
    assert resp.status_code == 401


def test_bob_gets_404_listing_the_lists_of_one_of_anas_projects(home, bob, client_for):
    resp = client_for(bob).get("/api/lists", {"project": home["project"].id})
    assert resp.status_code == 404


# ---- counts (task_count, done_count) ----------------------------------------------------------------


def test_counts_are_zero_for_a_list_without_tasks(home, ana, client_for):
    row = row_for(get_lists(client_for(ana), home["project"]), home["errands"].id)
    assert row["task_count"] == 0
    assert row["done_count"] == 0


def test_task_count_includes_top_level_and_nested_live_tasks(home, ana, client_for):
    errands = home["errands"]
    parent = make_task(errands, "Parent")
    child = make_task(errands, "Child", parent=parent)
    make_task(errands, "Grandchild", parent=child)
    make_task(errands, "Sibling")
    row = row_for(get_lists(client_for(ana), home["project"]), errands.id)
    assert row["task_count"] == 4


def test_done_count_includes_done_top_level_and_nested_live_tasks(home, ana, client_for):
    errands = home["errands"]
    parent = make_task(errands, "Parent", done=True)
    child = make_task(errands, "Child", parent=parent, done=True)
    make_task(errands, "Grandchild", parent=child, done=False)
    make_task(errands, "Open top-level")
    row = row_for(get_lists(client_for(ana), home["project"]), errands.id)
    assert row["task_count"] == 4
    assert row["done_count"] == 2


def test_counts_exclude_trashed_top_level_tasks(home, ana, client_for):
    errands = home["errands"]
    make_task(errands, "Live")
    make_task(errands, "Trashed", trashed=True)
    make_task(errands, "Trashed and done", done=True, trashed=True)
    row = row_for(get_lists(client_for(ana), home["project"]), errands.id)
    assert row["task_count"] == 1
    assert row["done_count"] == 0


def test_counts_exclude_trashed_nested_tasks_under_a_live_parent(home, ana, client_for):
    errands = home["errands"]
    parent = make_task(errands, "Live parent")
    make_task(errands, "Trashed child", parent=parent, trashed=True)
    make_task(errands, "Trashed done child", parent=parent, done=True, trashed=True)
    row = row_for(get_lists(client_for(ana), home["project"]), errands.id)
    assert row["task_count"] == 1
    assert row["done_count"] == 0


def test_counts_only_cover_their_own_list(home, ana, client_for):
    make_task(home["reading"], "Book", done=True)
    make_task(home["errands"], "Milk")
    rows = get_lists(client_for(ana), home["project"])
    assert row_for(rows, home["errands"].id)["task_count"] == 1
    assert row_for(rows, home["errands"].id)["done_count"] == 0
    assert row_for(rows, home["reading"].id)["task_count"] == 1
    assert row_for(rows, home["reading"].id)["done_count"] == 1


# ---- POST lists ------------------------------------------------------------------------------------


def test_post_creates_the_list_in_its_project_and_answers_201(home, ana, client_for):
    resp = client_for(ana).post("/api/lists", {"project": home["project"].id, "name": "Chores"}, format="json")
    assert resp.status_code == 201
    body = resp.json()
    assert set(body) == LIST_FIELDS
    assert body["project"] == home["project"].id
    assert body["name"] == "Chores"
    assert body["task_count"] == 0
    assert body["done_count"] == 0
    assert TaskList.objects.filter(pk=body["id"], project=home["project"], name="Chores").exists()


def test_post_puts_the_new_list_at_the_end_of_its_project(home, ana, client_for):
    TaskList.objects.filter(pk=home["reading"].pk).update(sort_order=7)
    resp = client_for(ana).post("/api/lists", {"project": home["project"].id, "name": "Chores"}, format="json")
    assert resp.status_code == 201
    rows = get_lists(client_for(ana), home["project"])
    assert ids(rows) == [home["errands"].id, home["reading"].id, resp.json()["id"]]


def test_post_with_an_empty_name_is_400_and_creates_nothing(home, ana, client_for):
    before = TaskList.objects.filter(project=home["project"]).count()
    resp = client_for(ana).post("/api/lists", {"project": home["project"].id, "name": ""}, format="json")
    assert resp.status_code == 400
    assert "name" in resp.json()
    assert TaskList.objects.filter(project=home["project"]).count() == before


def test_post_into_another_users_project_is_404(ana, client_for, bobs_list):
    resp = client_for(ana).post("/api/lists", {"project": bobs_list.project.id, "name": "Sneaky"}, format="json")
    assert resp.status_code == 404
    assert not TaskList.objects.filter(name="Sneaky").exists()


def test_bob_gets_404_creating_a_list_in_anas_project(home, bob, client_for):
    resp = client_for(bob).post("/api/lists", {"project": home["project"].id, "name": "Sneaky"}, format="json")
    assert resp.status_code == 404
    assert not TaskList.objects.filter(name="Sneaky").exists()


def test_post_requires_a_signed_in_user(home):
    resp = APIClient().post("/api/lists", {"project": home["project"].id, "name": "Chores"}, format="json")
    assert resp.status_code == 401


# ---- PATCH lists ------------------------------------------------------------------------------------


def test_patch_renames_the_list_and_answers_200(home, ana, client_for):
    errands = home["errands"]
    resp = client_for(ana).patch(f"/api/lists/{errands.id}", {"name": "Shopping"}, format="json")
    assert resp.status_code == 200
    assert resp.json()["name"] == "Shopping"
    errands.refresh_from_db()
    assert errands.name == "Shopping"


def test_patch_without_name_or_project_keeps_them(home, ana, client_for):
    errands = home["errands"]
    resp = client_for(ana).patch(f"/api/lists/{errands.id}", {"sort_order": 3}, format="json")
    assert resp.status_code == 200
    body = resp.json()
    assert body["name"] == "Errands"
    assert body["project"] == home["project"].id
    assert body["sort_order"] == 3


def test_patch_sort_order_reorders_the_lists_of_the_project(home, ana, client_for):
    resp = client_for(ana).patch(f"/api/lists/{home['errands'].id}", {"sort_order": 10}, format="json")
    assert resp.status_code == 200
    rows = get_lists(client_for(ana), home["project"])
    assert ids(rows) == [home["reading"].id, home["errands"].id]


def test_patch_project_moves_the_list_to_another_of_the_users_projects(home, work, ana, client_for):
    errands = home["errands"]
    resp = client_for(ana).patch(f"/api/lists/{errands.id}", {"project": work.id}, format="json")
    assert resp.status_code == 200
    assert resp.json()["project"] == work.id
    assert errands.id in ids(get_lists(client_for(ana), work))
    assert errands.id not in ids(get_lists(client_for(ana), home["project"]))


def test_moving_a_list_keeps_its_tasks_and_counts(home, work, ana, client_for):
    errands = home["errands"]
    task = make_task(errands, "Milk", done=True)
    resp = client_for(ana).patch(f"/api/lists/{errands.id}", {"project": work.id}, format="json")
    assert resp.status_code == 200
    assert resp.json()["task_count"] == 1
    assert resp.json()["done_count"] == 1
    assert Task.objects.filter(pk=task.pk, list=errands).exists()


def test_patch_with_an_empty_name_is_400(home, ana, client_for):
    errands = home["errands"]
    resp = client_for(ana).patch(f"/api/lists/{errands.id}", {"name": ""}, format="json")
    assert resp.status_code == 400
    assert "name" in resp.json()
    errands.refresh_from_db()
    assert errands.name == "Errands"


def test_patch_moving_into_another_users_project_is_404_and_the_list_stays(home, bobs_list, ana, client_for):
    errands = home["errands"]
    resp = client_for(ana).patch(f"/api/lists/{errands.id}", {"project": bobs_list.project.id}, format="json")
    assert resp.status_code == 404
    errands.refresh_from_db()
    assert errands.project_id == home["project"].id


def test_bob_gets_404_renaming_one_of_anas_lists(home, bob, client_for):
    errands = home["errands"]
    resp = client_for(bob).patch(f"/api/lists/{errands.id}", {"name": "Hijacked"}, format="json")
    assert resp.status_code == 404
    errands.refresh_from_db()
    assert errands.name == "Errands"


def test_bob_gets_404_moving_his_own_list_into_anas_project(home, bobs_list, bob, client_for):
    resp = client_for(bob).patch(f"/api/lists/{bobs_list.id}", {"project": home["project"].id}, format="json")
    assert resp.status_code == 404
    bobs_list.refresh_from_db()
    assert bobs_list.project.owner == bob


# ---- DELETE lists -----------------------------------------------------------------------------------


def test_delete_answers_204_and_removes_the_list(home, ana, client_for):
    errands = home["errands"]
    resp = client_for(ana).delete(f"/api/lists/{errands.id}")
    assert resp.status_code == 204
    assert not TaskList.objects.filter(pk=errands.pk).exists()
    assert errands.id not in ids(get_lists(client_for(ana), home["project"]))


def test_delete_removes_the_lists_live_and_nested_tasks(home, ana, client_for):
    errands = home["errands"]
    parent = make_task(errands, "Parent")
    child = make_task(errands, "Child", parent=parent)
    resp = client_for(ana).delete(f"/api/lists/{errands.id}")
    assert resp.status_code == 204
    assert not Task.objects.filter(pk__in=[parent.id, child.id]).exists()


def test_delete_removes_the_lists_trashed_tasks_too(home, ana, client_for):
    errands = home["errands"]
    trashed = make_task(errands, "Already in the Trash", trashed=True)
    resp = client_for(ana).delete(f"/api/lists/{errands.id}")
    assert resp.status_code == 204
    assert not Task.objects.filter(pk=trashed.pk).exists()


def test_delete_removes_the_notes_of_the_lists_tasks(home, ana, client_for):
    errands = home["errands"]
    task = make_task(errands, "Milk")
    note = Note.objects.create(task=task, content="<p>2 litres</p>")
    resp = client_for(ana).delete(f"/api/lists/{errands.id}")
    assert resp.status_code == 204
    assert not Note.objects.filter(pk=note.pk).exists()


def test_delete_leaves_the_other_lists_and_their_tasks_alone(home, ana, client_for):
    errands = home["errands"]
    reading = home["reading"]
    book = make_task(reading, "Dune")
    resp = client_for(ana).delete(f"/api/lists/{errands.id}")
    assert resp.status_code == 204
    assert TaskList.objects.filter(pk=reading.pk).exists()
    assert Task.objects.filter(pk=book.pk, list=reading).exists()


def test_delete_leaves_the_project_in_place(home, ana, client_for):
    resp = client_for(ana).delete(f"/api/lists/{home['errands'].id}")
    assert resp.status_code == 204
    assert Project.objects.filter(pk=home["project"].pk).exists()


def test_bob_gets_404_deleting_one_of_anas_lists(home, bob, client_for):
    errands = home["errands"]
    resp = client_for(bob).delete(f"/api/lists/{errands.id}")
    assert resp.status_code == 404
    assert TaskList.objects.filter(pk=errands.pk).exists()


# ---- POST lists/{id}/import -------------------------------------------------------------------------


def test_import_answers_201_with_only_the_created_top_level_tasks(home, ana, client_for):
    errands = home["errands"]
    payload = {"tasks": [{"title": "Milk"}, {"title": "Bread", "children": [{"title": "Rye"}]}]}
    resp = client_for(ana).post(f"/api/lists/{errands.id}/import", payload, format="json")
    assert resp.status_code == 201
    rows = resp.json()
    assert isinstance(rows, list)
    assert titles(rows) == ["Milk", "Bread"]
    assert all("id" in row for row in rows)


def test_import_appends_the_tree_after_the_existing_tasks(home, ana, client_for):
    errands = home["errands"]
    make_task(errands, "Old")
    payload = {"tasks": [{"title": "Milk"}, {"title": "Bread"}]}
    resp = client_for(ana).post(f"/api/lists/{errands.id}/import", payload, format="json")
    assert resp.status_code == 201
    assert titles(export_of(client_for(ana), errands.id)["tasks"]) == ["Old", "Milk", "Bread"]


def test_import_builds_nested_children_at_every_depth(home, ana, client_for):
    errands = home["errands"]
    payload = {"tasks": [{"title": "Trip", "children": [
        {"title": "Pack", "children": [{"title": "Socks"}]},
        {"title": "Book"},
    ]}]}
    resp = client_for(ana).post(f"/api/lists/{errands.id}/import", payload, format="json")
    assert resp.status_code == 201
    trip = export_of(client_for(ana), errands.id)["tasks"][0]
    assert trip["title"] == "Trip"
    assert titles(trip["children"]) == ["Pack", "Book"]
    assert titles(trip["children"][0]["children"]) == ["Socks"]
    assert trip["children"][1]["children"] == []


def test_import_keeps_the_given_order_of_children(home, ana, client_for):
    errands = home["errands"]
    payload = {"tasks": [{"title": "Plan", "children": [{"title": "C"}, {"title": "A"}, {"title": "B"}]}]}
    client_for(ana).post(f"/api/lists/{errands.id}/import", payload, format="json")
    plan = export_of(client_for(ana), errands.id)["tasks"][0]
    assert titles(plan["children"]) == ["C", "A", "B"]


def test_import_keeps_notes_and_done_flags(home, ana, client_for):
    errands = home["errands"]
    payload = {"tasks": [
        {"title": "Milk", "notes": "2 litres", "done": True},
        {"title": "Bread", "children": [{"title": "Rye", "done": True}]},
    ]}
    client_for(ana).post(f"/api/lists/{errands.id}/import", payload, format="json")
    tasks = export_of(client_for(ana), errands.id)["tasks"]
    assert tasks[0]["notes"] == "2 litres"
    assert tasks[0]["done"] is True
    assert tasks[1]["done"] is False
    assert tasks[1]["children"][0]["done"] is True


def test_import_counts_its_done_tasks_in_the_list_counts(home, ana, client_for):
    errands = home["errands"]
    payload = {"tasks": [{"title": "A", "done": True, "children": [{"title": "A1", "done": True}, {"title": "A2"}]}]}
    client_for(ana).post(f"/api/lists/{errands.id}/import", payload, format="json")
    row = row_for(get_lists(client_for(ana), home["project"]), errands.id)
    assert row["task_count"] == 3
    assert row["done_count"] == 2


def test_bob_gets_404_importing_into_one_of_anas_lists_and_nothing_is_created(home, bob, client_for):
    errands = home["errands"]
    before = Task.objects.filter(list=errands).count()
    payload = {"tasks": [{"title": "Injected"}]}
    resp = client_for(bob).post(f"/api/lists/{errands.id}/import", payload, format="json")
    assert resp.status_code == 404
    assert Task.objects.filter(list=errands).count() == before


# ---- GET lists/{id}/export --------------------------------------------------------------------------


def test_export_answers_the_list_name_and_a_tasks_array(home, ana, client_for):
    body = export_of(client_for(ana), home["errands"].id)
    assert set(body) == {"name", "tasks"}
    assert body["name"] == "Errands"
    assert isinstance(body["tasks"], list)


def test_export_of_a_list_without_tasks_has_no_tasks(home, ana, client_for):
    body = export_of(client_for(ana), home["reading"].id)
    assert body == {"name": "Reading", "tasks": []}


def test_export_tasks_carry_title_notes_done_and_children(home, ana, client_for):
    make_task(home["errands"], "Milk", notes="2 litres", done=True)
    node = export_of(client_for(ana), home["errands"].id)["tasks"][0]
    assert set(node) == {"title", "notes", "done", "children"}
    assert node["title"] == "Milk"
    assert node["notes"] == "2 litres"
    assert node["done"] is True
    assert node["children"] == []


def test_export_nests_children_under_their_parent(home, ana, client_for):
    errands = home["errands"]
    parent = make_task(errands, "Parent")
    child = make_task(errands, "Child", parent=parent)
    make_task(errands, "Grandchild", parent=child)
    tasks = export_of(client_for(ana), errands.id)["tasks"]
    assert titles(tasks) == ["Parent"]
    assert titles(tasks[0]["children"]) == ["Child"]
    assert titles(tasks[0]["children"][0]["children"]) == ["Grandchild"]


def test_export_orders_siblings_by_sort_order(home, ana, client_for):
    errands = home["errands"]
    parent = make_task(errands, "Parent", sort_order=5)
    make_task(errands, "C", sort_order=2)
    make_task(errands, "A", sort_order=0)
    make_task(errands, "B", sort_order=1)
    make_task(errands, "second", parent=parent, sort_order=1)
    make_task(errands, "first", parent=parent, sort_order=0)
    tasks = export_of(client_for(ana), errands.id)["tasks"]
    assert titles(tasks) == ["A", "B", "C", "Parent"]
    assert titles(tasks[3]["children"]) == ["first", "second"]


def test_export_breaks_sort_order_ties_by_created_at(home, ana, client_for):
    errands = home["errands"]
    make_task(errands, "Created second", sort_order=4, created_at=at(2026, 5, 2))
    make_task(errands, "Created first", sort_order=4, created_at=at(2026, 5, 1))
    tasks = export_of(client_for(ana), errands.id)["tasks"]
    assert titles(tasks) == ["Created first", "Created second"]


def test_export_excludes_trashed_top_level_tasks_and_their_subtrees(home, ana, client_for):
    errands = home["errands"]
    make_task(errands, "Live")
    trashed_parent = make_task(errands, "Trashed parent", trashed=True)
    make_task(errands, "Trashed child", parent=trashed_parent, trashed=True)
    tasks = export_of(client_for(ana), errands.id)["tasks"]
    assert titles(tasks) == ["Live"]


def test_export_excludes_trashed_nested_tasks_under_a_live_parent(home, ana, client_for):
    errands = home["errands"]
    parent = make_task(errands, "Live parent")
    make_task(errands, "Trashed child", parent=parent, trashed=True)
    make_task(errands, "Live child", parent=parent)
    tasks = export_of(client_for(ana), errands.id)["tasks"]
    assert titles(tasks) == ["Live parent"]
    assert titles(tasks[0]["children"]) == ["Live child"]


def test_bob_gets_404_exporting_one_of_anas_lists(home, bob, client_for):
    resp = client_for(bob).get(f"/api/lists/{home['errands'].id}/export")
    assert resp.status_code == 404
