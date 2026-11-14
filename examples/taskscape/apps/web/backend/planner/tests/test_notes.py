"""Contract tests for the notes resource (API.md, section "Notes: `notes`").

Written from the contract alone. Setup rows are created with the ORM; every request under
test goes through the API as the user named in the test.
"""
import logging
import re
from datetime import datetime, timedelta

import pytest
from django.utils import timezone
from rest_framework.test import APIClient

from planner.models import Note, Project, Task, TaskList

NOTES = "/api/notes"
NOTE_FIELDS = {"id", "task", "content", "sort_order", "created_at", "updated_at"}

ALLOWED_TAGS = [
    "p", "br", "strong", "b", "em", "i", "u", "s", "ul", "ol", "li", "a",
    "code", "pre", "blockquote", "h1", "h2", "h3",
]
DISALLOWED_PAIRED_TAGS = ["div", "span", "table", "iframe", "h4", "font", "section", "video"]
DISALLOWED_VOID_TAGS = ["img", "hr"]


def has_tag(html, tag):
    return re.search(rf"<{tag}[\s>/]", html) is not None


def sample_markup(tag):
    if tag == "br":
        return "x<br>y"
    if tag == "a":
        return '<a href="https://example.com/">x</a>'
    return f"<{tag}>x</{tag}>"


def saved_content(client, task, html):
    """POST a note with this HTML and return the content the API stored and answered."""
    r = client.post(NOTES, {"task": task.id, "content": html}, format="json")
    assert r.status_code == 201, r.content
    return r.json()["content"]


def write_lines(caplog):
    """INFO lines from the log that name the notes resource, split into words."""
    messages = [rec.getMessage() for rec in caplog.records if rec.levelno == logging.INFO]
    return [m.split() for m in messages if m.split()[:1] in (["notes"], ["note"])]


def assert_one_write_line(caplog, note_id, user_id):
    lines = write_lines(caplog)
    assert len(lines) == 1, [rec.getMessage() for rec in caplog.records]
    assert len(lines[0]) == 4, lines[0]  # resource action id user
    assert lines[0][2] == str(note_id)
    assert lines[0][3] == str(user_id)


@pytest.fixture
def milk(home):
    return Task.objects.create(list=home["errands"], title="Buy milk")


@pytest.fixture
def bread(home):
    return Task.objects.create(list=home["errands"], title="Buy bread")


@pytest.fixture
def bobs_task(bob):
    p = Project.objects.create(owner=bob, name="Bob's")
    lst = TaskList.objects.create(project=p, name="Bob's list")
    return Task.objects.create(list=lst, title="Bob's task")


@pytest.fixture
def ana_note(milk):
    return Note.objects.create(task=milk, content="<p>first</p>", sort_order=0)


# GET notes?task={id}

def test_get_answers_200_with_a_list_of_the_note_objects(ana, milk, ana_note, client_for):
    r = client_for(ana).get(NOTES, {"task": milk.id})
    assert r.status_code == 200
    body = r.json()
    assert isinstance(body, list)
    assert len(body) == 1
    assert set(body[0]) == NOTE_FIELDS
    assert body[0]["id"] == ana_note.id
    assert body[0]["task"] == milk.id
    assert body[0]["content"] == "<p>first</p>"
    assert body[0]["sort_order"] == 0


def test_get_on_a_task_without_notes_answers_an_empty_list(ana, milk, client_for):
    r = client_for(ana).get(NOTES, {"task": milk.id})
    assert r.status_code == 200
    assert r.json() == []


def test_get_returns_only_the_notes_of_the_asked_task(ana, milk, bread, client_for):
    mine_1 = Note.objects.create(task=milk, content="<p>a</p>", sort_order=0)
    mine_2 = Note.objects.create(task=milk, content="<p>b</p>", sort_order=1)
    theirs = Note.objects.create(task=bread, content="<p>c</p>", sort_order=0)
    r = client_for(ana).get(NOTES, {"task": milk.id})
    ids = [n["id"] for n in r.json()]
    assert ids == [mine_1.id, mine_2.id]
    assert theirs.id not in ids


def test_get_returns_every_note_of_the_task_without_pagination(ana, milk, client_for):
    for i in range(30):
        Note.objects.create(task=milk, content=f"<p>{i}</p>", sort_order=i)
    r = client_for(ana).get(NOTES, {"task": milk.id})
    assert r.status_code == 200
    assert len(r.json()) == 30


def test_get_orders_notes_by_sort_order_before_creation_order(ana, milk, client_for):
    first = Note.objects.create(task=milk, content="<p>a</p>", sort_order=0)
    second = Note.objects.create(task=milk, content="<p>b</p>", sort_order=1)
    c = client_for(ana)
    assert c.patch(f"{NOTES}/{second.id}", {"sort_order": -5}, format="json").status_code == 200
    ids = [n["id"] for n in c.get(NOTES, {"task": milk.id}).json()]
    assert ids == [second.id, first.id]


def test_equal_sort_order_falls_back_to_creation_order(ana, milk, client_for):
    older = Note.objects.create(task=milk, content="<p>a</p>", sort_order=3)
    newer = Note.objects.create(task=milk, content="<p>b</p>", sort_order=3)
    ids = [n["id"] for n in client_for(ana).get(NOTES, {"task": milk.id}).json()]
    assert ids == [older.id, newer.id]


def test_get_of_another_users_task_is_404_for_bob(bob, milk, ana_note, client_for):
    r = client_for(bob).get(NOTES, {"task": milk.id})
    assert r.status_code == 404


def test_anonymous_get_is_refused(ana, milk, ana_note):
    r = APIClient().get(NOTES, {"task": milk.id})
    assert r.status_code in (401, 403)


# POST notes {task, content}

def test_post_creates_the_note_and_answers_201_with_its_fields(ana, milk, client_for):
    html = "<p>Hello <strong>world</strong></p>"
    r = client_for(ana).post(NOTES, {"task": milk.id, "content": html}, format="json")
    assert r.status_code == 201
    body = r.json()
    assert set(body) == NOTE_FIELDS
    assert isinstance(body["id"], int)
    assert body["task"] == milk.id
    assert body["content"] == html
    assert isinstance(body["sort_order"], int)
    assert Note.objects.filter(pk=body["id"], task=milk).exists()


def test_post_appends_each_new_note_after_the_existing_ones(ana, milk, client_for):
    c = client_for(ana)
    ids = [
        c.post(NOTES, {"task": milk.id, "content": f"<p>{i}</p>"}, format="json").json()["id"]
        for i in range(3)
    ]
    got = [n["id"] for n in c.get(NOTES, {"task": milk.id}).json()]
    assert got == ids


def test_post_goes_after_a_note_moved_to_a_high_sort_order(ana, milk, ana_note, client_for):
    c = client_for(ana)
    assert c.patch(f"{NOTES}/{ana_note.id}", {"sort_order": 10}, format="json").status_code == 200
    new_id = c.post(NOTES, {"task": milk.id, "content": "<p>new</p>"}, format="json").json()["id"]
    got = [n["id"] for n in c.get(NOTES, {"task": milk.id}).json()]
    assert got == [ana_note.id, new_id]


def test_post_without_task_is_400_naming_task(ana, client_for):
    r = client_for(ana).post(NOTES, {"content": "<p>x</p>"}, format="json")
    assert r.status_code == 400
    assert "task" in r.json()


def test_post_with_a_non_integer_task_is_400_naming_task(ana, client_for):
    r = client_for(ana).post(NOTES, {"task": "abc", "content": "<p>x</p>"}, format="json")
    assert r.status_code == 400
    assert "task" in r.json()


def test_post_to_another_users_task_is_404_for_bob_and_creates_nothing(bob, milk, client_for):
    before = Note.objects.filter(task=milk).count()
    r = client_for(bob).post(NOTES, {"task": milk.id, "content": "<p>sneaky</p>"}, format="json")
    assert r.status_code == 404
    assert Note.objects.filter(task=milk).count() == before


def test_post_to_own_task_succeeds_for_bob(bob, bobs_task, client_for):
    r = client_for(bob).post(NOTES, {"task": bobs_task.id, "content": "<p>mine</p>"}, format="json")
    assert r.status_code == 201
    assert r.json()["task"] == bobs_task.id


def test_created_and_updated_times_are_iso8601_utc(ana, milk, client_for):
    r = client_for(ana).post(NOTES, {"task": milk.id, "content": "<p>t</p>"}, format="json")
    assert r.status_code == 201
    for key in ("created_at", "updated_at"):
        parsed = datetime.fromisoformat(r.json()[key].replace("Z", "+00:00"))
        assert parsed.utcoffset() == timedelta(0), key


# PATCH notes/{id} {content?, sort_order?}

def test_patch_content_answers_200_with_the_new_content(ana, ana_note, client_for):
    r = client_for(ana).patch(f"{NOTES}/{ana_note.id}", {"content": "<p>changed</p>"}, format="json")
    assert r.status_code == 200
    assert r.json()["id"] == ana_note.id
    assert r.json()["content"] == "<p>changed</p>"
    ana_note.refresh_from_db()
    assert ana_note.content == "<p>changed</p>"


def test_patch_sort_order_answers_200_with_the_new_sort_order(ana, ana_note, client_for):
    r = client_for(ana).patch(f"{NOTES}/{ana_note.id}", {"sort_order": 4}, format="json")
    assert r.status_code == 200
    assert r.json()["sort_order"] == 4


def test_patch_with_only_content_leaves_sort_order_unchanged(ana, ana_note, client_for):
    ana_note.sort_order = 7
    ana_note.save()
    r = client_for(ana).patch(f"{NOTES}/{ana_note.id}", {"content": "<p>new</p>"}, format="json")
    assert r.status_code == 200
    assert r.json()["sort_order"] == 7


def test_patch_with_only_sort_order_leaves_content_unchanged(ana, ana_note, client_for):
    r = client_for(ana).patch(f"{NOTES}/{ana_note.id}", {"sort_order": 3}, format="json")
    assert r.status_code == 200
    assert r.json()["content"] == "<p>first</p>"


def test_patch_with_a_non_integer_sort_order_is_400_naming_sort_order(ana, ana_note, client_for):
    r = client_for(ana).patch(f"{NOTES}/{ana_note.id}", {"sort_order": "first"}, format="json")
    assert r.status_code == 400
    assert "sort_order" in r.json()


def test_patch_of_another_users_note_is_404_and_changes_nothing(bob, ana_note, client_for):
    r = client_for(bob).patch(f"{NOTES}/{ana_note.id}", {"content": "<p>hacked</p>"}, format="json")
    assert r.status_code == 404
    ana_note.refresh_from_db()
    assert ana_note.content == "<p>first</p>"


# DELETE notes/{id}

def test_delete_answers_204_and_removes_the_note(ana, milk, ana_note, client_for):
    c = client_for(ana)
    r = c.delete(f"{NOTES}/{ana_note.id}")
    assert r.status_code == 204
    assert r.content == b""
    assert not Note.objects.filter(pk=ana_note.pk).exists()
    assert c.get(NOTES, {"task": milk.id}).json() == []


def test_delete_leaves_the_other_notes_of_the_task_in_place(ana, milk, ana_note, client_for):
    other = Note.objects.create(task=milk, content="<p>second</p>", sort_order=1)
    client_for(ana).delete(f"{NOTES}/{ana_note.id}")
    ids = [n["id"] for n in client_for(ana).get(NOTES, {"task": milk.id}).json()]
    assert ids == [other.id]


def test_delete_of_another_users_note_is_404_and_keeps_the_note(bob, ana_note, client_for):
    r = client_for(bob).delete(f"{NOTES}/{ana_note.id}")
    assert r.status_code == 404
    assert Note.objects.filter(pk=ana_note.pk).exists()


# Sanitization of content on save

@pytest.mark.parametrize("tag", ALLOWED_TAGS)
def test_allowed_tag_is_kept(ana, milk, client_for, tag):
    content = saved_content(client_for(ana), milk, sample_markup(tag))
    assert has_tag(content, tag), content


@pytest.mark.parametrize("tag", DISALLOWED_PAIRED_TAGS)
def test_disallowed_tag_is_dropped_and_its_text_kept(ana, milk, client_for, tag):
    content = saved_content(client_for(ana), milk, f"<p>before</p><{tag}>keep-me</{tag}>")
    assert not has_tag(content, tag), content
    assert "keep-me" in content


@pytest.mark.parametrize("tag", DISALLOWED_VOID_TAGS)
def test_disallowed_void_tag_is_dropped_and_its_attributes_with_it(ana, milk, client_for, tag):
    content = saved_content(client_for(ana), milk, f'<p>keep-me</p><{tag} src="x.png">')
    assert not has_tag(content, tag), content
    assert "x.png" not in content
    assert "keep-me" in content


def test_script_element_is_removed(ana, milk, client_for):
    content = saved_content(client_for(ana), milk, "<p>ok</p><script>alert(1)</script>")
    assert "<script" not in content.lower()
    assert "ok" in content


def test_style_element_is_removed(ana, milk, client_for):
    content = saved_content(client_for(ana), milk, "<style>p{color:red}</style><p>ok</p>")
    assert "<style" not in content.lower()
    assert "ok" in content


def test_event_handler_attribute_on_a_disallowed_tag_is_dropped(ana, milk, client_for):
    content = saved_content(client_for(ana), milk, '<img src="x.png" onerror="alert(1)">')
    assert "onerror" not in content.lower()
    assert "alert" not in content


def test_attributes_are_dropped_from_allowed_tags(ana, milk, client_for):
    content = saved_content(client_for(ana), milk, '<p class="x" style="color:red" onclick="evil()">hi</p>')
    for attr in ("class=", "style=", "onclick="):
        assert attr not in content, content
    assert has_tag(content, "p")
    assert "hi" in content


@pytest.mark.parametrize("href", [
    "https://example.com/a?b=1",
    "http://example.com/",
    "mailto:ana@example.com",
])
def test_link_keeps_an_http_https_or_mailto_href(ana, milk, client_for, href):
    content = saved_content(client_for(ana), milk, f'<a href="{href}">link</a>')
    assert f'href="{href}"' in content, content


@pytest.mark.parametrize("href", [
    "javascript:alert(1)",
    "JaVaScRiPt:alert(1)",
    "ftp://files.example.com/a",
    "/relative/path",
    "data:text/html,hi",
])
def test_link_drops_any_other_href_and_keeps_its_text(ana, milk, client_for, href):
    content = saved_content(client_for(ana), milk, f'<a href="{href}">link</a>')
    assert "href" not in content.lower(), content
    assert "javascript" not in content.lower()
    assert "link" in content


def test_link_drops_the_target_attribute_but_keeps_the_href(ana, milk, client_for):
    content = saved_content(client_for(ana), milk, '<a href="https://example.com/" target="_blank">x</a>')
    assert "target" not in content
    assert 'href="https://example.com/"' in content


def test_patch_sanitizes_content_on_save(ana, ana_note, client_for):
    r = client_for(ana).patch(
        f"{NOTES}/{ana_note.id}", {"content": "<p>ok</p><script>alert(1)</script>"}, format="json",
    )
    assert r.status_code == 200
    assert "<script" not in r.json()["content"].lower()
    ana_note.refresh_from_db()
    assert "<script" not in ana_note.content.lower()
    assert "ok" in ana_note.content


# Cascades

def test_deleting_a_project_deletes_its_notes(ana, home, milk, client_for):
    Project.objects.create(owner=ana, name="Spare")  # so the delete is not of the last project
    note = Note.objects.create(task=milk, content="<p>x</p>", sort_order=0)
    r = client_for(ana).delete(f"/api/projects/{home['project'].id}")
    assert r.status_code == 204
    assert not Note.objects.filter(pk=note.pk).exists()


def test_deleting_a_list_deletes_the_notes_of_its_tasks(ana, home, milk, client_for):
    reading_task = Task.objects.create(list=home["reading"], title="Read")
    gone = Note.objects.create(task=milk, content="<p>gone</p>", sort_order=0)
    kept = Note.objects.create(task=reading_task, content="<p>kept</p>", sort_order=0)
    r = client_for(ana).delete(f"/api/lists/{home['errands'].id}")
    assert r.status_code == 204
    assert not Note.objects.filter(pk=gone.pk).exists()
    assert Note.objects.filter(pk=kept.pk).exists()


def test_purging_a_trashed_task_deletes_its_notes(ana, home, milk, client_for):
    milk.deleted_at = timezone.now()
    milk.save()
    note = Note.objects.create(task=milk, content="<p>x</p>", sort_order=0)
    r = client_for(ana).post("/api/trash/purge", {"ids": [milk.id]}, format="json")
    assert r.status_code == 204
    assert not Note.objects.filter(pk=note.pk).exists()


def test_purging_a_trashed_task_deletes_the_notes_of_its_trashed_subtree(ana, home, milk, client_for):
    now = timezone.now()
    milk.deleted_at = now
    milk.save()
    child = Task.objects.create(list=home["errands"], parent=milk, title="Oat milk", deleted_at=now)
    child_note = Note.objects.create(task=child, content="<p>child</p>", sort_order=0)
    r = client_for(ana).post("/api/trash/purge", {"ids": [milk.id]}, format="json")
    assert r.status_code == 204
    assert not Note.objects.filter(pk=child_note.pk).exists()


def test_emptying_the_trash_deletes_notes_of_trashed_tasks_but_not_of_live_ones(ana, home, milk, client_for):
    milk.deleted_at = timezone.now()
    milk.save()
    live_task = Task.objects.create(list=home["reading"], title="Read")
    gone = Note.objects.create(task=milk, content="<p>gone</p>", sort_order=0)
    kept = Note.objects.create(task=live_task, content="<p>kept</p>", sort_order=0)
    r = client_for(ana).post("/api/trash/empty")
    assert r.status_code == 204
    assert not Note.objects.filter(pk=gone.pk).exists()
    assert Note.objects.filter(pk=kept.pk).exists()


# Logging: every write logs one INFO line (resource action id user)

def test_create_writes_one_info_line_naming_the_note_and_user(ana, milk, client_for, caplog):
    caplog.set_level(logging.INFO)
    r = client_for(ana).post(NOTES, {"task": milk.id, "content": "<p>log</p>"}, format="json")
    assert r.status_code == 201
    assert_one_write_line(caplog, r.json()["id"], ana.id)


def test_update_writes_one_info_line_naming_the_note_and_user(ana, ana_note, client_for, caplog):
    caplog.set_level(logging.INFO)
    r = client_for(ana).patch(f"{NOTES}/{ana_note.id}", {"content": "<p>log</p>"}, format="json")
    assert r.status_code == 200
    assert_one_write_line(caplog, ana_note.id, ana.id)


def test_delete_writes_one_info_line_naming_the_note_and_user(ana, ana_note, client_for, caplog):
    caplog.set_level(logging.INFO)
    r = client_for(ana).delete(f"{NOTES}/{ana_note.id}")
    assert r.status_code == 204
    assert_one_write_line(caplog, ana_note.id, ana.id)
