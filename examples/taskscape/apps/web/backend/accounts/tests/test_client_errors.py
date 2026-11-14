import logging

import pytest
from rest_framework.test import APIClient


@pytest.fixture(autouse=True)
def fresh_throttle():
    from django.core.cache import cache
    cache.clear()


@pytest.mark.django_db
def test_a_browser_error_is_logged_at_error_level(caplog):
    with caplog.at_level(logging.ERROR, logger="taskscape.client"):
        r = APIClient(enforce_csrf_checks=True).post("/api/client-errors", {"message": "TypeError: x is undefined", "url": "/"}, format="json")
    assert r.status_code == 204
    assert any("TypeError: x is undefined" in m for m in caplog.messages)


@pytest.mark.django_db
def test_an_empty_message_is_refused():
    assert APIClient().post("/api/client-errors", {}, format="json").status_code == 400


@pytest.mark.django_db
def test_one_client_is_rate_limited():
    c = APIClient()
    codes = [c.post("/api/client-errors", {"message": f"e{i}"}, format="json").status_code for i in range(21)]
    assert codes[:20] == [204] * 20 and codes[20] == 429
