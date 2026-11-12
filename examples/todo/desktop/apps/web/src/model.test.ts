// The store's rules, and the fixture file's identity: `npm test`.
import { test } from "node:test";
import { strictEqual, deepStrictEqual } from "node:assert";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { TodoStore, seedTasks, visible, left, emptyMessage, type TodoEvent } from "./model.ts";

test("the seeds are the shared fixture file, byte for byte", () => {
  const sha = createHash("sha256").update(readFileSync(new URL("./seeds.json", import.meta.url))).digest("hex");
  strictEqual(sha, "c1ace19804fea36d7f4a6e1b96c2e183c518741010339f09c058615992a51d2e");
  deepStrictEqual(["empty", "three", "many"].map((s) => seedTasks(s as any).length), [0, 3, 20]);
});

test("adding trims the title, continues the ids and reports it", () => {
  const events: TodoEvent[] = [];
  const s = new TodoStore(seedTasks("three"), {}, (e) => events.push(e));
  strictEqual(s.add("  Water the plants ")?.id, "t4");
  strictEqual(s.add("   "), undefined);
  s.delete("t4");
  strictEqual(s.add("Again")?.id, "t5");
  deepStrictEqual(events.map((e) => e.name), ["added", "deleted", "added"]);
});

test("filters, the count left, empty states and show completed", () => {
  const s = new TodoStore(seedTasks("three"));
  strictEqual(left(s.get()), 2);
  s.select("done");
  deepStrictEqual(visible(s.get()).map((t) => t.id), ["t2"]);
  s.setDone("t2", false);
  strictEqual(emptyMessage(s.get()), "Nothing done yet");
  const hidden = new TodoStore(seedTasks("three"), { showCompleted: false });
  deepStrictEqual(visible(hidden.get()).map((t) => t.id), ["t1", "t3"]);
  strictEqual(emptyMessage(new TodoStore([]).get()), "No tasks yet");
});

test("only real changes are reported", () => {
  const events: TodoEvent[] = [];
  const s = new TodoStore(seedTasks("three"), {}, (e) => events.push(e));
  s.setDone("t1", true); s.setDone("t1", true); s.rename("t1", "Buy milk"); s.select("all"); s.chooseTheme("system"); s.delete("t9");
  deepStrictEqual(events, [{ name: "toggled", fields: { id: "t1", done: true } }]);
});
