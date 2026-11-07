package check

import (
	"bytes"
	"os"
	"path/filepath"
	"regexp"
	"strings"
	"testing"

	"omnicui/internal/contract"
	"omnicui/internal/project"
	"omnicui/internal/testutil"
)

// R-PROBE-1: the render check passes for a target that draws its picture, and the run says so.
func TestProbePasses(t *testing.T) {
	rep, _ := run(t, map[string]T{"a": {}})
	if rep.Targets[0].Probe != "passed" {
		t.Fatalf("probe: %q", rep.Targets[0].Probe)
	}
}

// R-PROBE-2: a target whose picture is cut off or blank keeps running its behaviour tests, but
// its screenshots are off for the run, with the reason, and --require target:screenshot fails.
func TestProbeFails(t *testing.T) {
	for setting, want := range map[string]string{"small": "not the 320×240 asked for", "blank": "black square at the top left is missing"} {
		rep, _ := run(t, map[string]T{"a": {}, "b": {Env: map[string]string{"FAKE_PROBE": setting}}}, "b:screenshot")
		b := rep.Targets[1]
		if b.Name != "b" || !strings.Contains(b.Probe, want) || !strings.Contains(b.Cannot["screenshot"], "the render check failed") {
			t.Fatalf("%s: %+v", setting, b)
		}
		for _, r := range rep.Results {
			if r.Target == "b" && (r.Status != Pass || len(r.Screenshots) > 0) {
				t.Errorf("%s: %+v", setting, r)
			}
		}
		if !rep.Failed() {
			t.Errorf("%s: --require b:screenshot should fail the run", setting)
		}
		var out bytes.Buffer
		Print(&out, rep, t.TempDir())
		if !strings.Contains(out.String(), "b: render check failed: ") {
			t.Errorf("%s: %s", setting, out.String())
		}
	}
}

// R-PROBE-3: a target that doesn't advertise probe is not checked (older adapters keep working).
func TestProbeNotAdvertised(t *testing.T) {
	rep, _ := run(t, map[string]T{"a": {Env: map[string]string{"FAKE_CAN": "mount,press,focus,key,type,read,events,screenshot,trace,record"}}})
	if rep.Targets[0].Probe != "not advertised" || rep.Failed() {
		t.Fatalf("%+v", rep.Targets[0])
	}
}

// R-VISIBLE-1: a part whose text passed while it had no size on screen is a visibility warning.
func TestInvisibleReadWarns(t *testing.T) {
	rep, _ := run(t, map[string]T{"a": {}, "b": {Env: map[string]string{"FAKE_HIDDEN": "value"}}})
	ws := warnings(rep, "visibility")
	if len(ws) == 0 || rep.Failed() {
		t.Fatalf("%+v", rep.Comparisons)
	}
	for _, w := range ws {
		if w.A != "b" || !strings.Contains(w.Note, "`value` was read but has no size on screen (0×0)") {
			t.Errorf("%+v", w)
		}
	}
	if n, _ := rep.WarningsFailing([]string{"visibility"}); n != len(ws) {
		t.Errorf("--fail-on visibility: %d", n)
	}
}

// R-FAILON-1: --fail-on turns the named kind of warning into a failed run, and only that kind.
func TestFailOn(t *testing.T) {
	rep, _ := run(t, map[string]T{"a": {}, "b": {Env: map[string]string{"FAKE_COLOR": "c0392b", "FAKE_SHIFT": "8"}}})
	if n, _ := rep.WarningsFailing(nil); n != 0 {
		t.Error("no --fail-on: warnings don't fail")
	}
	if n, _ := rep.WarningsFailing([]string{"timing"}); n != 0 {
		t.Error("--fail-on timing: there are no timing warnings")
	}
	if n, _ := rep.WarningsFailing([]string{"pixels"}); n != len(warnings(rep, "pixels")) || n == 0 {
		t.Errorf("--fail-on pixels: %d", n)
	}
	if n, _ := rep.WarningsFailing([]string{"warnings"}); n != rep.Summary.Warnings {
		t.Errorf("--fail-on warnings: %d of %d", n, rep.Summary.Warnings)
	}
}

// R-PROGRESS-1: one line per finished test, numbered per target.
func TestProgress(t *testing.T) {
	root := testutil.Project(t, map[string]T{"a": {}})
	p := project.Project{Root: root}
	cs, _, _ := contract.LoadDir(p.Contracts())
	ts, _ := p.Targets()
	var progress bytes.Buffer
	if _, err := Run(Options{Project: p, Contracts: cs, Targets: ts, Progress: &progress}); err != nil {
		t.Fatal(err)
	}
	lines := strings.Split(strings.TrimSpace(progress.String()), "\n")
	total := 0
	for _, c := range cs {
		total += len(c.Tests)
	}
	if len(lines) != total || !strings.HasPrefix(lines[0], "[a 1/") || !regexp.MustCompile(`  pass  \d+\.\ds$`).MatchString(lines[0]) || !strings.Contains(lines[0], "counter › ") {
		t.Fatalf("%q", lines)
	}
}

// R-PRINT-1: a message that many tests of one target share is printed once, with how many.
func TestRepeatedMessagesPrintedOnce(t *testing.T) {
	rep, _ := run(t, map[string]T{"a": {}, "b": {Env: map[string]string{"FAKE_ERROR_ON": "read"}}})
	var out bytes.Buffer
	Print(&out, rep, t.TempDir())
	s := out.String()
	if !strings.Contains(s, "b: ") || !strings.Contains(s, " tests: read ") || strings.Count(s, "deliberate error on read") != strings.Count(s, " tests: read ") {
		t.Fatalf("%s", s)
	}
}

// R-FIELDS-1: role and focused are read from the target and compared; a wrong role fails.
func TestRoleAndFocused(t *testing.T) {
	root := testutil.Project(t, map[string]T{"a": {}, "b": {Env: map[string]string{"FAKE_ROLE": "increment=generic"}}})
	contractYAML := `id: ui.counter
props:
  start: { type: integer, default: 0 }
events:
  changed:
    count: { type: integer }
parts:
  value:     { role: status }
  increment: { role: button }
tests:
  - name: the button is a focused button after a press
    steps:
      - { action: press, part: increment }
    expect:
      - { part: increment, role: button, focused: true, visible: true }
      - { part: value, text: "1", role: status }
`
	os.Remove(filepath.Join(root, "contracts", "switch.contract.yaml"))
	if err := os.WriteFile(filepath.Join(root, "contracts", "counter.contract.yaml"), []byte(contractYAML), 0o644); err != nil {
		t.Fatal(err)
	}
	p := project.Project{Root: root}
	cs, ps, err := contract.LoadDir(p.Contracts())
	if err != nil || len(cs) == 0 || contract.HasErrors(ps) {
		t.Fatal(err, ps)
	}
	ts, _ := p.Targets()
	rep, err := Run(Options{Project: p, Contracts: cs, Targets: ts})
	if err != nil {
		t.Fatal(err)
	}
	a, b := result(t, rep, "a", "the button is a focused button after a press"), result(t, rep, "b", "the button is a focused button after a press")
	if a.Status != Pass {
		t.Errorf("a: %+v", a)
	}
	if b.Status != Fail || len(b.Messages) != 1 || b.Messages[0] != `increment.role: expected "button", got "generic"` {
		t.Errorf("b: %+v", b)
	}
}
