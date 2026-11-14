import importlib

import pytest
from django.core.files.uploadedfile import SimpleUploadedFile
from django.urls import clear_url_caches

from planner.models import Attachment, Task


@pytest.fixture
def stored(ana, home, tmp_path, settings):
    settings.MEDIA_ROOT = tmp_path
    task = Task.objects.create(list=home["errands"], title="Scan receipts")
    a = Attachment(task=task, kind="file", name="receipt.txt", size=5, mime="text/plain")
    a.file.save("receipt.txt", SimpleUploadedFile("receipt.txt", b"hello"), save=True)
    return a


@pytest.mark.django_db
def test_the_owner_downloads_the_file(client, ana, stored):
    client.force_login(ana)
    r = client.get(stored.file.url)
    assert r.status_code == 200
    assert b"".join(r.streaming_content) == b"hello"
    assert r["Content-Type"] == "text/plain"


@pytest.mark.django_db
def test_another_user_gets_404(client, bob, stored):
    client.force_login(bob)
    assert client.get(stored.file.url).status_code == 404


@pytest.mark.django_db
def test_a_visitor_gets_404(stored):
    from rest_framework.test import APIClient
    assert APIClient().get(stored.file.url).status_code == 404


@pytest.fixture
def frontend(tmp_path, settings):
    (tmp_path / "assets").mkdir()
    (tmp_path / "index.html").write_text("<!doctype html><div id=root></div>")
    (tmp_path / "assets" / "app.js").write_text("console.log(1)")
    settings.FRONTEND_DIST = str(tmp_path)
    import config.urls
    clear_url_caches()
    importlib.reload(config.urls)
    yield
    settings.FRONTEND_DIST = ""
    clear_url_caches()
    importlib.reload(config.urls)


@pytest.mark.django_db
def test_the_built_app_is_served_for_its_own_routes(client, frontend):
    for path in ("/", "/signin", "/lists/3"):
        r = client.get(path)
        assert r.status_code == 200 and b"id=root" in b"".join(r.streaming_content)
    asset = client.get("/assets/app.js")
    assert asset.status_code == 200
    assert "immutable" in asset["Cache-Control"]  # hashed names: a reload loads the app from the cache
    assert client.get("/api/auth/me").status_code == 401
