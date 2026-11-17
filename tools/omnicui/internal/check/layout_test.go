package check

import (
	"strings"
	"testing"
)

// R-LAYOUT-2: leftOf, rightOf, above, below and middleWith are worked out from both parts'
// boxes, give or take a pixel, and a failure says by how much.
func TestLayoutRelations(t *testing.T) {
	layout := func(s string) string {
		return addTest(noKeyboard(s), `  - name: side by side
    expect:
      - { part: value, leftOf: increment, middleWith: increment }
      - { part: increment, rightOf: value }
`)
	}
	// value at 24,24 (60×30); increment beside it, its middle level (40, a pixel off: still level).
	level := map[string]string{"FAKE_BOUNDS": "value=24,24,60,30;increment=90,25,40,30"}
	rep := runEdited(t, map[string]T{"a": {Env: level}}, layout)
	if r := result(t, rep, "a", "side by side"); r.Status != Pass {
		t.Fatalf("%+v", r)
	}
	// The button 6 px lower and overlapping the value: two failures, each with its numbers.
	off := map[string]string{"FAKE_BOUNDS": "value=24,24,60,30;increment=80,30,40,30"}
	rep = runEdited(t, map[string]T{"a": {Env: off}}, layout)
	r := result(t, rep, "a", "side by side")
	msgs := strings.Join(r.Messages, "\n")
	for _, want := range []string{
		"value.leftOf: expected left of `increment`, but its right edge is at 84 and `increment` starts at 80",
		"value.middleWith: expected its middle level with `increment`, but it sits 6 px higher",
		"increment.rightOf: expected right of `value`, but it starts at 80 and `value` ends at 84",
	} {
		if !strings.Contains(msgs, want) {
			t.Errorf("missing %q in:\n%s", want, msgs)
		}
	}
	if r.Status != Fail {
		t.Errorf("status %v", r.Status)
	}
}

// R-LAYOUT-3: above and below.
func TestLayoutAboveBelow(t *testing.T) {
	stacked := func(s string) string {
		return addTest(noKeyboard(s), `  - name: stacked
    expect:
      - { part: value, above: increment }
      - { part: increment, below: value }
`)
	}
	rep := runEdited(t, map[string]T{"a": {Env: map[string]string{"FAKE_BOUNDS": "value=24,24,60,30;increment=24,54,60,30"}}}, stacked)
	if r := result(t, rep, "a", "stacked"); r.Status != Pass {
		t.Fatalf("%+v", r)
	}
	rep = runEdited(t, map[string]T{"a": {Env: map[string]string{"FAKE_BOUNDS": "value=24,40,60,30;increment=24,54,60,30"}}}, stacked)
	r := result(t, rep, "a", "stacked")
	if r.Status != Fail || !strings.Contains(strings.Join(r.Messages, "\n"), "value.above: expected above `increment`, but its bottom is at 70 and `increment` starts at 54") {
		t.Fatalf("%+v", r)
	}
}
