import pytest
from rest_framework.test import APIClient

from accounts.models import User, UserSettings
from planner.models import Project, TaskList


@pytest.fixture
def make_user(db):
    def make(email="ana@example.com", password="correct-horse"):
        u = User.objects.create_user(username=email, email=email, password=password)
        UserSettings.objects.create(user=u)
        return u
    return make


@pytest.fixture
def client_for():
    """An API client signed in as the given user (CSRF checks off, as DRF's test client does)."""
    def make(user):
        c = APIClient()
        c.force_authenticate(user)
        return c
    return make


@pytest.fixture
def ana(make_user):
    return make_user()


@pytest.fixture
def bob(make_user):
    return make_user("bob@example.com")


@pytest.fixture
def home(ana):
    """Ana's project with two lists."""
    p = Project.objects.create(owner=ana, name="Home")
    return {"project": p, "errands": TaskList.objects.create(project=p, name="Errands", sort_order=0),
            "reading": TaskList.objects.create(project=p, name="Reading", sort_order=1)}
