package check

import (
	"bytes"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"omnicui/internal/contract"
	"omnicui/internal/project"
	"omnicui/internal/testutil"
)

// Designs: the same behaviour, a different UI. These tests come from experiments/redesign, where
// a second design of the web counter and switch (a card, a native checkbox) was run against the
// same contracts.

// runContract runs a check of one contract (by file name), edited by edit, on targets.
func runContract(t *testing.T, targets map[string]T, file string, edit func(string) string) (*Report, project.Project) {
	t.Helper()
	root := testutil.Project(t, targets)
	path := filepath.Join(root, "contracts", file)
	data, _ := os.ReadFile(path)
	os.WriteFile(path, []byte(edit(string(data))), 0o644)
	p := project.Project{Root: root}
	cs, ps, err := contract.LoadDir(p.Contracts())
	if err != nil || contract.HasErrors(ps) {
		t.Fatal(err, ps)
	}
	var mine []*contract.Contract
	for _, c := range cs {
		if filepath.Base(c.File) == file {
			mine = append(mine, c)
		}
	}
	ts, _ := p.Targets()
	rep, err := Run(Options{Project: p, Contracts: mine, Targets: ts})
	if err != nil {
		t.Fatal(err)
	}
	return rep, p
}

const pressTest = "a press turns it on, and the thumb slides across"

func optionalThumb(s string) string {
	return strings.Replace(s, "  thumb:  { role: generic }", "  thumb:  { role: generic, optional: true }", 1)
}

// R-DESIGN-1: a part one design has and another doesn't (a switch's thumb, which a checkbox
// lacks) is `optional: true`. Where a target has no such part, its trace, its clip and what a
// test expects of it are skipped, with a note, and the rest of the test still counts; the steps
// the trace ran are run again on a fresh component, so they act once. A test with nothing left
// to check there is not measured. Without `optional`, the failure says how to mark it.
func TestOptionalPartAbsent(t *testing.T) {
	targets := map[string]T{"a": {}, "b": {Env: map[string]string{"FAKE_NO_PARTS": "thumb"}}}
	rep, p := runContract(t, targets, "switch.contract.yaml", func(s string) string {
		s = optionalThumb(s)
		s = strings.Replace(s, "    record: true", "    record: { part: thumb }", 1)
		return strings.Replace(s, "\nkeyboard:", "\n  - name: the thumb is shown\n    expect:\n      - { part: thumb, visible: true }\n\nkeyboard:", 1)
	})
	r := result(t, rep, "b", pressTest)
	if r.Status != Pass || len(r.Traces) != 0 || len(r.Clips) != 0 ||
		!contains(r.Absent, "no `thumb` in b's design (an optional part): its trace is skipped") ||
		!contains(r.Absent, "no `thumb` in b's design (an optional part): its clip is skipped") {
		t.Fatalf("%+v", r)
	}
	if a := result(t, rep, "a", pressTest); a.Status != Pass || len(a.Traces) != 1 || len(a.Absent) != 0 {
		t.Fatalf("%+v", a)
	}
	if r := result(t, rep, "b", "the thumb is shown"); r.Status != NotMeasured ||
		!contains(r.Absent, "no `thumb` in b's design (an optional part): what the test expects of it is skipped") {
		t.Fatalf("%+v", r)
	}
	if r := result(t, rep, "a", "the thumb is shown"); r.Status != Pass {
		t.Fatalf("%+v", r)
	}
	// Only a's trace is compared, with the named timing; nothing between a and b.
	for _, c := range rep.Comparisons {
		if c.Kind == "timing" && (c.A == "b" || c.B == "b") {
			t.Errorf("b has no thumb to compare: %+v", c)
		}
	}
	var out bytes.Buffer
	Print(&out, rep, p.RunDir())
	if !strings.Contains(out.String(), "b: no `thumb` in b's design (an optional part): its trace is skipped") {
		t.Fatal(out.String())
	}

	// Without optional: the trace fails, and says both ways out.
	rep, _ = runContract(t, targets, "switch.contract.yaml", func(s string) string { return s })
	r = result(t, rep, "b", pressTest)
	if r.Status != Fail || !strings.Contains(strings.Join(r.Messages, "\n"), "if b's design has no `thumb`, mark the part `optional: true` in the contract") {
		t.Fatalf("%+v", r)
	}
}

// R-DESIGN-2: a test whose steps act on an optional part a target doesn't have is not measured
// there, as is a generated keyboard test on it.
func TestOptionalPartSteps(t *testing.T) {
	targets := map[string]T{"a": {}, "b": {Env: map[string]string{"FAKE_NO_PARTS": "increment"}}}
	rep, _ := runContract(t, targets, "counter.contract.yaml", func(s string) string {
		return strings.Replace(s, "  increment: { role: button }", "  increment: { role: button, optional: true }", 1)
	})
	r := result(t, rep, "b", "each press adds one")
	if r.Status != NotMeasured || !contains(r.Absent, "no `increment` in b's design (an optional part): the test acts on it, so there is nothing to measure here") {
		t.Fatalf("%+v", r)
	}
	if r := result(t, rep, "b", "keyboard: Enter on `increment` does what a press does"); r.Status != NotMeasured ||
		!strings.Contains(strings.Join(r.Messages, ""), "no `increment` in b's design (an optional part)") {
		t.Fatalf("%+v", r)
	}
	if r := result(t, rep, "a", "each press adds one"); r.Status != Pass {
		t.Fatalf("%+v", r)
	}
	if rep.Failed() {
		t.Fatalf("not measured is not a failure: %+v", rep.Summary)
	}
}

// R-DESIGN-3: targets of different designs (target.yaml `design:`) are compared by their
// palettes, not pixel by pixel: the same colours in another layout pass, other colours warn and
// say which. Targets of one design are still compared pixel by pixel.
func TestDesigns(t *testing.T) {
	redesign := "design: redesign\n"
	// Another layout (moved 100 px), the same colours.
	rep, p := run(t, map[string]T{"a": {}, "b": {Env: map[string]string{"FAKE_SHIFT": "100"}, Extra: redesign}})
	pal := 0
	for _, c := range rep.Comparisons {
		switch c.Kind {
		case "pixels":
			t.Errorf("no pixels between designs: %+v", c)
		case "palette":
			pal++
			if c.Warning || c.Value != 0 || c.Palette == nil || !strings.Contains(c.Note, "a is the default design and b design redesign, so their colours are compared, not their pixels: the same colours") {
				t.Errorf("%+v", c)
			}
		}
	}
	if pal != 5 {
		t.Errorf("%d palette comparisons, want one per screenshot test", pal)
	}
	if rep.Targets[1].Design != "redesign" && rep.Targets[0].Design != "redesign" {
		t.Errorf("%+v", rep.Targets)
	}
	var out bytes.Buffer
	Print(&out, rep, p.RunDir())
	if !strings.Contains(out.String(), "designs: default: a · redesign: b (pixels compared within a design, colours across designs)") {
		t.Fatal(out.String())
	}

	// Other colours: a warning that names them.
	rep, p = run(t, map[string]T{"a": {}, "b": {Env: map[string]string{"FAKE_SHIFT": "100", "FAKE_COLOR": "7b2cbf"}, Extra: redesign}})
	var warned *Comparison
	for i, c := range rep.Comparisons {
		if c.Kind == "palette" && c.Test == "starts at the start value" {
			warned = &rep.Comparisons[i]
		}
	}
	if warned == nil || !warned.Warning || warned.Value != 100 || len(warned.Palette.OnlyB) != 1 || warned.Palette.OnlyB[0].Hex != "#7b2cbf" {
		t.Fatalf("%+v", warned)
	}
	if n, _ := rep.WarningsFailing([]string{"palette"}); n == 0 {
		t.Error("--fail-on palette")
	}
	out.Reset()
	Print(&out, rep, p.RunDir())
	if !strings.Contains(out.String(), "colours (other designs)") || !strings.Contains(out.String(), "b uses #7b2cbf (100%), which a doesn't use") {
		t.Fatal(out.String())
	}

	// The same layout change within one design is a pixel warning, as before.
	rep, _ = run(t, map[string]T{"a": {}, "b": {Env: map[string]string{"FAKE_SHIFT": "100"}}})
	if c := pixelsOf(rep, "starts at the start value"); c == nil || !c.Warning {
		t.Fatalf("%+v", c)
	}
}

// R-DESIGN-4: `text` and `name` can be looser: { contains: … } or { matches: <regular
// expression> }. Exact text still means exact.
func TestTextMatch(t *testing.T) {
	targets := map[string]T{"a": {Env: map[string]string{"FAKE_TEXT_SUFFIX": " taps"}}}
	rep, _ := runContract(t, targets, "counter.contract.yaml", func(s string) string { return s })
	if r := result(t, rep, "a", "each press adds one"); r.Status != Fail || !contains(r.Messages, `value.text: expected "3", got "3 taps"`) {
		t.Fatalf("%+v", r)
	}
	rep, _ = runContract(t, targets, "counter.contract.yaml", func(s string) string {
		s = strings.Replace(s, `{ part: value, text: "3" }`, `{ part: value, text: { contains: "3" } }`, 1)
		s = strings.Replace(s, `{ part: value, text: "5" }`, `{ part: value, text: { matches: "^5( taps)?$" } }`, 1)
		return strings.Replace(s, `{ part: value, text: "1" }`, `{ part: value, text: { matches: "^4" }, name: { contains: "" } }`, 1)
	})
	for test, want := range map[string]Status{"each press adds one": Pass, "starts at the start value": Pass, "the keyboard works too": Fail} {
		if r := result(t, rep, "a", test); r.Status != want {
			t.Errorf("%s: %+v", test, r)
		}
	}
	if r := result(t, rep, "a", "the keyboard works too"); !contains(r.Messages, `value.text: expected text matching /^4/, got "1 taps"`) {
		t.Fatalf("%+v", r)
	}
}

// R-DESIGN-5: a regular expression that doesn't compile is a contract error, at its line.
func TestTextMatchContractError(t *testing.T) {
	root := testutil.Project(t, map[string]T{})
	path := filepath.Join(root, "contracts", "counter.contract.yaml")
	data, _ := os.ReadFile(path)
	os.WriteFile(path, []byte(strings.Replace(string(data), `{ part: value, text: "3" }`, `{ part: value, text: { matches: "(" } }`, 1)), 0o644)
	_, ps, err := contract.LoadDir(filepath.Join(root, "contracts"))
	if err != nil || !contract.HasErrors(ps) {
		t.Fatal(err, ps)
	}
	found := false
	for _, p := range ps {
		found = found || strings.Contains(p.Message, "`(` is not a regular expression")
	}
	if !found {
		t.Fatalf("%+v", ps)
	}
}

// R-DESIGN-6: two targets that move a traced point in different directions (a thumb sliding
// down against one sliding across) warn, whatever their timing.
func TestDirection(t *testing.T) {
	rep, p := run(t, map[string]T{"a": {}, "b": {Env: map[string]string{"FAKE_VERTICAL": "1"}}})
	var dir, tim *Comparison
	for i, c := range rep.Comparisons {
		if c.Test == pressTest && c.B == "b" {
			switch c.Kind {
			case "direction":
				dir = &rep.Comparisons[i]
			case "timing":
				tim = &rep.Comparisons[i]
			}
		}
	}
	if dir == nil || !dir.Warning || dir.Value != 90 || dir.Note != "a went right 20 px, b down 20 px" {
		t.Fatalf("%+v", dir)
	}
	if tim == nil || tim.Warning {
		t.Fatalf("the timing is the same: %+v", tim)
	}
	var out bytes.Buffer
	Print(&out, rep, p.RunDir())
	if !strings.Contains(out.String(), "direction thumb") || !strings.Contains(out.String(), "a ↔ b 90°") || !strings.Contains(out.String(), "a went right 20 px, b down 20 px") {
		t.Fatal(out.String())
	}
}
