"""Writes contract workflow results to omnicui/contracts/ and checks each with omnicui.

  collect_contracts.py write <results.json>...   one <id>.contract.yaml per widget
  collect_contracts.py check                     omnicui contracts on each file alone; errors to spec/library/contract-errors.json
"""
import html, json, os, shutil, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "../.."))
CONTRACTS = os.path.join(ROOT, "omnicui", "contracts")
OMNICUI = os.path.abspath(os.path.join(ROOT, "../../bin/omnicui"))


def results(paths):
    for p in paths:
        r = json.load(open(p))
        if isinstance(r, dict) and "result" in r and "agentCount" in r:
            r = r["result"]
        yield from (r if isinstance(r, list) else [r])


def cmd_write(paths):
    os.makedirs(CONTRACTS, exist_ok=True)
    for r in results(paths):
        if not r or r.get("failed") or not r.get("contract"):
            print("FAILED", r and r.get("name"))
            continue
        y = html.unescape(r["contract"]["yaml"]).strip()
        if y.startswith("```"):
            y = y.split("\n", 1)[1].rsplit("```", 1)[0]
        y = "\n".join(l for l in y.split("\n") if not l.startswith("//"))
        name = r["id"].split(".", 1)[1]
        open(os.path.join(CONTRACTS, f"{name}.contract.yaml"), "w").write(y.rstrip() + "\n")
        print("wrote", name)


def check_one(path):
    with tempfile.TemporaryDirectory() as d:
        os.makedirs(os.path.join(d, "contracts"))
        shutil.copy(path, os.path.join(d, "contracts"))
        shutil.copy(os.path.join(ROOT, "omnicui", "platforms.yaml"), d)
        p = subprocess.run([OMNICUI, "--project", d, "contracts"], capture_output=True, text=True)
        return p.returncode, (p.stdout + p.stderr).replace(d, "").strip()


def cmd_check():
    errors = {}
    for f in sorted(os.listdir(CONTRACTS)):
        if not f.endswith(".contract.yaml"):
            continue
        code, out = check_one(os.path.join(CONTRACTS, f))
        if code != 0:
            errors[f] = out[-3000:]
        print("ok " if code == 0 else "BAD", f)
    json.dump(errors, open(os.path.join(ROOT, "spec", "library", "contract-errors.json"), "w"), indent=1)
    print(len(errors), "with errors")


if __name__ == "__main__":
    c, rest = sys.argv[1], sys.argv[2:]
    if c != "mechanical":
        {"write": lambda: cmd_write(rest), "check": cmd_check}[c]()


ROLE_MAP = {
    "option": "button", "tab": "button", "menuitem": "button", "menuitemradio": "button", "menuitemcheckbox": "button",
    "treeitem": "button", "link": "button", "combobox": "button",
    "listbox": "generic", "list": "generic", "listitem": "generic", "toolbar": "generic", "menu": "generic",
    "tree": "generic", "tablist": "generic", "tabpanel": "generic", "dialog": "generic", "alertdialog": "generic",
    "banner": "generic", "img": "generic", "group": "generic", "region": "generic", "separator": "generic",
    "progressbar": "status", "none": "generic", "presentation": "generic", "radio": "checkbox", "radiogroup": "generic", "navigation": "generic", "main": "generic", "complementary": "generic", "contentinfo": "generic", "article": "generic", "figure": "generic", "grid": "generic", "row": "generic", "gridcell": "generic", "cell": "generic", "status": "status", "textbox": "textbox", "alert": "status", "log": "status", "tooltip": "status", "searchbox": "textbox",
}


def cmd_mechanical():
    """Fixes the two systematic mistakes without an agent: ARIA roles omnicui lacks, and extra prop fields."""
    import re
    for f in sorted(os.listdir(CONTRACTS)):
        p = os.path.join(CONTRACTS, f)
        s = o = open(p).read()
        s = "\n".join(l for l in s.split("\n") if not l.startswith("//"))
        s = re.sub(r"(role:\s*)[\"']?([a-z]+)[\"']?", lambda m: m.group(1) + ROLE_MAP.get(m.group(2), m.group(2)) + (f"  # aria: {m.group(2)}" if m.group(2) in ROLE_MAP and "\n" not in m.group(0) and False else ""), s)
        lines, out, in_props = s.split("\n"), [], False
        for line in lines:
            if re.match(r"^props:\s*$", line):
                in_props = True
            elif re.match(r"^\S", line):
                in_props = False
            if in_props and re.match(r"^\s{4,}(description|enum|values|examples):", line):
                out.append(re.sub(r"^(\s*)", r"\1# ", line, 1))
                continue
            out.append(line)
        s = "\n".join(out)
        if s != o:
            open(p, "w").write(s)
            print("fixed", f)


if __name__ == "__main__" and sys.argv[1] == "mechanical":
    cmd_mechanical()
