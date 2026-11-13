"""Posts review outcomes to the flowchart page as replies, and resolves the settled ones.
usage: post_review.py <page url> <outcomes.json>..."""
import json, sys, urllib.request

base, files = sys.argv[1].rstrip('/'), sys.argv[2:]
live = {c["id"]: c for c in json.load(urllib.request.urlopen(f"{base}/api/comments"))}
replied = {c["parent"] for c in live.values() if c["parent"] and c["author"] == "Haiku review"}
LABEL = {"answered": "Answered", "decided": "Decided", "invalid": "Not a problem", "needs_mujtaba": "Needs Mujtaba"}


def call(method, path, body):
    req = urllib.request.Request(f"{base}{path}", data=json.dumps(body).encode(), method=method, headers={"content-type": "application/json"})
    return json.load(urllib.request.urlopen(req))


done = 0
for f in files:
    for o in json.load(open(f)):
        c = live.get(o["id"])
        if not c or o["id"] in replied:
            continue
        text = f"{LABEL[o['outcome']]}. {o['reply']}"
        if o["outcome"] == "needs_mujtaba" and o.get("question"):
            text += f"\n\nQuestion: {o['question']}"
        text += f"\n\nEvidence: {o['evidence']}"
        call("POST", "/api/comments", {"node": c["node"], "parent": c["id"], "author": "Haiku review", "tag": c["tag"], "text": text[:5000]})
        if o["outcome"] != "needs_mujtaba":
            call("PATCH", f"/api/comments/{c['id']}", {"resolved": True})
        done += 1
print(done, "comments answered")
