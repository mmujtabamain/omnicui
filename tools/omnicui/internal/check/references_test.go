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

// runRoot runs a check on a project folder made by testutil.Project.
func runRoot(t *testing.T, root string) *Report {
	t.Helper()
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

func references(rep *Report) map[string]Comparison {
	out := map[string]Comparison{}
	for _, c := range rep.Comparisons {
		if c.Kind == "reference" && c.Test == "starts at the start value" {
			out[c.B] = c
		}
	}
	return out
}

// R-REFERENCE-1: a recorded screenshot listed in references.yaml is compared with each target's
// screenshot of the test at its size and colour scheme, whatever the target's design: the same
// picture passes, a shifted one warns, a mask over the difference passes again, and a reference
// no target took a screenshot of its size for is reported.
func TestReferences(t *testing.T) {
	root := testutil.Project(t, map[string]T{"a": {}, "b": {Env: map[string]string{"FAKE_SHIFT": "12"}, Extra: "design: other\n"}})
	rep := runRoot(t, root)
	// The reference: what target a drew, as if recorded from the app.
	var shot string
	for _, r := range rep.Results {
		if r.Target == "a" && r.Test == "starts at the start value" {
			shot = r.Screenshots["320x240-light"]
		}
	}
	data, err := os.ReadFile(filepath.Join(project.Project{Root: root}.RunDir(), shot))
	if err != nil {
		t.Fatal(err)
	}
	os.WriteFile(filepath.Join(root, "ref.png"), data, 0o644)
	write := func(extra string) {
		os.WriteFile(filepath.Join(root, "references.yaml"), []byte(`- contract: ui.counter
  test: starts at the start value
  image: ref.png
`+extra), 0o644)
	}
	write("")
	refs := references(runRoot(t, root))
	if a := refs["a"]; a.Warning || a.Measures == nil || a.Measures.Area != 0 {
		t.Fatalf("a: %+v", a)
	}
	// b is another design: no pixels between a and b, but the reference is compared with both.
	if b := refs["b"]; !b.Warning || !strings.Contains(b.Note, "against ref.png") || b.Diff == "" {
		t.Fatalf("b: %+v", b)
	}
	write("  mask: [{ x: 0, y: 0, width: 320, height: 240 }]\n")
	if b := references(runRoot(t, root))["b"]; b.Warning {
		t.Fatalf("masked: %+v", b)
	}
	write("  crop: { x: 0, y: 0, width: 100, height: 100 }\n")
	if r := references(runRoot(t, root))[""]; !r.Warning || !strings.Contains(r.Note, "no target took a 100×100 light screenshot") {
		t.Fatalf("no size: %+v", r)
	}
}

// R-REFERENCE-2: an entry without an image that exists is refused when the run starts.
func TestReferencesMissingImage(t *testing.T) {
	root := testutil.Project(t, map[string]T{"a": {}})
	os.WriteFile(filepath.Join(root, "references.yaml"), []byte("- { contract: ui.counter, test: starts at the start value, image: nope.png }\n"), 0o644)
	p := project.Project{Root: root}
	cs, _, _ := contract.LoadDir(p.Contracts())
	ts, _ := p.Targets()
	if _, err := Run(Options{Project: p, Contracts: cs[:1], Targets: ts}); err == nil || !strings.Contains(err.Error(), "no image nope.png") {
		t.Fatalf("%v", err)
	}
}
