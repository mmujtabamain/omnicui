from django.contrib.auth import get_user_model
from django.core.management.base import BaseCommand

from planner import demo


class Command(BaseCommand):
    help = "Create (or reset) the demo account with one of the demo data sets."

    def add_arguments(self, parser):
        parser.add_argument("--email", default="demo@taskscape.test")
        parser.add_argument("--password", default="taskscape-demo")
        parser.add_argument("--dataset", default="standard", choices=sorted(demo.DATASETS))

    def handle(self, email, password, dataset, **_):
        User = get_user_model()
        user = User.objects.filter(email=email).first() or User.objects.create_user(username=email, email=email, password=password, first_name="Demo")
        counts = demo.load(user, dataset)
        self.stdout.write(f"{email}: {dataset} ({counts['projects']} projects, {counts['lists']} lists, {counts['tasks']} tasks)")
