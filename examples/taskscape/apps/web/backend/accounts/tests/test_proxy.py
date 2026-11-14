import pytest
from rest_framework.test import APIClient

from planner.models import Project

HOST = "preview.example.test"


@pytest.fixture
def behind_proxy(settings):
    settings.TRUST_PROXY = True
    settings.ALLOWED_HOSTS = [HOST]


def signed_in_write(ana, origin):
    c = APIClient(enforce_csrf_checks=True)
    c.force_login(ana)
    token = c.get("/api/auth/csrf", HTTP_HOST=HOST).json()["csrfToken"]
    return c.post("/api/projects", {"name": "Work"}, format="json", HTTP_HOST=HOST, HTTP_ORIGIN=origin, HTTP_X_CSRFTOKEN=token)


@pytest.mark.django_db
def test_a_write_from_this_https_origin_passes_behind_a_proxy(ana, behind_proxy):
    assert signed_in_write(ana, f"https://{HOST}").status_code == 201


@pytest.mark.django_db
def test_another_origin_is_still_refused(ana, behind_proxy):
    assert signed_in_write(ana, "https://elsewhere.example.test").status_code == 403
    assert not Project.objects.filter(owner=ana, name="Work").exists()


@pytest.mark.django_db
def test_without_trust_proxy_the_scheme_is_not_inferred(ana, settings):
    settings.TRUST_PROXY = False
    settings.ALLOWED_HOSTS = [HOST]
    assert signed_in_write(ana, f"https://{HOST}").status_code == 403


@pytest.mark.django_db
def test_the_public_host_may_come_as_x_forwarded_host(ana, settings):
    settings.TRUST_PROXY = True
    settings.USE_X_FORWARDED_HOST = True
    settings.ALLOWED_HOSTS = ["*"]
    c = APIClient(enforce_csrf_checks=True)
    c.force_login(ana)
    token = c.get("/api/auth/csrf").json()["csrfToken"]
    r = c.post("/api/projects", {"name": "Work"}, format="json", HTTP_HOST="10.0.0.5:8080",
               HTTP_X_FORWARDED_HOST=HOST, HTTP_ORIGIN=f"https://{HOST}", HTTP_X_CSRFTOKEN=token)
    assert r.status_code == 201


@pytest.mark.django_db
def test_a_proxy_that_forwards_its_own_http_hop_still_passes(ana, behind_proxy, settings):
    settings.SECURE_PROXY_SSL_HEADER = ("HTTP_X_FORWARDED_PROTO", "https")
    c = APIClient(enforce_csrf_checks=True)
    c.force_login(ana)
    token = c.get("/api/auth/csrf", HTTP_HOST=HOST).json()["csrfToken"]
    r = c.post("/api/projects", {"name": "Work"}, format="json", HTTP_HOST=HOST, HTTP_X_FORWARDED_PROTO="http",
               HTTP_ORIGIN=f"https://{HOST}", HTTP_X_CSRFTOKEN=token)
    assert r.status_code == 201
