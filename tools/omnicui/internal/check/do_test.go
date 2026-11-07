package check

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"omnicui/internal/contract"
)

// R-DO-1: a `do` step is an action the targets define: omnicui passes its name, part and `with`
// on unchanged, and a target that doesn't advertise do.<name> reports the test not advertised,
// with its reason.
func TestDoStep(t *testing.T) {
	targets := map[string]T{
		"a": {Env: map[string]string{"FAKE_CAN": "mount,press,focus,key,type,drag,hover,read,events,screenshot,probe,bounds,do.bump"}},
		"b": {Env: map[string]string{"FAKE_CANNOT": "do.bump=touch only: no such gesture here"}},
	}
	rep := runEdited(t, targets, func(s string) string {
		return strings.Replace(s, "tests:\n", "tests:\n  - name: a target-defined bump\n    steps:\n      - { do: bump, part: increment, with: { by: 4 } }\n    expect:\n      - { part: value, text: \"4\" }\n\n", 1)
	})
	var a, b *TestResult
	for i := range rep.Results {
		r := &rep.Results[i]
		if r.Test == "a target-defined bump" {
			if r.Target == "a" {
				a = r
			} else {
				b = r
			}
		}
	}
	if a == nil || a.Status != Pass {
		t.Fatalf("%+v", a)
	}
	if b == nil || b.Status != NotAdvertised || b.Missing["do.bump"] != "touch only: no such gesture here" {
		t.Fatalf("%+v", b)
	}
}

// R-DO-2: a `do` step needs no part, may not also have an action, and its part, when given,
// must exist.
func TestDoStepContract(t *testing.T) {
	dir := t.TempDir()
	load := func(steps string) []contract.Problem {
		os.WriteFile(filepath.Join(dir, "x.contract.yaml"), []byte("id: ui.x\nparts: { a: { role: status } }\ntests:\n  - name: t\n    steps: [ "+steps+" ]\n    expect: [ { part: a, text: x } ]\n"), 0o644)
		_, ps, _ := contract.LoadDir(dir)
		return ps
	}
	if ps := load("{ do: back }"); contract.HasErrors(ps) {
		t.Fatal(problems(ps))
	}
	if ps := load("{ do: swipe, part: a, with: { direction: left, speed: 2 } }"); contract.HasErrors(ps) {
		t.Fatal(problems(ps))
	}
	if ps := load("{ do: swipe, part: b }"); !strings.Contains(problems(ps), "part `b` does not exist") {
		t.Fatal(problems(ps))
	}
	if ps := load("{ do: swipe, action: press, part: a }"); !contract.HasErrors(ps) {
		t.Fatal("a step with both do and action should be refused")
	}
	if ps := load("{ part: a }"); !contract.HasErrors(ps) {
		t.Fatal("a step with neither should be refused")
	}
}
