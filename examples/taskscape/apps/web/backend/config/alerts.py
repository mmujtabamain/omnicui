"""Posts error log records to the team's alert webhook (ALERT_WEBHOOK_URL, ALERT_WEBHOOK_TOKEN).

Sending never blocks a request for long and never raises: a failed alert is only printed.
"""
import json
import logging
import os
import threading
import urllib.request


class AlertHandler(logging.Handler):
    def emit(self, record):
        url, token = os.environ.get("ALERT_WEBHOOK_URL"), os.environ.get("ALERT_WEBHOOK_TOKEN")
        if not url or not token:
            return
        try:
            text = self.format(record)[:3500]
        except Exception:
            text = record.getMessage()[:3500]
        body = json.dumps({"text": f"Taskscape web: {text}", "level": "error", "source": "taskscape-web"}).encode()

        def send():
            try:
                req = urllib.request.Request(url.rstrip("/") + "/api/notify/", data=body, method="POST", headers={
                    "content-type": "application/json", "authorization": f"Bearer {token}"})
                urllib.request.urlopen(req, timeout=5).read()
            except Exception as e:  # never let alerting break the app
                print(f"alert not sent: {e}")

        threading.Thread(target=send, daemon=True).start()
