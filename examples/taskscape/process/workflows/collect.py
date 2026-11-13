"""Turns workflow results into the feature list.

  collect.py merged <results.json>...   W1/W2 results -> spec/features/area-NN.json (ids assigned)
  collect.py index                      one line per feature, for W3's critics
  collect.py seams <result.json>        W3's finds -> appended to their areas, new ids
  collect.py chunks [size]              W3b items: each area's features in chunks
  collect.py final <results.json>...    W3b results -> area files updated in place
  collect.py markdown                   spec/features.md, the whole list for review
"""
import html, json, os, re, sys

HERE = os.path.dirname(os.path.abspath(__file__))
FEAT = os.path.abspath(os.path.join(HERE, "../../spec/features"))
AREAS = {a["n"]: a for a in json.load(open(os.path.join(HERE, "areas.json")))["areas"]}


def unescape(v):
    if isinstance(v, str):
        return html.unescape(v)
    if isinstance(v, list):
        return [unescape(x) for x in v]
    if isinstance(v, dict):
        return {k: unescape(x) for k, x in v.items()}
    return v


def area_file(n):
    return os.path.join(FEAT, f"area-{n:02d}.json")


def load(n):
    return json.load(open(area_file(n)))


def save(n, data):
    os.makedirs(FEAT, exist_ok=True)
    json.dump(data, open(area_file(n), "w"), indent=1, ensure_ascii=False)


def results(paths):
    for p in paths:
        r = json.load(open(p))
        if isinstance(r, dict) and "result" in r and "agentCount" in r:
            r = r["result"]  # a Workflow task's output file
        yield from (r if isinstance(r, list) else [r])


def cmd_merged(paths):
    for r in results(paths):
        if not r or r.get("failed"):
            print("FAILED", r and r.get("area"))
            continue
        n = r["area"]
        feats = unescape(r["merged"]["features"])
        for i, f in enumerate(feats, 1):
            f["id"] = f"F{n:02d}-{i:03d}"
        save(n, {"area": n, "name": r["name"], "tier": r["tier"],
                 "counts": {"A": r["A"], "B": r["B"], "critic_issues": r["issues"], "merged": len(feats)},
                 "dropped": unescape(r["merged"].get("dropped", [])), "features": feats})
        print(n, r["name"], f"A={r['A']} B={r['B']} merged={len(feats)}")


def all_areas():
    return sorted(int(m.group(1)) for f in os.listdir(FEAT) if (m := re.match(r"area-(\d+)\.json", f)))


def cmd_index():
    for n in all_areas():
        for f in load(n)["features"]:
            print(f"{f['id']} | {n} | {f['title']} | {f['trigger']}")


def cmd_seams(path):
    finds = unescape(json.load(open(path))["features"])
    for f in finds:
        n = f.pop("area")
        data = load(n)
        last = max(int(x["id"].split("-")[1]) for x in data["features"]) if data["features"] else 0
        f["id"] = f"F{n:02d}-{last + 1:03d}"
        f["found_by"] = ["seams"]
        data["features"].append(f)
        save(n, data)
        print(f["id"], f["title"])


def cmd_chunks(size=35, part=None, parts=None):
    items = []
    for n in all_areas():
        a = AREAS[n]
        feats = load(n)["features"]
        keep = ("id", "title", "trigger", "context", "result", "exact", "edge_cases", "sources")
        for i in range(0, len(feats), size):
            chunk = feats[i:i + size]
            # Checker B gets the recordings this chunk cites first, at most 9 in all.
            cited = [s for s in a["statesB"] if any(f"state:{s}" in src for f in chunk for src in f["sources"])]
            states = (cited + [s for s in a["statesB"] if s not in cited])[:9]
            key = f"{n}.{i // size + 1}"
            path = os.path.join(FEAT, "chunks", f"{key}.json")
            os.makedirs(os.path.dirname(path), exist_ok=True)
            json.dump([{k: f[k] for k in keep if k in f} for f in chunk], open(path, "w"), indent=1, ensure_ascii=False)
            items.append({"key": key, "file": path,
                          "area": {**{k: a[k] for k in ("n", "name", "filesA", "docsB")}, "statesB": states}})
    if part is not None:
        items = items[part::parts]
    print(json.dumps(items))


def cmd_final(paths):
    res = {}
    for r in results(paths):
        if not r or r.get("failed"):
            print("FAILED", r and r.get("key"))
            continue
        for x in unescape(r["final"]["resolutions"]):
            res[x["id"]] = x
    counts = {}
    for n in all_areas():
        data = load(n)
        for f in data["features"]:
            x = res.get(f["id"])
            if not x:
                f.setdefault("check", "unchecked")
            else:
                for k, v in (x.get("corrections") or {}).items():
                    if v not in (None, "", []):
                        f[k] = v
                f["check"] = x["check"]
                f["sources"] = list(dict.fromkeys(f["sources"] + [e for e in x.get("evidence", []) if e]))
                if x.get("check_notes"):
                    f["check_notes"] = x["check_notes"]
            counts[f["check"]] = counts.get(f["check"], 0) + 1
        save(n, data)
    print(counts)


def short(src):
    src = re.sub(r"^/\S*?/taskscape-tauri/", "", src)
    return re.sub(r"^/\S*?/spec/references/", "state:", src)


def area_markdown(n):
    data = load(n)
    out = [f"## {n}. {data['name']} ({len(data['features'])} features)", ""]
    for f in data["features"]:
        out.append(f"**{f['id']} · {f['title']}**")
        out.append(f"- *Trigger:* {f['trigger']}")
        out.append(f"- *When:* {f['context']}")
        out.append(f"- *Result:* {f['result']}")
        if f.get("exact"):
            out.append("- *Exact:* " + "; ".join(f"{e['what']} = `{e['value']}`" for e in f["exact"]))
        if f.get("edge_cases"):
            out.append("- *Edge cases:* " + " · ".join(f["edge_cases"]))
        tags = [f.get("check", "unchecked").replace("_", " "), "found by " + "+".join(f.get("found_by") or ["?"])]
        if f.get("system"):
            tags.append("OS integration")
        out.append(f"- *Status:* {', '.join(tags)}")
        if f.get("check_notes"):
            out.append(f"- *Check notes:* {f['check_notes']}")
        out.append("- *Sources:* " + ", ".join(dict.fromkeys(short(x) for x in f["sources"])))
        out.append("")
    return "\n".join(out)


def cmd_markdown():
    total, parts = 0, []
    for n in all_areas():
        total += len(load(n)["features"])
        parts.append(area_markdown(n))
    head = ["# Taskscape: the feature list", "",
            f"{total} features of the reference app in {len(all_areas())} areas. Each was found independently from the code (A) "
            "and from recordings of the running app or its docs (B), merged, then checked for exactness against both.", "",
            "Pixel sizes: the reference sets the root font size to 14px, so 1rem = 14px and a Tailwind class like h-8 is 28px.", ""]
    open(os.path.join(FEAT, "..", "features.md"), "w").write("\n".join(head + parts))
    print(total, "features")


if __name__ == "__main__":
    c, rest = sys.argv[1], sys.argv[2:]
    {"merged": lambda: cmd_merged(rest), "index": cmd_index, "seams": lambda: cmd_seams(rest[0]),
     "chunks": lambda: cmd_chunks(*(int(x) for x in rest)), "final": lambda: cmd_final(rest),
     "markdown": cmd_markdown}[c]()
