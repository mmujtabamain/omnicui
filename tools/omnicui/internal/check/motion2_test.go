package check

import (
	"strings"
	"testing"
)

// R-MOTION-14: property: scale traces a part's drawn scale; a spring (up and back) is a round
// trip, measured and compared; a target without trace.scale reports it not advertised.
func TestScaleTrace(t *testing.T) {
	yaml := `id: ui.switch
props:
  on: { type: boolean, default: false }
  label: { type: string, default: "Wi-Fi" }
events:
  toggled: { on: { type: boolean } }
parts:
  switch: { role: switch }
  thumb:  { role: generic }
tests:
  - name: the thumb springs
    steps: [ { action: press, part: switch } ]
    expect: [ { part: switch, checked: true } ]
    trace: [ { part: thumb, property: scale } ]
`
	rep, _ := runWith(t, map[string]T{"a": {}, "b": {}, "c": {Env: map[string]string{"FAKE_CANNOT": "trace.scale=no transforms here"}}}, "switch", yaml)
	var traced int
	for _, r := range rep.Results {
		if r.Target == "c" {
			if r.Missing["trace.scale"] != "no transforms here" {
				t.Errorf("c: %+v", r)
			}
			continue
		}
		for _, tr := range r.Traces {
			if tr.Property == "scale" && tr.Curve.Moved && tr.Curve.RoundTrip {
				traced++
			}
		}
	}
	if traced != 2 {
		t.Fatalf("%+v", rep.Results)
	}
	for _, c := range rep.Comparisons {
		if c.Kind == "timing" && c.A == "a" && c.B == "b" && (c.Warning || c.Value > 1) {
			t.Errorf("same spring on a and b: %+v", c)
		}
	}
}

// R-MOTION-15: record: { part: x } measures the clip over that part's box; a selected field is
// read from targets that advertise it.
func TestRecordPartAndSelected(t *testing.T) {
	yaml := `id: ui.counter
props:
  start: { type: integer, default: 0 }
events:
  changed: { count: { type: integer } }
parts:
  value:     { role: status }
  increment: { role: button }
tests:
  - name: chosen after a press
    steps: [ { action: press, part: increment } ]
    expect: [ { part: increment, selected: true } ]
`
	can := "mount,press,focus,key,type,drag,hover,read,events,screenshot,probe,bounds,role,focused,selected"
	rep, _ := runWith(t, map[string]T{"a": {Env: map[string]string{"FAKE_CAN": can}}, "b": {}}, "counter", yaml)
	if r := result(t, rep, "a", "chosen after a press"); r.Status != Pass {
		t.Errorf("a: %+v", r)
	}
	if r := result(t, rep, "b", "chosen after a press"); r.Status != NotAdvertised || !strings.Contains(strings.Join(r.Messages, " "), "`selected`") {
		t.Errorf("b: %+v", r)
	}
	yaml = `id: ui.switch
props:
  on: { type: boolean, default: false }
  label: { type: string, default: "Wi-Fi" }
events:
  toggled: { on: { type: boolean } }
parts:
  switch: { role: switch }
  thumb:  { role: generic }
tests:
  - name: the thumb slides
    steps: [ { action: press, part: switch } ]
    expect: [ { part: switch, checked: true } ]
    record: { part: thumb }
`
	rep, _ = runWith(t, map[string]T{"a": {}, "b": {}}, "switch", yaml)
	r := result(t, rep, "a", "the thumb slides")
	clip := r.Clips["320x240-light"]
	if clip.Progress == nil || !clip.Progress.Moved || clip.Note != "" {
		t.Fatalf("%+v", clip)
	}
}

// R-MOTION-16: a trace whose part the target can't find says that a part appearing during the
// steps should be looked up at each frame.
func TestTraceLatePart(t *testing.T) {
	rep, _ := run(t, map[string]T{"a": {Env: map[string]string{"FAKE_LATE_PART": "1"}}})
	for _, r := range rep.Results {
		if r.Test == "a press turns it on, and the thumb slides across" {
			if r.Status != Fail || !strings.Contains(strings.Join(r.Messages, " "), "should look it up at each frame") {
				t.Fatalf("%+v", r)
			}
			return
		}
	}
	t.Fatal("no traced test")
}
