import pytest
from rest_framework.test import APIClient

from accounts.models import User
from planner.models import Project


@pytest.mark.django_db
def test_signup_signs_in_and_creates_a_first_project_and_list():
    c = APIClient()
    r = c.post("/api/auth/signup", {"email": "New@Example.com", "password": "long-enough-1"}, format="json")
    assert r.status_code == 201
    assert r.json()["email"] == "new@example.com"
    assert c.get("/api/auth/me").status_code == 200
    p = Project.objects.get(owner__email="new@example.com")
    assert [l.name for l in p.lists.all()] == ["Inbox"]


@pytest.mark.django_db
def test_signup_refuses_a_taken_email_and_a_short_password(ana):
    c = APIClient()
    assert c.post("/api/auth/signup", {"email": "ANA@example.com", "password": "long-enough-1"}, format="json").status_code == 400
    assert c.post("/api/auth/signup", {"email": "x@example.com", "password": "short"}, format="json").status_code == 400


@pytest.mark.django_db
def test_signin_and_signout(ana):
    c = APIClient()
    assert c.post("/api/auth/signin", {"email": "ana@example.com", "password": "wrong"}, format="json").status_code == 400
    assert c.post("/api/auth/signin", {"email": "ana@example.com", "password": "correct-horse"}, format="json").status_code == 200
    assert c.get("/api/auth/me").json()["email"] == "ana@example.com"
    assert c.post("/api/auth/signout").status_code == 204
    assert c.get("/api/auth/me").status_code == 401


@pytest.mark.django_db
def test_everything_else_needs_a_session():
    c = APIClient()
    assert c.get("/api/settings").status_code == 401


@pytest.mark.django_db
def test_settings_round_trip_and_refuse_someone_elses_project(ana, bob, client_for, home):
    c = client_for(ana)
    r = c.patch("/api/settings", {"theme": "dark", "reduced_motion": "on", "show_completed": False}, format="json")
    assert r.status_code == 200 and r.json()["theme"] == "dark" and r.json()["reduced_motion"] == "on"
    assert c.patch("/api/settings", {"theme": "purple"}, format="json").status_code == 400
    assert client_for(bob).patch("/api/settings", {"last_project": home["project"].id}, format="json").status_code == 400


@pytest.mark.django_db
def test_shortcuts_set_conflict_unbind_and_reset(ana, client_for):
    c = client_for(ana)
    assert c.put("/api/shortcuts", {"command": "new_task", "accel": "Alt+N"}, format="json").status_code == 200
    assert c.put("/api/shortcuts", {"command": "new_list", "accel": "Alt+N"}, format="json").status_code == 409
    rows = c.put("/api/shortcuts", {"command": "new_list", "accel": ""}, format="json").json()
    assert {"command": "new_list", "accel": ""} in rows and {"command": "new_task", "accel": "Alt+N"} in rows
    r = c.put("/api/shortcuts", {"command": "new_task", "accel": None}, format="json")
    assert {"command": "new_task", "accel": "Alt+N"} not in r.json()
