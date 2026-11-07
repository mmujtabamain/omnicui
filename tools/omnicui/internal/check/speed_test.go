package check

import (
	"bytes"
	"os"
	"path/filepath"
	"sort"
	"strconv"
	"strings"
	"testing"
	"time"

	"omnicui/internal/contract"
	"omnicui/internal/project"
	"omnicui/internal/testutil"
)

// R-CHECK-20: `check --test` matches a part of a name, or a glob over the whole name, ignoring
// case; FilterTests keeps only those tests and drops contracts left with none.
func TestFilterTests(t *testing.T) {
	cases := []struct {
		name     string
		patterns []string
		want     bool
	}{
		{"each press adds one", []string{"PRESS"}, true},
		{"each press adds one", []string{"each*one"}, true},
		{"each press adds one", []string{"each*"}, true},
		{"each press adds one", []string{"*two"}, false},
		{"each press adds one", []string{"nothing", "adds"}, true},
		{"keyboard: Enter on `increment` does what a press does", []string{"keyboard:*"}, true},
	}
	for _, c := range cases {
		if got := TestMatches(c.name, c.patterns); got != c.want {
			t.Errorf("TestMatches(%q, %q) = %v", c.name, c.patterns, got)
		}
	}
	cs, _, err := contract.LoadDir(filepath.Join(testutil.ProjectRoot(), "contracts"))
	if err != nil {
		t.Fatal(err)
	}
	before := len(cs[0].Tests)
	got := FilterTests(cs, func(c *contract.Contract, tt contract.Test) bool { return TestMatches(tt.Name, []string{"each press"}) })
	if len(got) != 1 || len(got[0].Tests) != 1 || got[0].Tests[0].Name != "each press adds one" {
		t.Fatalf("%+v", got)
	}
	if len(cs[0].Tests) != before {
		t.Fatal("FilterTests changed the contract it was given")
	}
	if s := NearestTests(cs, []string{"press twice"}); !strings.Contains(s, "`each press adds one`") {
		t.Fatal(s)
	}
}

// R-CHECK-21: `check --failed` takes the tests that failed, broke or warned in the last run.
func TestLastProblems(t *testing.T) {
	dir := t.TempDir()
	path := filepath.Join(dir, "results.json")
	if _, err := LastProblems(path); err == nil || !strings.Contains(err.Error(), "run `omnicui check` first") {
		t.Fatal(err)
	}
	os.WriteFile(path, []byte(`{"results":[
		{"contract":"ui.counter","test":"a","target":"x","status":"pass"},
		{"contract":"ui.counter","test":"b","target":"x","status":"fail"},
		{"contract":"ui.switch","test":"c","target":"y","status":"broken"},
		{"contract":"ui.switch","test":"d","target":"y","status":"not advertised"}],
		"comparisons":[{"contract":"ui.switch","test":"e","warning":true},{"contract":"ui.switch","test":"a","warning":false}]}`), 0o644)
	got, err := LastProblems(path)
	if err != nil {
		t.Fatal(err)
	}
	if s := strings.Join(sortedTests(got), "; "); s != "ui.counter › b; ui.switch › c; ui.switch › e" {
		t.Fatal(s)
	}
}

// R-CHECK-22: each result says how long it took; a failing test keeps what the adapter wrote to
// standard error meanwhile, a passing one doesn't; the console lists the slowest tests.
func TestTimesAndStderr(t *testing.T) {
	rep, p := run(t, map[string]T{"a": {Env: map[string]string{"FAKE_BUG": "double", "FAKE_STDERR_ON": "press", "FAKE_SLEEP_MS": "40"}}})
	var failed, passed *TestResult
	for i := range rep.Results {
		r := &rep.Results[i]
		if r.Ms <= 0 {
			t.Fatalf("no time: %+v", r)
		}
		if r.Test == "each press adds one" {
			failed = r
		}
		if r.Test == "starts at the start value" {
			passed = r
		}
	}
	if failed == nil || failed.Status != Fail || len(failed.Stderr) == 0 || !strings.Contains(strings.Join(failed.Stderr, "\n"), "adapter note during press") {
		t.Fatalf("%+v", failed)
	}
	if passed == nil || passed.Status != Pass || len(passed.Stderr) != 0 {
		t.Fatalf("%+v", passed)
	}
	if rep.Targets[0].Ms <= 0 || rep.Targets[0].StartMs <= 0 || rep.Targets[0].StartMs > rep.Targets[0].Ms {
		t.Fatalf("%+v", rep.Targets[0])
	}
	var out bytes.Buffer
	Print(&out, rep, p.RunDir())
	if !strings.Contains(out.String(), "stderr | adapter note during press") {
		t.Fatal(out.String())
	}
	if !strings.Contains(out.String(), "  a took ") {
		t.Fatal(out.String())
	}
	for ms, want := range map[int64]string{400: "0.4s", 12300: "12s", 125000: "2m05s"} {
		if got := Seconds(ms); got != want {
			t.Errorf("Seconds(%d) = %s", ms, got)
		}
	}
	slow := Slowest(&Report{Results: []TestResult{{Test: "x", Ms: 1500}, {Test: "y", Ms: 900}, {Test: "z", Ms: 3000}}}, 5)
	if len(slow) != 2 || slow[0].Test != "z" || slow[1].Test != "x" {
		t.Fatalf("%+v", slow)
	}
}

// R-CHECK-23: while a target starts, progress shows a line every so often with the time, the
// limit and the adapter's last stderr line, then when it started.
func TestStartProgress(t *testing.T) {
	old := startTick
	startTick = 60 * time.Millisecond
	defer func() { startTick = old }()
	root := testutil.Project(t, map[string]T{"a": {Env: map[string]string{"FAKE_START_MS": "400"}}})
	p := project.Project{Root: root}
	cs, _, _ := contract.LoadDir(p.Contracts())
	ts, _ := p.Targets()
	var progress bytes.Buffer
	if _, err := Run(Options{Project: p, Contracts: cs[:1], Targets: ts, Progress: &progress}); err != nil {
		t.Fatal(err)
	}
	s := progress.String()
	if !strings.Contains(s, "[a] starting: ") || !strings.Contains(s, "of startTimeout 5s · adapter: compiling the harness") || !strings.Contains(s, "[a] started in ") {
		t.Fatal(s)
	}
}

// R-CHECK-24: an exclusive target runs with no other target; without it, targets run side by
// side. `--serial` (Options.Serial) makes every target exclusive.
func TestExclusive(t *testing.T) {
	overlap := func(extra string, serial bool) bool {
		spans := filepath.Join(t.TempDir(), "spans.txt")
		env := func(n string) map[string]string {
			return map[string]string{"FAKE_SPAN_FILE": spans, "FAKE_NAME": n, "FAKE_START_MS": "300"}
		}
		root := testutil.Project(t, map[string]T{"a": {Env: env("a"), Extra: extra}, "b": {Env: env("b")}})
		p := project.Project{Root: root}
		cs, _, _ := contract.LoadDir(p.Contracts())
		ts, err := p.Targets()
		if err != nil || len(ts) != 2 {
			t.Fatal(err, ts)
		}
		var progress bytes.Buffer
		var rep *Report
		rep, err = Run(Options{Project: p, Contracts: cs[:1], Targets: ts, Serial: serial, Progress: &progress})
		if err != nil {
			t.Fatal(err)
		}
		data, _ := os.ReadFile(spans)
		if len(strings.TrimSpace(string(data))) == 0 {
			t.Fatalf("no spans; progress:\n%s\n%+v", progress.String(), rep.Targets)
		}
		type span struct{ start, stop int64 }
		got := map[string]*span{}
		for _, l := range strings.Split(strings.TrimSpace(string(data)), "\n") {
			f := strings.Fields(l)
			n, _ := strconv.ParseInt(f[2], 10, 64)
			if got[f[0]] == nil {
				got[f[0]] = &span{}
			}
			if f[1] == "start" {
				got[f[0]].start = n
			} else {
				got[f[0]].stop = n
			}
		}
		if len(got) != 2 {
			t.Fatalf("%s", data)
		}
		s := []*span{got["a"], got["b"]}
		sort.Slice(s, func(i, j int) bool { return s[i].start < s[j].start })
		if extra != "" || serial {
			if !strings.Contains(progress.String(), "waiting") {
				t.Errorf("no waiting line: %s", progress.String())
			}
		}
		return s[1].start < s[0].stop
	}
	if !overlap("", false) {
		t.Error("targets didn't run side by side")
	}
	if overlap("exclusive: true\n", false) {
		t.Error("an exclusive target ran alongside another")
	}
	if overlap("", true) {
		t.Error("--serial ran targets side by side")
	}
}
