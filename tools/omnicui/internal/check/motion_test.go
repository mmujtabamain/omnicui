package check

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"omnicui/internal/contract"
	"omnicui/internal/project"
	"omnicui/internal/testutil"
)

// runWith runs fake targets against one contract of the test's own.
func runWith(t *testing.T, targets map[string]T, name, yaml string) (*Report, project.Project) {
	t.Helper()
	root := testutil.Project(t, targets)
	files, _ := filepath.Glob(filepath.Join(root, "contracts", "*.contract.yaml"))
	for _, f := range files {
		os.Remove(f)
	}
	if err := os.WriteFile(filepath.Join(root, "contracts", name+".contract.yaml"), []byte(yaml), 0o644); err != nil {
		t.Fatal(err)
	}
	p := project.Project{Root: root}
	cs, ps, err := contract.LoadDir(p.Contracts())
	if err != nil || contract.HasErrors(ps) {
		t.Fatal(err, ps)
	}
	ts, _ := p.Targets()
	rep, err := Run(Options{Project: p, Contracts: cs, Targets: ts})
	if err != nil {
		t.Fatal(err)
	}
	return rep, p
}

const switchHead = `id: ui.switch
props:
  on:    { type: boolean, default: false }
  label: { type: string, default: "Dark mode" }
events:
  toggled:
    on: { type: boolean }
parts:
  switch: { role: switch }
  thumb:  { role: generic }
`

func kind(rep *Report, k string) []Comparison {
	var out []Comparison
	for _, c := range rep.Comparisons {
		if c.Kind == k {
			out = append(out, c)
		}
	}
	return out
}

// R-MOTION-7: clips are compared by their visual progress. A thumb that slides 450 ms linearly
// on one target and 200 ms ease-out on another is a motion warning, though nothing was traced.
func TestClipMotionWarning(t *testing.T) {
	yaml := switchHead + `tests:
  - name: the thumb slides
    steps: [ { action: press, part: switch } ]
    expect: [ { part: switch, checked: true } ]
    record: true
`
	rep, p := runWith(t, map[string]T{"a": {}, "b": {Env: map[string]string{"FAKE_EASING": "linear", "FAKE_DURATION": "450"}}}, "switch", yaml)
	ms := kind(rep, "motion")
	if len(ms) != 1 || !ms[0].Warning || ms[0].Value <= 40 || !strings.Contains(ms[0].Note, "of the change") {
		t.Fatalf("%+v", ms)
	}
	r := result(t, rep, "a", "the thumb slides")
	clip := r.Clips["320x240-light"]
	if clip.File == "" || clip.Progress == nil || len(clip.Inputs) != 1 || clip.Inputs[0] != 50 || len(clip.Values) != len(clip.Frames) {
		t.Fatalf("clip: %+v", clip)
	}
	if !strings.HasSuffix(clip.File, ".png") {
		t.Errorf("clips are animated PNGs: %s", clip.File)
	}
	html, _ := os.ReadFile(filepath.Join(p.RunDir(), "report.html"))
	for _, want := range []string{`class="player"`, "Visual progress of each target", "lined up at its first input", `<select class="speed">`} {
		if !strings.Contains(string(html), want) {
			t.Errorf("report.html should contain %q", want)
		}
	}
	// The same easing on both: no warning.
	rep, _ = runWith(t, map[string]T{"a": {}, "b": {}}, "switch", yaml)
	if ms := kind(rep, "motion"); len(ms) != 1 || ms[0].Warning {
		t.Errorf("same motion: %+v", ms)
	}
}

// R-MOTION-8: an adapter that replies to record with the frames alone (protocol 1 as first
// written) still works; its clip has no input times, and the player lines it up by its change.
func TestOldRecordReply(t *testing.T) {
	yaml := switchHead + `tests:
  - name: the thumb slides
    steps: [ { action: press, part: switch } ]
    expect: [ { part: switch, checked: true } ]
    record: true
`
	rep, _ := runWith(t, map[string]T{"a": {Env: map[string]string{"FAKE_BUG": "old-record"}}, "b": {}}, "switch", yaml)
	clip := result(t, rep, "a", "the thumb slides").Clips["320x240-light"]
	if len(clip.Frames) == 0 || len(clip.Inputs) != 0 || clip.Progress == nil {
		t.Fatalf("%+v", clip)
	}
	if ms := kind(rep, "motion"); len(ms) != 1 || ms[0].Warning {
		t.Errorf("%+v", ms)
	}
}

// R-MOTION-9: an opacity trace is compared like a position trace, against the other targets
// and the named timing; a target without trace.opacity is "not advertised" for it.
func TestOpacityTrace(t *testing.T) {
	yaml := switchHead + `tests:
  - name: the thumb dims as it slides
    steps: [ { action: press, part: switch } ]
    expect: [ { part: switch, checked: true } ]
    trace:
      - { part: thumb, property: opacity, timing: { duration: 200ms, easing: ease-out } }
`
	rep, _ := runWith(t, map[string]T{"a": {}, "b": {Env: map[string]string{"FAKE_EASING": "linear", "FAKE_DURATION": "300"}},
		"c": {Env: map[string]string{"FAKE_CAN": "mount,press,focus,key,read,events,trace"}}}, "switch", yaml)
	var aTiming, bTiming *Comparison
	for i, c := range kind(rep, "timing") {
		if c.Part != "thumb opacity" {
			t.Errorf("label: %+v", c)
		}
		if c.B == "timing" && c.A == "a" {
			aTiming = &kind(rep, "timing")[i]
		}
		if c.B == "timing" && c.A == "b" {
			bTiming = &kind(rep, "timing")[i]
		}
	}
	if aTiming == nil || aTiming.Warning || bTiming == nil || !bTiming.Warning {
		t.Fatalf("a matches ease-out, b doesn't: %+v", kind(rep, "timing"))
	}
	if r := result(t, rep, "c", "the thumb dims as it slides"); r.Missing["trace.opacity"] == "" {
		t.Errorf("c: %+v", r)
	}
}

// R-MOTION-10: allCombinations takes traces and clips at every viewport × colour scheme, and
// compares each combination on its own.
func TestAllCombinations(t *testing.T) {
	yaml := switchHead + `tests:
  - name: the thumb slides
    steps: [ { action: press, part: switch } ]
    expect: [ { part: switch, checked: true } ]
    trace: [ { part: thumb, point: center } ]
    record: true
    allCombinations: true
viewports: [ { width: 320, height: 240 }, { width: 400, height: 300 } ]
colorSchemes: [ light, dark ]
`
	rep, _ := runWith(t, map[string]T{"a": {}, "b": {}}, "switch", yaml)
	r := result(t, rep, "a", "the thumb slides")
	if len(r.Traces) != 4 || len(r.Clips) != 4 {
		t.Fatalf("traces %d, clips %d: %+v", len(r.Traces), len(r.Clips), r)
	}
	combos := map[string]bool{}
	for _, c := range kind(rep, "timing") {
		combos[c.Combo] = true
	}
	if len(combos) != 4 || len(kind(rep, "motion")) != 4 {
		t.Errorf("timing at %v, %d motion", combos, len(kind(rep, "motion")))
	}
}

// R-VIEWPORT-1: a viewport's props reach every mount at that viewport; a test's own props win.
func TestViewportProps(t *testing.T) {
	yaml := switchHead + `tests:
  - name: named by the viewport
    expect: [ { part: switch, name: "Small" } ]
  - name: named by the test
    props: { label: "Test" }
    expect: [ { part: switch, name: "Test" } ]
viewports:
  - { width: 320, height: 240, props: { label: "Small" } }
`
	rep, _ := runWith(t, map[string]T{"a": {}}, "switch", yaml)
	if rep.Summary.Pass != 2 {
		t.Fatalf("%+v", rep.Results)
	}
}

// R-KEYS-1: keyboard: true adds a test per key a role is activated by; a key that doesn't do
// what a press does fails, saying how they differ.
func TestKeyboardChecks(t *testing.T) {
	yaml := `id: ui.counter
props:
  start: { type: integer, default: 0 }
events:
  changed:
    count: { type: integer }
parts:
  value:     { role: status }
  increment: { role: button }
tests:
  - name: starts at zero
    expect: [ { part: value, text: "0" } ]
keyboard: true
`
	rep, _ := runWith(t, map[string]T{"a": {}, "b": {Env: map[string]string{"FAKE_BUG": "no-enter"}}, "c": {Env: map[string]string{"FAKE_BUG": "space-twice"}}}, "counter", yaml)
	enter, space := "keyboard: Enter on `increment` does what a press does", "keyboard: Space on `increment` does what a press does"
	if result(t, rep, "a", enter).Status != Pass || result(t, rep, "a", space).Status != Pass {
		t.Errorf("a: %+v", rep.Results)
	}
	if r := result(t, rep, "b", enter); r.Status != Fail || len(r.Messages) != 1 ||
		r.Messages[0] != "Enter on `increment` didn't do what a press does: a press gave event changed {\"count\":1}, value.text \"1\"; the key gave value.text \"0\"" {
		t.Errorf("b: %+v", r)
	}
	if r := result(t, rep, "c", space); r.Status != Fail || !strings.Contains(r.Messages[0], "the key gave event changed {\"count\":2}") {
		t.Errorf("c: %+v", r)
	}
}
