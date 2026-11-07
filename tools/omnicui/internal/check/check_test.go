package check

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"omnicui/internal/contract"
	"omnicui/internal/project"
	"omnicui/internal/testutil"
)

type T = testutil.Target

func run(t *testing.T, targets map[string]T, require ...string) (*Report, project.Project) {
	t.Helper()
	root := testutil.Project(t, targets)
	p := project.Project{Root: root}
	cs, ps, err := contract.LoadDir(p.Contracts())
	if err != nil || contract.HasErrors(ps) {
		t.Fatal(err, ps)
	}
	ts, err := p.Targets()
	if err != nil {
		t.Fatal(err)
	}
	rep, err := Run(Options{Project: p, Contracts: cs, Targets: ts, Require: require})
	if err != nil {
		t.Fatal(err)
	}
	return rep, p
}

func result(t *testing.T, rep *Report, target, test string) TestResult {
	t.Helper()
	for _, r := range rep.Results {
		if r.Target == target && r.Test == test {
			return r
		}
	}
	t.Fatalf("no result for %s / %s", target, test)
	return TestResult{}
}

func warnings(rep *Report, kind string) []Comparison {
	var out []Comparison
	for _, c := range rep.Comparisons {
		if c.Kind == kind && c.Warning {
			out = append(out, c)
		}
	}
	return out
}

// R-CHECK-1: two targets that behave and look alike: every test passes on both, with no warnings,
// and results.json, report.html, omnicui.log and protocol.log are written.
func TestAllPass(t *testing.T) {
	rep, p := run(t, map[string]T{"a": {}, "b": {}})
	if rep.Summary.Pass != 14 || rep.Failed() || rep.Summary.Warnings != 0 {
		t.Fatalf("summary %+v\n%+v", rep.Summary, rep.Results)
	}
	for _, f := range []string{"results.json", "report.html", "omnicui.log", "protocol.log"} {
		if _, err := os.Stat(filepath.Join(p.RunDir(), f)); err != nil {
			t.Error(err)
		}
	}
	var pix, tim int
	for _, c := range rep.Comparisons {
		if c.Kind == "pixels" {
			pix++
			if _, err := os.Stat(filepath.Join(p.RunDir(), c.Diff)); err != nil {
				t.Error("diff image:", err)
			}
		} else {
			tim++
		}
	}
	// 5 tests take screenshots, one pair each; one trace: the pair and each target against the timing.
	if pix != 5 || tim != 3 {
		t.Errorf("comparisons: %d pixels, %d timing", pix, tim)
	}
	if r := result(t, rep, "a", "a press turns it on, and the thumb slides across"); r.Clip == "" || len(r.Traces) != 1 || len(r.Screenshots) != 2 {
		t.Errorf("artifacts: %+v", r)
	}
}

// R-CHECK-2: a mismatch fails the test on that target only, saying what was expected and what came back.
func TestFailMessage(t *testing.T) {
	rep, _ := run(t, map[string]T{"a": {}, "b": {Env: map[string]string{"FAKE_BUG": "no-name"}}})
	r := result(t, rep, "b", "the keyboard works too")
	if r.Status != Fail || len(r.Messages) != 1 || r.Messages[0] != `increment.name: expected "Increment", got nothing` {
		t.Fatalf("%+v", r)
	}
	if result(t, rep, "a", "the keyboard works too").Status != Pass || !rep.Failed() {
		t.Error("only b may fail, and the run must fail")
	}
	rep, _ = run(t, map[string]T{"a": {Env: map[string]string{"FAKE_BUG": "double"}}})
	r = result(t, rep, "a", "each press adds one")
	if r.Status != Fail || !strings.Contains(strings.Join(r.Messages, "\n"), `value.text: expected "3", got "6"`) ||
		!strings.Contains(strings.Join(r.Messages, "\n"), "changed.last.count: expected 3, got 6") {
		t.Fatalf("%+v", r)
	}
}

// R-CHECK-3: components are not advertised; a target without one fails its tests at mount.
func TestMissingComponent(t *testing.T) {
	rep, _ := run(t, map[string]T{"a": {Env: map[string]string{"FAKE_COMPONENTS": "ui.counter"}}})
	r := result(t, rep, "a", "starts on when asked")
	if r.Status != Fail || r.Messages[0] != "mount: no component ui.switch" {
		t.Fatalf("%+v", r)
	}
	if result(t, rep, "a", "each press adds one").Status != Pass {
		t.Error("the counter must still pass")
	}
}

// R-CHECK-4: a capability the target doesn't advertise is "not advertised", with its reason; it doesn't fail.
func TestNotAdvertised(t *testing.T) {
	rep, _ := run(t, map[string]T{"a": {}, "b": {Env: map[string]string{"FAKE_CANNOT": "screenshot=no display,trace=no display,record=no display"}}})
	r := result(t, rep, "b", "a press turns it on, and the thumb slides across")
	if r.Status != Pass || r.Missing["screenshot"] != "no display" || r.Missing["trace"] != "no display" || r.Missing["record"] != "no display" {
		t.Fatalf("%+v", r)
	}
	if rep.Failed() || rep.Summary.NotAdvertised != 7 || len(rep.Comparisons) != 1 {
		t.Errorf("summary %+v, %d comparisons", rep.Summary, len(rep.Comparisons))
	}
	// A test whose steps need a construct the target lacks can't run there at all.
	rep, _ = run(t, map[string]T{"a": {Env: map[string]string{"FAKE_CAN": "mount,press,read,events,screenshot"}}})
	r = result(t, rep, "a", "the keyboard works too")
	if r.Status != NotAdvertised || !strings.Contains(r.Messages[0], "needs `focus`") {
		t.Fatalf("%+v", r)
	}
}

// R-CHECK-5: --require target:capability turns "not advertised" into a failure of the run.
func TestRequire(t *testing.T) {
	rep, _ := run(t, map[string]T{"a": {Env: map[string]string{"FAKE_CANNOT": "screenshot=no display"}}}, "a:screenshot", "a:trace", "zz:trace")
	if !rep.Failed() || len(rep.Required) != 2 || !strings.Contains(rep.Required[0], "a must advertise `screenshot` (--require), but it doesn't: no display") ||
		!strings.Contains(rep.Required[1], "there is no target `zz`") {
		t.Fatalf("%v", rep.Required)
	}
}

// R-CHECK-6: screenshots that differ by more than the threshold are warnings, with a diff image; they don't fail.
func TestPixelWarning(t *testing.T) {
	rep, _ := run(t, map[string]T{"a": {}, "b": {Env: map[string]string{"FAKE_COLOR": "c0392b", "FAKE_SHIFT": "8"}}})
	ws := warnings(rep, "pixels")
	if len(ws) != 5 || rep.Failed() || ws[0].Value <= ws[0].Threshold {
		t.Fatalf("%+v", ws)
	}
}

// R-CHECK-7: a trace that moves with another timing is a warning, against the other target and the named timing.
func TestTimingWarning(t *testing.T) {
	rep, _ := run(t, map[string]T{"a": {}, "b": {Env: map[string]string{"FAKE_EASING": "linear", "FAKE_DURATION": "300"}}})
	ws := warnings(rep, "timing")
	if len(ws) != 2 || rep.Failed() {
		t.Fatalf("%+v", rep.Comparisons)
	}
	for _, c := range rep.Comparisons {
		if c.Kind == "timing" && c.A == "a" && c.B == "timing" && c.Warning {
			t.Errorf("a matches its timing: %+v", c)
		}
	}
}

// R-CHECK-8 (failure: a worker stops mid-task): the target is broken from that test on, with its
// stderr; the tests after it are "not run"; other targets are not affected.
func TestCrashMidRun(t *testing.T) {
	rep, _ := run(t, map[string]T{"a": {}, "b": {Env: map[string]string{"FAKE_CRASH_ON": "12"}}})
	var broken, notRun int
	for _, r := range rep.Results {
		if r.Target == "a" && r.Status != Pass {
			t.Errorf("a: %+v", r)
		}
		if r.Target == "b" && r.Status == Broken {
			if strings.HasPrefix(r.Messages[0], "not run: the target broke earlier") {
				notRun++
			} else if strings.Contains(r.Messages[0], "exited with code 3") {
				broken++
			}
		}
	}
	if broken != 1 || notRun < 4 || !rep.Failed() {
		t.Fatalf("broken %d, not run %d", broken, notRun)
	}
	for _, ti := range rep.Targets {
		if ti.Name == "b" && (ti.Broken == "" || !strings.Contains(strings.Join(ti.Stderr, "\n"), "fatal:")) {
			t.Errorf("target info: %+v", ti)
		}
	}
}

// R-CHECK-9 (failure: a dependency times out): a hung step breaks the target after its step timeout.
func TestHang(t *testing.T) {
	rep, _ := run(t, map[string]T{"a": {Env: map[string]string{"FAKE_HANG_ON": "3"}, Extra: "stepTimeout: 500ms\n"}})
	r := rep.Results[0]
	if r.Status != Broken || !strings.Contains(r.Messages[0], "timed out after 500ms") {
		t.Fatalf("%+v", r)
	}
}

// R-CHECK-10: a target that can't start, or doesn't answer advertise, is broken for every test.
func TestCannotStart(t *testing.T) {
	rep, _ := run(t, map[string]T{"a": {Env: map[string]string{"FAKE_STRAY_ON": "1"}}})
	if rep.Summary.Broken != 7 || !strings.Contains(rep.Targets[0].Broken, "did not answer advertise: stray output") {
		t.Fatalf("%+v %+v", rep.Summary, rep.Targets)
	}
}

func strip(t *testing.T, p project.Project) map[string]any {
	t.Helper()
	data, err := os.ReadFile(filepath.Join(p.RunDir(), "results.json"))
	if err != nil {
		t.Fatal(err)
	}
	var m map[string]any
	json.Unmarshal(data, &m)
	delete(m, "runId")
	return m
}

// R-CHECK-11 (the same request twice / a restart): omnicui keeps no state; running the same check
// again gives the same results, and each run replaces the last run folder entirely.
func TestRepeatable(t *testing.T) {
	root := testutil.Project(t, map[string]T{"a": {}, "b": {}})
	p := project.Project{Root: root}
	cs, _, _ := contract.LoadDir(p.Contracts())
	ts, _ := p.Targets()
	Run(Options{Project: p, Contracts: cs, Targets: ts})
	first, _ := json.Marshal(strip(t, p))
	os.WriteFile(filepath.Join(p.RunDir(), "stale.txt"), []byte("left over"), 0o644)
	Run(Options{Project: p, Contracts: cs, Targets: ts})
	second, _ := json.Marshal(strip(t, p))
	if string(first) != string(second) {
		t.Error("two runs of the same check differ")
	}
	if _, err := os.Stat(filepath.Join(p.RunDir(), "stale.txt")); err == nil {
		t.Error("the run folder kept a file from the run before")
	}
}

// R-CHECK-12: a target's stub generator gets every contract as JSON; if it fails, the target is broken.
func TestStubs(t *testing.T) {
	rep, p := run(t, map[string]T{"a": {Extra: "stubs: 'exec FAKE --stubs contracts.json'\n"}, "b": {Extra: "stubs: 'exec FAKE --fail-stubs'\n"}})
	data, err := os.ReadFile(filepath.Join(p.TargetsDir(), "a", "contracts.json"))
	var got []map[string]any
	if err != nil || json.Unmarshal(data, &got) != nil || len(got) != 2 || got[0]["id"] != "ui.counter" {
		t.Fatalf("stubs input: %v %s", err, data)
	}
	if rep.Summary.Broken != 7 || !strings.Contains(rep.Targets[1].Broken, "stub generator") {
		t.Fatalf("%+v", rep.Targets)
	}
}

// R-CHECK-13: checking one component still gives the stub generator every contract.
func TestStubsGetAllContracts(t *testing.T) {
	root := testutil.Project(t, map[string]T{"a": {Extra: "stubs: 'exec FAKE --stubs contracts.json'\n"}})
	p := project.Project{Root: root}
	cs, _, _ := contract.LoadDir(p.Contracts())
	ts, _ := p.Targets()
	if _, err := Run(Options{Project: p, Contracts: cs[1:], All: cs, Targets: ts}); err != nil {
		t.Fatal(err)
	}
	data, _ := os.ReadFile(filepath.Join(root, "targets", "a", "contracts.json"))
	var got []map[string]any
	if json.Unmarshal(data, &got) != nil || len(got) != 2 {
		t.Fatalf("the stub generator got %d contracts: %s", len(got), data)
	}
}

// R-LOG-1: every request is linked to its run, target, contract and test in omnicui.log, and the
// log holds no props or text values.
func TestLogFields(t *testing.T) {
	_, p := run(t, map[string]T{"a": {}})
	data, _ := os.ReadFile(filepath.Join(p.RunDir(), "omnicui.log"))
	lines := strings.Split(strings.TrimSpace(string(data)), "\n")
	var calls int
	for _, l := range lines {
		var m map[string]any
		if err := json.Unmarshal([]byte(l), &m); err != nil {
			t.Fatalf("not JSON: %s", l)
		}
		if m["run"] == nil || m["level"] == nil || m["msg"] == nil {
			t.Fatalf("missing fields: %s", l)
		}
		if m["msg"] == "call" {
			calls++
			if m["id"] == nil || m["method"] == nil || m["target"] == nil {
				t.Fatalf("call without id/method/target: %s", l)
			}
			if m["method"] != "advertise" && (m["contract"] == nil || m["test"] == nil || m["combo"] == nil) {
				t.Fatalf("call not linked to its test: %s", l)
			}
			if strings.Count(l, `"target":`) != 1 {
				t.Fatalf("target logged twice: %s", l)
			}
		}
		if strings.Contains(l, "Dark mode") || strings.Contains(l, "Wi-Fi") {
			t.Fatalf("a text value is in the log: %s", l)
		}
	}
	if calls == 0 {
		t.Fatal("no calls logged")
	}
}

func TestCollapse(t *testing.T) {
	cbs := []Combo{{Scheme: "light"}, {Scheme: "dark"}}
	got := collapse([]string{"x (0×0 light)", "x (0×0 dark)", "y (0×0 dark)"}, cbs)
	if strings.Join(got, "|") != "x|y (0×0 dark)" {
		t.Errorf("%q", got)
	}
}
