#!/usr/bin/env python3
"""Render matching egui states using the reference ZIP and the browser recorder's fixtures."""
import argparse
import json
from pathlib import Path
import re
import shutil
import sqlite3
import struct
import subprocess
import sys
import tempfile
import zipfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
STORAGE = ROOT / "apps/desktop/storage"
ADAPTER = ROOT / "omnicui/targets/desktop"


def seed(directory, fixture, epoch):
    directory.mkdir()
    with sqlite3.connect(directory / "taskscape.db") as db:
        db.execute("CREATE TABLE schema_migrations (version TEXT PRIMARY KEY, applied_at INTEGER NOT NULL)")
        for migration in sorted((STORAGE / "migrations").glob("*.sql")):
            db.executescript(migration.read_text())
            db.execute("INSERT INTO schema_migrations VALUES (?, ?)", (migration.stem, epoch))
        for source, table in [("projects", "projects"), ("lists", "lists"), ("tasks", "tasks"),
                              ("notes", "task_notes"), ("attachments", "attachments")]:
            columns = [r[1] for r in db.execute(f"PRAGMA table_info({table})")]
            for row in fixture.get(source, []):
                keys = [k for k in columns if k in row]
                db.execute(f"INSERT INTO {table} ({','.join(keys)}) VALUES ({','.join('?' for _ in keys)})",
                           [row[k] for k in keys])
    (directory / "settings.json").write_text(json.dumps({"seeded": "1", **fixture.get("settings", {})}))
    (directory / "attachments").mkdir()
    shutil.copyfile(HERE / "mock/files/hero-sketch.png", directory / "attachments/hero-sketch.png")


def png_dimensions(header):
    if header[:8] != b"\x89PNG\r\n\x1a\n" or header[12:16] != b"IHDR":
        raise ValueError("expected PNG header")
    return struct.unpack(">II", header[16:24])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("states", nargs="*")
    parser.add_argument("--binary", type=Path, help="use an already built record-egui binary")
    args = parser.parse_args()
    if sys.platform != "darwin":
        parser.error("macOS is required for the reference fonts")
    epoch = int(re.search(r"const T = (\d+);", (HERE / "capture.mjs").read_text())[1])
    viewer = json.loads(subprocess.check_output(
        ["node", "--input-type=module", "-e",
         "import {fixtures} from './mock/fixtures.js'; import {states} from './states.mjs';"
         "console.log(JSON.stringify({fixtures:fixtures(+process.argv[1]),states:states.map(s=>s.id)}))",
         str(epoch)], cwd=HERE))
    ids = args.states or viewer["states"]
    args.output.mkdir(parents=True, exist_ok=True)
    with (tempfile.TemporaryDirectory(prefix="taskscape-egui-", dir=args.output.resolve().parent) as tmp,
          zipfile.ZipFile(args.reference) as archive):
        tmp = Path(tmp)
        manifest = []
        for name in ids:
            if name not in viewer["states"]:
                parser.error(f"unknown reference state {name}")
            meta = json.loads(archive.read(f"references/{name}/meta.json"))
            if meta["id"] != name or meta["problems"]:
                parser.error(f"the ZIP has no clean recording for {name}")
            with archive.open(f"references/{name}/shot.png") as shot:
                meta["size"] = png_dimensions(shot.read(24))
            directory = tmp / name
            seed(directory, viewer["fixtures"][meta["fixture"]], epoch)
            manifest.append({**meta, "data": str(directory), "epoch": epoch})
        request = tmp / "manifest.json"
        request.write_text(json.dumps(manifest))
        command = ([str(args.binary.resolve())] if args.binary else
                   ["cargo", "run", "--locked", "--manifest-path", str(ADAPTER / "Cargo.toml"),
                    "--bin", "record-egui", "--"])
        log = args.output / "egui-record.log"
        with log.open("w") as out:
            proc = subprocess.Popen(command + [str(request), str(args.output.resolve())],
                                    stdout=subprocess.PIPE, text=True)
            for line in proc.stdout:
                print(line, end="", flush=True)
                out.write(line)
            status = proc.wait()
        if status:
            return status
        for meta in manifest:
            with (args.output / meta["id"] / "shot.png").open("rb") as shot:
                if list(png_dimensions(shot.read(24))) != list(meta["size"]):
                    raise ValueError(f"{meta['id']}: PNG size does not match the reference")
        return status


if __name__ == "__main__":
    raise SystemExit(main())
