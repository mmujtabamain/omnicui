import pytest
from django.core.management import call_command
from django.test import override_settings
from django.urls import clear_url_caches
from rest_framework.test import APIClient

from accounts.models import User
from planner import demo
from planner.models import Attachment, Project, Task, TaskList


@pytest.fixture
def media(tmp_path, settings):
    settings.MEDIA_ROOT = tmp_path


@pytest.mark.django_db
def test_standard_dataset_matches_the_reference_screens(ana, media):
    demo.load(ana, "standard")
    assert list(Project.objects.filter(owner=ana).values_list("name", flat=True)) == ["Home", "Spring launch"]
    assert list(TaskList.objects.filter(project__owner=ana).order_by("project__sort_order", "sort_order").values_list("name", flat=True)) == ["Errands", "Reading", "Garden", "Website", "Press kit"]
    assert Task.objects.filter(list__project__owner=ana, deleted_at__isnull=False).count() == 2
    hero = Task.objects.get(list__project__owner=ana, title="Rewrite the hero section")
    assert hero.note_items.count() == 2
    assert sorted(hero.attachments.values_list("kind", flat=True)) == ["file", "link"]
    from accounts.models import UserSettings
    assert UserSettings.objects.get(user=ana).last_list.name == "Errands"


@pytest.mark.django_db
def test_loading_again_replaces_the_data_and_its_files(ana, media):
    demo.load(ana, "standard")
    old = Attachment.objects.get(kind="file").file.path
    demo.load(ana, "empty")
    assert list(TaskList.objects.filter(project__owner=ana).values_list("name", flat=True)) == ["Inbox"]
    assert not Task.objects.filter(list__project__owner=ana).exists()
    import os
    assert not os.path.exists(old)


@pytest.mark.django_db
def test_nolists_leaves_a_project_without_lists(ana, media):
    demo.load(ana, "nolists")
    assert Project.objects.filter(owner=ana).count() == 1
    assert not TaskList.objects.filter(project__owner=ana).exists()


@pytest.mark.django_db
def test_seed_demo_command_creates_the_account(media):
    call_command("seed_demo", "--dataset", "long")
    user = User.objects.get(email="demo@taskscape.test")
    assert user.check_password("taskscape-demo")
    assert Task.objects.filter(list__project__owner=user, title__startswith="Errand number").count() == 40


@pytest.mark.django_db
def test_reset_is_not_routed_unless_enabled():
    assert APIClient().post("/api/demo/reset", {"dataset": "standard"}, format="json").status_code == 404


@pytest.mark.django_db
def test_reset_signs_in_the_demo_account_when_enabled(media, settings):
    settings.DEMO_RESET = True
    import importlib
    import config.urls
    clear_url_caches()
    importlib.reload(config.urls)
    try:
        c = APIClient()
        r = c.post("/api/demo/reset", {"dataset": "empty"}, format="json")
        assert r.status_code == 200 and r.json()["lists"] == 1
        assert c.get("/api/auth/me").json()["email"] == "demo@taskscape.test"
        assert c.post("/api/demo/reset", {"dataset": "nope"}, format="json").status_code == 400
    finally:
        settings.DEMO_RESET = False
        clear_url_caches()
        importlib.reload(config.urls)
