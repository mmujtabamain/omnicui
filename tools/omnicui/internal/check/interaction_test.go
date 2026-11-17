package check

import (
	"strings"
	"testing"
)

// R-LEAVE-1: `leave` moves the pointer off the component, so what a hover showed can be expected
// gone again (`seen`: faded out, though it keeps its box); a part that stays after the pointer left fails.
func TestLeaveHidesWhatHoverShowed(t *testing.T) {
	hover := func(s string) string {
		return addTest(noKeyboard(s), `  - name: shown on hover
    steps:
      - { action: hover, part: increment }
    expect:
      - { part: value, seen: true }
  - name: gone when the pointer leaves
    steps:
      - { action: hover, part: increment }
      - { action: leave }
    expect:
      - { part: value, visible: true, seen: false }
`)
	}
	rep := runEdited(t, map[string]T{"a": {Env: map[string]string{"FAKE_REVEAL": "value"}}}, hover)
	for _, name := range []string{"shown on hover", "gone when the pointer leaves"} {
		if r := result(t, rep, "a", name); r.Status != Pass {
			t.Fatalf("%s: %+v", name, r)
		}
	}
	rep = runEdited(t, map[string]T{"a": {Env: map[string]string{"FAKE_REVEAL": "value", "FAKE_BUG": "sticky-hover"}}}, hover)
	r := result(t, rep, "a", "gone when the pointer leaves")
	if r.Status != Fail || !strings.Contains(strings.Join(r.Messages, "\n"), "value.seen: expected false, got true") {
		t.Fatalf("%+v", r)
	}
	// A target that can't move the pointer away doesn't run the test.
	rep = runEdited(t, map[string]T{"a": {Env: map[string]string{"FAKE_REVEAL": "value", "FAKE_CANNOT": "leave=no pointer"}}}, hover)
	if r := result(t, rep, "a", "gone when the pointer leaves"); r.Status != NotAdvertised {
		t.Fatalf("%+v", r)
	}
}

// R-FOCUSVISIBLE-1: focusVisible tells keyboard focus from a press: a press gives focus but no
// ring, focus from the keyboard shows it.
func TestFocusVisible(t *testing.T) {
	ring := func(s string) string {
		return addTest(noKeyboard(s), `  - name: a press shows no ring
    steps:
      - { action: press, part: increment }
    expect:
      - { part: increment, focused: true, focusVisible: false }
  - name: keyboard focus shows the ring
    steps:
      - { action: focus, part: increment }
    expect:
      - { part: increment, focused: true, focusVisible: true }
`)
	}
	rep := runEdited(t, map[string]T{"a": {}}, ring)
	for _, name := range []string{"a press shows no ring", "keyboard focus shows the ring"} {
		if r := result(t, rep, "a", name); r.Status != Pass {
			t.Fatalf("%s: %+v", name, r)
		}
	}
	rep = runEdited(t, map[string]T{"a": {Env: map[string]string{"FAKE_BUG": "ring-on-press"}}}, ring)
	r := result(t, rep, "a", "a press shows no ring")
	if r.Status != Fail || !strings.Contains(strings.Join(r.Messages, "\n"), "increment.focusVisible: expected false, got true") {
		t.Fatalf("%+v", r)
	}
}

// R-WINDOW-1: a part's window: who draws its title bar, and whether it can be minimised or zoomed.
// A target with no windows to read doesn't run the test.
func TestWindowFields(t *testing.T) {
	window := func(s string) string {
		return addTest(noKeyboard(s), `  - name: a panel window
    expect:
      - { part: value, titleBar: app, minimizable: false, maximizable: false }
`)
	}
	rep := runEdited(t, map[string]T{"a": {}}, window)
	if r := result(t, rep, "a", "a panel window"); r.Status != Pass {
		t.Fatalf("%+v", r)
	}
	rep = runEdited(t, map[string]T{"a": {Env: map[string]string{"FAKE_TITLEBAR_KIND": "os", "FAKE_WINDOW": "minimizable,maximizable"}}}, window)
	r := result(t, rep, "a", "a panel window")
	msgs := strings.Join(r.Messages, "\n")
	for _, want := range []string{`value.titleBar: expected "app", got "os"`, "value.minimizable: expected false, got true", "value.maximizable: expected false, got true"} {
		if !strings.Contains(msgs, want) {
			t.Errorf("missing %q in %s", want, msgs)
		}
	}
	rep = runEdited(t, map[string]T{"a": {Env: map[string]string{"FAKE_CANNOT": "window=a browser page has no window of its own"}}}, window)
	if r := result(t, rep, "a", "a panel window"); r.Status != NotAdvertised {
		t.Fatalf("%+v", r)
	}
}
