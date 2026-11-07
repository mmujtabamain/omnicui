package check

import (
	"bytes"
	"strings"
	"testing"
)

func keyboardResults(rep *Report, part string) map[string]TestResult {
	out := map[string]TestResult{}
	for _, r := range rep.Results {
		if strings.HasPrefix(r.Test, "keyboard: ") && strings.Contains(r.Test, "`"+part+"`") {
			out[strings.Fields(r.Test)[1]] = r
		}
	}
	return out
}

// R-KEY-10: a generated keyboard test whose part isn't on screen when the component mounts is
// "not measured", with what to do about it, not a failure.
func TestKeyboardPartNotShown(t *testing.T) {
	rep, p := run(t, map[string]T{"a": {Env: map[string]string{"FAKE_HIDDEN": "increment"}}})
	got := keyboardResults(rep, "increment")
	if len(got) != 2 {
		t.Fatalf("%+v", got)
	}
	for key, r := range got {
		if r.Status != NotMeasured || !strings.Contains(strings.Join(r.Messages, " "), "`increment` isn't on screen when ui.counter mounts") ||
			!strings.Contains(strings.Join(r.Messages, " "), "make that a component of its own") {
			t.Fatalf("%s: %+v", key, r)
		}
	}
	if rep.Summary.NotMeasured < 2 {
		t.Fatalf("%+v", rep.Summary)
	}
	var out bytes.Buffer
	Print(&out, rep, p.RunDir())
	if !strings.Contains(out.String(), " not measured · ") {
		t.Fatal(out.String())
	}
}

// R-KEY-11: a press that changes nothing readable leaves nothing to compare a key with: the
// generated test is "not measured", not a pass.
func TestKeyboardInertPress(t *testing.T) {
	rep, _ := run(t, map[string]T{"a": {Env: map[string]string{"FAKE_BUG": "inert"}}})
	for key, r := range keyboardResults(rep, "increment") {
		if r.Status != NotMeasured || !strings.Contains(strings.Join(r.Messages, " "), "a press on `increment` changed nothing") {
			t.Fatalf("%s: %+v", key, r)
		}
	}
	// The switch still changes, so its test runs and passes.
	for key, r := range keyboardResults(rep, "switch") {
		if r.Status != Pass {
			t.Fatalf("%s: %+v", key, r)
		}
	}
}

// R-KEY-12: a target can say which keys activate a role (advertise keys); a key it leaves out is
// not checked there, and the others still are.
func TestKeyboardTargetKeys(t *testing.T) {
	rep, _ := run(t, map[string]T{"a": {Env: map[string]string{"FAKE_KEYS": "button=Enter,switch=", "FAKE_BUG": "no-enter"}}})
	got := keyboardResults(rep, "increment")
	if got["Space"].Status != NotAdvertised || !strings.Contains(got["Space"].Missing["key Space"], "a says Space doesn't activate a button there (advertise keys: button: Enter)") {
		t.Fatalf("%+v", got["Space"])
	}
	if got["Enter"].Status != Fail {
		t.Fatalf("Enter should still be checked, and fail with no-enter: %+v", got["Enter"])
	}
	sw := keyboardResults(rep, "switch")["Space"]
	if sw.Status != NotAdvertised || !strings.Contains(sw.Missing["key Space"], "no key activates a switch there") {
		t.Fatalf("%+v", sw)
	}
	if len(rep.Targets[0].Keys) != 2 {
		t.Fatalf("%+v", rep.Targets[0])
	}
}
