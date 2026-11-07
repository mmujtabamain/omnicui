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

// runEdited runs a check on a project whose counter contract was changed by edit.
func runEdited(t *testing.T, targets map[string]T, edit func(string) string) *Report {
	t.Helper()
	root := testutil.Project(t, targets)
	path := filepath.Join(root, "contracts", "counter.contract.yaml")
	data, _ := os.ReadFile(path)
	os.WriteFile(path, []byte(edit(string(data))), 0o644)
	p := project.Project{Root: root}
	cs, ps, err := contract.LoadDir(p.Contracts())
	if err != nil || contract.HasErrors(ps) {
		t.Fatal(err, ps)
	}
	ts, _ := p.Targets()
	rep, err := Run(Options{Project: p, Contracts: cs[:1], Targets: ts})
	if err != nil {
		t.Fatal(err)
	}
	return rep
}

func pixelsOf(rep *Report, test string) *Comparison {
	for i, c := range rep.Comparisons {
		if c.Kind == "pixels" && c.Test == test {
			return &rep.Comparisons[i]
		}
	}
	return nil
}

// R-MASK-1: a mask leaves a part out of the comparison on both targets, even a part (optional)
// that only one target shows, such as window controls; the whole screenshots' measures stay in
// the comparison and its note.
func TestMaskOptionalPart(t *testing.T) {
	targets := map[string]T{"a": {Env: map[string]string{"FAKE_CONTROLS": "1"}}, "b": {}}
	optional := func(s string) string {
		return strings.Replace(s, "  increment: { role: button }", "  increment: { role: button }\n  controls:  { role: generic, optional: true }", 1)
	}
	rep := runEdited(t, targets, func(s string) string { return optional(s) + "mask: [controls]\n" })
	cmp := pixelsOf(rep, "starts at the start value")
	if cmp == nil || cmp.Warning || cmp.Measures.Area != 0 || cmp.Raw == nil || cmp.Raw.Area == 0 ||
		!strings.Contains(cmp.Note, "masked: controls (whole screenshots: area ") {
		t.Fatalf("%+v %+v", cmp, cmp.Raw)
	}
	// Without the mask, the controls are a difference. (The part must then be used elsewhere.)
	rep = runEdited(t, targets, func(s string) string {
		return strings.Replace(optional(s), "screenshot: true", "screenshot: { mask: [] }", 1) + "mask: [value]\n"
	})
	if cmp := pixelsOf(rep, "starts at the start value"); cmp == nil || cmp.Measures.Area == 0 {
		t.Fatalf("%+v", cmp)
	}
	// A test's own mask works the same way.
	rep = runEdited(t, targets, func(s string) string {
		return strings.Replace(optional(s), "screenshot: true", "screenshot: { mask: [controls] }", 1)
	})
	if cmp := pixelsOf(rep, "starts at the start value"); cmp == nil || cmp.Measures.Area != 0 {
		t.Fatalf("%+v", cmp)
	}
}

// R-MASK-2: screenshot: {part: x} compares only that part's box on each target, wherever it is.
func TestScreenshotPart(t *testing.T) {
	targets := map[string]T{"a": {}, "b": {Env: map[string]string{"FAKE_SHIFT": "12"}}}
	rep := runEdited(t, targets, func(s string) string { return s })
	if cmp := pixelsOf(rep, "starts at the start value"); cmp == nil || cmp.Measures.Area == 0 {
		t.Fatalf("a shifted target should differ: %+v", cmp)
	}
	rep = runEdited(t, targets, func(s string) string {
		return strings.Replace(s, "screenshot: true", "screenshot: { part: value }", 1)
	})
	cmp := pixelsOf(rep, "starts at the start value")
	if cmp == nil || cmp.Measures.Area != 0 || !strings.Contains(cmp.Note, "only `value`") || cmp.Raw == nil {
		t.Fatalf("%+v", cmp)
	}
}

// R-MASK-3: masks and screenshot parts name declared parts; masking the part compared is an error.
func TestMaskContractErrors(t *testing.T) {
	dir := t.TempDir()
	write := func(body string) []contract.Problem {
		os.WriteFile(filepath.Join(dir, "x.contract.yaml"), []byte(body), 0o644)
		_, ps, _ := contract.LoadDir(dir)
		return ps
	}
	base := "id: ui.x\nparts: { a: { role: status }, b: { role: status, optional: true } }\ntests:\n  - name: t\n    expect: [ { part: a, text: x } ]\n    screenshot: %s\n"
	ps := write(strings.Replace(base, "%s", "{ mask: [c] }", 1))
	if !strings.Contains(problems(ps), "part `c` does not exist") {
		t.Fatal(problems(ps))
	}
	ps = write(strings.Replace(base, "%s", "{ part: a, mask: [a] }", 1))
	if !strings.Contains(problems(ps), "compares only `a` and masks it too") {
		t.Fatal(problems(ps))
	}
	ps = write(strings.Replace(base, "%s", "{ part: a, mask: [b] }", 1))
	if contract.HasErrors(ps) || strings.Contains(problems(ps), "`b` is never used") {
		t.Fatal(problems(ps))
	}
}

func problems(ps []contract.Problem) string {
	var out []string
	for _, p := range ps {
		out = append(out, p.Message)
	}
	return strings.Join(out, "\n")
}
