"""Applies the platform-tag audit (workflow taskscape-platform-tags) to the contracts and platforms.yaml.

    python3 process/workflows/apply_platform_tags.py changes.json [--dry-run]

changes.json is the workflow's result: [{batch, changes: [{contract, kind, test, tag, after, yaml, part, input,
forms, reason}]}]. Edits are made on the text, so comments and layout stay as they are. A change that names a
test or part the contract does not have is skipped and reported, never guessed at.
"""
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CONTRACTS = ROOT / "omnicui" / "contracts"
PLATFORMS = ROOT / "omnicui" / "platforms.yaml"


def contract_file(cid: str) -> Path:
    return CONTRACTS / f"{cid.split('.', 1)[-1]}.contract.yaml"


def norm(name: str) -> str:
    return name.strip().strip("'\"").strip()


def test_blocks(lines):
    """(start, end, name) of each test in the tests: list; end is exclusive."""
    blocks, inside, start, name = [], False, None, None
    for i, line in enumerate(lines):
        if re.match(r"^tests:\s*$", line):
            inside = True
            continue
        if inside and re.match(r"^\S", line):  # the next top-level key ends the list
            if start is not None:
                blocks.append((start, i, name))
            inside, start = False, None
            continue
        m = re.match(r"^  - name:\s*(.*?)\s*$", line) if inside else None
        if m:
            if start is not None:
                blocks.append((start, i, name))
            start, name = i, norm(m.group(1))
    if inside and start is not None:
        blocks.append((start, len(lines), name))
    # trailing blank and comment lines belong to the gap before the next test, not to this one
    out = []
    for s, e, n in blocks:
        while e - 1 > s and (lines[e - 1].strip() == "" or lines[e - 1].lstrip().startswith("#")):
            e -= 1
        out.append((s, e, n))
    return out


def find(lines, name):
    want = norm(name)
    hits = [b for b in test_blocks(lines) if b[2] == want]
    return hits[0] if hits else None


def indent_item(yaml_text: str) -> list[str]:
    """A test written at column 0 ("- name: …"), moved under tests: (two spaces)."""
    body = yaml_text.strip("\n").split("\n")
    return [("  " + l) if l.strip() else "" for l in body]


def apply(path: Path, changes, report, dry):
    lines = path.read_text().split("\n")
    for c in changes:
        kind, what = c["kind"], c.get("test") or c.get("after") or c.get("part") or ""
        if kind in ("tag", "remove", "replace"):
            b = find(lines, c.get("test", ""))
            if not b:
                report.append(f"SKIP {path.name}: no test {c.get('test')!r} ({kind})")
                continue
            s, e, _ = b
            if kind == "remove":
                del lines[s:e]
            elif kind == "replace":
                lines[s:e] = indent_item(c["yaml"])
            else:
                block = [l for l in lines[s:e] if not re.match(r"^    (only|except):", l)]
                tag = (c.get("tag") or "").strip()
                if tag and tag != "none":
                    block.insert(1, f"    {tag}")
                lines[s:e] = block
        elif kind == "add":
            b = find(lines, c.get("after", ""))
            if not b:
                report.append(f"SKIP {path.name}: no test {c.get('after')!r} to add after")
                continue
            lines[b[1]:b[1]] = [""] + indent_item(c["yaml"])
        elif kind == "optional":
            part = c.get("part", "")
            for i, l in enumerate(lines):
                m = re.match(rf"^  {re.escape(part)}:\s*\{{(.*?)\}}(.*)$", l)
                if m:
                    if "optional" not in m.group(1):
                        lines[i] = f"  {part}: {{{m.group(1).rstrip()}, optional: true }}{m.group(2)}"
                    break
            else:
                report.append(f"SKIP {path.name}: no part {part!r}")
                continue
        else:
            continue
        report.append(f"OK   {path.name}: {kind} {what}")
    if not dry:
        path.write_text("\n".join(lines))


def main():
    src, dry = sys.argv[1], "--dry-run" in sys.argv
    data = json.loads(Path(src).read_text())
    changes = [c for b in data if b and b.get("changes") for c in b["changes"]]
    report, by_file, inputs, unsure = [], {}, [], []
    for c in changes:
        if c["kind"] == "input":
            inputs.append(c)
        elif c["kind"] == "unsure":
            unsure.append(c)
        else:
            by_file.setdefault(contract_file(c["contract"]), []).append(c)
    for path, cs in sorted(by_file.items()):
        if not path.exists():
            report.append(f"SKIP no contract file {path.name}")
            continue
        apply(path, cs, report, dry)
    if inputs:
        text = PLATFORMS.read_text()
        for c in inputs:
            if re.search(rf"^  {re.escape(c['input'])}:", text, re.M):
                report.append(f"SKIP platforms.yaml: {c['input']} exists")
                continue
            text = text.rstrip("\n") + f"\n  {c['input']}: {c['forms']}\n"
            report.append(f"OK   platforms.yaml: input {c['input']}")
        if not dry:
            PLATFORMS.write_text(text)
    print("\n".join(report))
    for c in unsure:
        print(f"UNSURE {c['contract']}: {c.get('test')}: {c['reason']}")


if __name__ == "__main__":
    main()
