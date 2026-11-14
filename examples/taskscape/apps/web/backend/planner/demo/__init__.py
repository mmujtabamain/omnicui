"""Demo data: the projects, lists and tasks the reference screenshots show, for previews, screen
tests and recordings. load(user, "standard" | "empty" | "nolists" | "long") replaces the user's data."""
from datetime import timedelta
from pathlib import Path

from django.core.files import File
from django.db import transaction
from django.utils import timezone

from accounts.models import UserSettings
from planner.models import Attachment, Note, Project, Task, TaskList

HERE = Path(__file__).parent
MIN, DAY = timedelta(minutes=1), timedelta(days=1)

STANDARD = {
    "projects": [("Home", 30 * DAY), ("Spring launch", 20 * DAY)],
    "lists": [("Home", "Errands"), ("Home", "Reading"), ("Home", "Garden"), ("Spring launch", "Website"), ("Spring launch", "Press kit")],
    # (key, list, title, parent key, extra)
    "tasks": [
        ("groceries", "Errands", "Buy groceries for the week", None, {"age": 2 * DAY}),
        ("milk", "Errands", "Oat milk", "groceries", {"done": True}),
        ("bread", "Errands", "Sourdough loaf", "groceries", {}),
        ("apples", "Errands", "Apples", "groceries", {}),
        ("pharmacy", "Errands", "Pick up prescription", None, {"notes": "Counter closes at 6"}),
        ("post", "Errands", "Post the parcel to Mum", None, {"done": True, "age": 3 * DAY}),
        ("bike", "Errands", "Book bike service", None, {"age": 5 * DAY}),
        ("keys", "Errands", "Cut a spare key", None, {}),
        ("novel", "Reading", "Finish the novel on the nightstand", None, {}),
        ("article", "Reading", "Read the long article on soil health", None, {"done": True}),
        ("library", "Reading", "Return library books", None, {}),
        ("seeds", "Garden", "Order tomato seeds", None, {}),
        ("hedge", "Garden", "Trim the hedge", None, {}),
        ("hero", "Website", "Rewrite the hero section", None, {"age": 4 * DAY}),
        ("copy", "Website", "Draft headline options", "hero", {}),
        ("shot", "Website", "Pick the hero image", "hero", {"done": True}),
        ("pricing", "Website", "Pricing page: compare plans table", None, {}),
        ("faq", "Website", "Collect FAQ questions from support", None, {}),
        ("logo", "Press kit", "Export logos in SVG and PNG", None, {}),
        ("bios", "Press kit", "Team bios, 50 words each", None, {"done": True}),
        ("old-flyer", "Errands", "Print the old flyer", None, {"deleted": 2 * DAY}),
        ("old-call", "Website", "Call the printer about samples", None, {"deleted": 30 * MIN}),
    ],
    "notes": [
        ("hero", '<p>Lead with the <strong>one-line promise</strong>, then three proof points.</p><ul><li>Fast capture</li><li>Works offline</li><li>Keyboard first</li></ul>'),
        ("hero", "<p>Ask design for two layouts by Friday.</p>"),
        ("groceries", "<p>Check the cupboard before going.</p>"),
    ],
    "links": [("hero", "Brand brief", "https://example.com/brand-brief")],
    "files": [("hero", "hero-sketch.png")],
    "last": ("Home", "Errands"),
}
DATASETS = {
    "standard": STANDARD,
    "empty": {"projects": [("Home", DAY)], "lists": [("Home", "Inbox")], "tasks": [], "notes": [], "links": [], "files": [], "last": ("Home", "Inbox")},
    "nolists": {"projects": [("Home", DAY)], "lists": [], "tasks": [], "notes": [], "links": [], "files": [], "last": ("Home", None)},
    "long": {**STANDARD, "tasks": STANDARD["tasks"] + [
        (f"long-{i}", "Errands", f"Errand number {i}", None, {"done": i % 5 == 1}) for i in range(1, 41)]},
}


@transaction.atomic
def load(user, name="standard"):
    data, now = DATASETS[name], timezone.now()
    for a in Attachment.objects.filter(task__list__project__owner=user).exclude(file=""):
        a.file.delete(save=False)
    Project.objects.filter(owner=user).delete()
    projects = {}
    for i, (pname, age) in enumerate(data["projects"]):
        projects[pname] = Project.objects.create(owner=user, name=pname, sort_order=i)
        Project.objects.filter(pk=projects[pname].pk).update(created_at=now - age, updated_at=now - age)
    lists, per_project = {}, {}
    for pname, lname in data["lists"]:
        n = per_project[pname] = per_project.get(pname, -1) + 1
        lists[lname] = TaskList.objects.create(project=projects[pname], name=lname, sort_order=n)
    tasks, order = {}, 0
    for key, lname, title, parent, extra in data["tasks"]:
        age = extra.get("age", DAY)
        t = Task.objects.create(list=lists[lname], parent=tasks.get(parent), title=title, notes=extra.get("notes", ""),
                                done=extra.get("done", False), done_at=now - age / 2 if extra.get("done") else None,
                                sort_order=order, deleted_at=now - extra["deleted"] if "deleted" in extra else None)
        Task.objects.filter(pk=t.pk).update(created_at=now - age, updated_at=now - age)
        tasks[key], order = t, order + 1
    for i, (key, content) in enumerate(data["notes"]):
        Note.objects.create(task=tasks[key], content=content, sort_order=i)
    for key, name, url in data["links"]:
        Attachment.objects.create(task=tasks[key], kind=Attachment.Kind.LINK, name=name, url=url)
    for key, fname in data["files"]:
        with open(HERE / fname, "rb") as f:
            a = Attachment(task=tasks[key], kind=Attachment.Kind.FILE, name=fname, size=(HERE / fname).stat().st_size, mime="image/png")
            a.file.save(fname, File(f), save=True)
    s, _ = UserSettings.objects.get_or_create(user=user)
    pname, lname = data["last"]
    s.last_project, s.last_list = projects.get(pname), lists.get(lname) if lname else None
    s.save()
    return {"projects": len(projects), "lists": len(lists), "tasks": len(tasks)}
