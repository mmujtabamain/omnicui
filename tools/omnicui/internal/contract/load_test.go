package contract

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

const good = `id: ui.counter
props:
  start: { type: integer, default: 0 }
events:
  changed:
    count: { type: integer }
parts:
  value:     { role: status }
  increment: { role: button }
tests:
  - name: three presses
    steps:
      - { action: press, part: increment, times: 3 }
    expect:
      - { part: value, text: "3" }
      - { event: changed, times: 3, last: { count: 3 } }
`

func parse(t *testing.T, yaml string) (*Contract, []Problem) {
	t.Helper()
	return Parse("x.contract.yaml", []byte(yaml))
}

// R-CONTRACT-1: the example contracts load with no errors and no warnings.
func TestExampleContractsLoad(t *testing.T) {
	cs, ps, err := LoadDir(filepath.Join("..", "..", "..", "..", "omnicui", "contracts"))
	if err != nil {
		t.Fatal(err)
	}
	if len(ps) != 0 {
		t.Fatalf("problems: %v", ps)
	}
	if len(cs) != 2 || cs[0].ID != "ui.counter" || cs[1].ID != "ui.switch" {
		t.Fatalf("got %d contracts", len(cs))
	}
}

// R-CONTRACT-2: a valid contract becomes the model, in file order, with steps expanded later.
func TestModel(t *testing.T) {
	c, ps := parse(t, good)
	if c == nil {
		t.Fatalf("problems: %v", ps)
	}
	if c.Parts[0].Name != "value" || c.Parts[1].ID != "ui.counter.increment" {
		t.Errorf("parts: %+v", c.Parts)
	}
	st := c.Tests[0].Steps[0]
	if st.Times != 3 || st.Action != "press" || st.Line != 13 {
		t.Errorf("step: %+v", st)
	}
	e := c.Tests[0].Expect
	if e[0].Checks[0].Field != "text" || e[0].Checks[0].Want != "3" || *e[1].Times != 3 || e[1].Last["count"] != int64(3) {
		t.Errorf("expect: %+v", e)
	}
	if c.Props[0].Default != int64(0) || !c.Props[0].HasDefault {
		t.Errorf("props: %+v", c.Props)
	}
}

// R-CONTRACT-3: what a contract leaves out takes the documented defaults.
func TestDefaults(t *testing.T) {
	c, _ := parse(t, good)
	if len(c.Viewports) != 1 || c.Viewports[0] != DefaultViewport {
		t.Errorf("viewports: %v", c.Viewports)
	}
	if len(c.ColorSchemes) != 1 || c.ColorSchemes[0] != "light" {
		t.Errorf("schemes: %v", c.ColorSchemes)
	}
	if c.Pixels != DefaultPixelThresholds || c.TimingThreshold != 20 {
		t.Errorf("thresholds: %+v %v", c.Pixels, c.TimingThreshold)
	}
	c, _ = parse(t, good+"thresholds: { area: 25%, strength: 30%, shift: 7px, spot: 5%, timing: 18ms }\n")
	if c.Pixels != (PixelThresholds{Area: 25, Strength: 30, Shift: 7, Spot: 5}) || c.TimingThreshold != 18 {
		t.Errorf("thresholds: %+v %v", c.Pixels, c.TimingThreshold)
	}
	// Only the measures named change; the others keep their defaults.
	c, _ = parse(t, good+"thresholds: { spot: 5% }\n")
	if want := DefaultPixelThresholds; c.Pixels.Spot != 5 || c.Pixels.Area != want.Area || c.Pixels.Shift != want.Shift {
		t.Errorf("thresholds: %+v", c.Pixels)
	}
}

func replace(old, new string) string { return strings.Replace(good, old, new, 1) }

// R-CONTRACT-4: every mistake is reported with its line, and typos get the closest match.
func TestErrors(t *testing.T) {
	cases := []struct {
		name, yaml string
		line       int
		want       string
	}{
		{"unknown top field", good + "screenshots: true\n", 17, "unknown field `screenshots`"},
		{"typo in a test field", replace("    expect:", "    expectt:"), 14, "unknown field `expectt`; did you mean `expect`?"},
		{"wrong value type", replace(`text: "3"`, "text: 3"), 15, "expected string, got number"},
		{"bad role", replace("role: status", "role: label"), 8, "is not one of"},
		{"bad action", replace("action: press", "action: tap"), 13, "`tap` is not one of"},
		{"missing key for key action", replace("action: press, part: increment, times: 3", "action: key, part: increment"), 13, "missing field `key`"},
		{"bad id", replace("id: ui.counter", "id: Counter"), 1, "`Counter` is not a valid id: an id is lower-case words joined by dots"},
		{"no tests", strings.Split(good, "tests:")[0], 1, "missing field `tests`"},
		{"unknown part", replace("part: increment, times", "part: incremnt, times"), 13, "part `incremnt` does not exist; did you mean `increment`?"},
		{"role refuses action", replace("part: increment, times", "part: value, times"), 13, "`value` is a status, which does not accept `press`"},
		{"role has no field", replace(`{ part: value, text: "3" }`, `{ part: value, checked: true }`), 15, "`value` is a status, which has no field `checked`"},
		{"unknown event", replace("event: changed", "event: chnged"), 16, "event `chnged` does not exist; did you mean `changed`?"},
		{"unknown event field", replace("last: { count: 3 }", "last: { cuont: 3 }"), 16, "event `changed` has no field `cuont`; did you mean `count`?"},
		{"event field type", replace("last: { count: 3 }", `last: { count: "3" }`), 16, "field `count` of `changed` is a string, but its type is integer"},
		{"unknown prop", replace("  - name: three presses\n", "  - name: three presses\n    props: { strat: 5 }\n"), 12, "prop `strat` does not exist; did you mean `start`?"},
		{"prop type", replace("  - name: three presses\n", "  - name: three presses\n    props: { start: \"5\" }\n"), 12, "prop `start` is a string, but its type is integer"},
		{"default type", replace("default: 0", `default: "zero"`), 3, "the default of `start` is a string, but its type is integer"},
		{"duplicate test", good + "  - name: three presses\n    expect: [{ part: value, text: \"0\" }]\n", 17, "another test is already called `three presses`"},
		{"trace without steps", good + "  - name: t\n    trace: [{ part: value, point: center }]\n", 18, "traces a part but has no steps"},
		{"bad easing", replace("    expect:", "    trace: [{ part: value, point: center, timing: { duration: 200ms, easing: bouncy } }]\n    expect:"), 14, "`bouncy` is not a timing function: linear, ease"},
		{"not yaml", "id: [unclosed\n", 1, "not valid YAML"},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			c, ps := parse(t, tc.yaml)
			if c != nil {
				t.Fatalf("expected an error, got a contract")
			}
			for _, p := range ps {
				if strings.Contains(p.Message, tc.want) && !p.Warning {
					if p.Line != tc.line {
						t.Errorf("line %d, want %d (%s)", p.Line, tc.line, p.Message)
					}
					return
				}
			}
			t.Errorf("no problem containing %q in %v", tc.want, ps)
		})
	}
}

// R-CONTRACT-5: a part no test uses is a warning, not an error.
func TestUnusedPartWarns(t *testing.T) {
	c, ps := parse(t, replace("  increment: { role: button }\n", "  increment: { role: button }\n  extra: { role: heading }\n"))
	if c == nil {
		t.Fatalf("a warning must not stop the contract: %v", ps)
	}
	if len(ps) != 1 || !ps[0].Warning || ps[0].Line != 10 || !strings.Contains(ps[0].Message, "part `extra` is never used by any test") {
		t.Errorf("problems: %v", ps)
	}
}

// R-CONTRACT-9: thresholds.pixels was renamed to area, and is refused with the new spelling.
func TestPixelsRenamedToArea(t *testing.T) {
	c, ps := parse(t, good+"thresholds: { pixels: 30% }\n")
	if c != nil || !HasErrors(ps) || ps[0].Line != 17 || !strings.Contains(ps[0].Message, "thresholds.pixels was renamed to area") || !strings.Contains(ps[0].Message, "`area: 30%`") {
		t.Errorf("got %v", ps)
	}
}

// R-CONTRACT-10: a threshold far stricter than its default is allowed, with a warning that says
// what correct renderings usually measure, because otherwise every comparison warns.
func TestStrictThresholdWarns(t *testing.T) {
	c, ps := parse(t, good+"thresholds: { area: 1%, timing: 20ms }\n")
	if c == nil || len(ps) != 1 || !ps[0].Warning || ps[0].Line != 17 {
		t.Fatalf("got %v", ps)
	}
	for _, want := range []string{"thresholds.area 1% is far below the default 30%", "3–26%", "expect most comparisons to warn"} {
		if !strings.Contains(ps[0].Message, want) {
			t.Errorf("%q should say %q", ps[0].Message, want)
		}
	}
	if _, ps := parse(t, good+"thresholds: { area: 20%, timing: 15ms }\n"); len(ps) != 0 {
		t.Errorf("a modest change should not warn: %v", ps)
	}
}

// R-CONTRACT-11: a value in the wrong form is told what form it should take, and a name gets
// its camelCase spelling.
func TestPatternMessages(t *testing.T) {
	cases := []struct{ yaml, want string }{
		{replace("  value:     { role: status }", "  card-number: { role: status }"), "`card-number` is not a valid name: names are camelCase, a lower-case letter and then letters and digits; write `cardNumber`"},
		{replace("id: ui.counter", "id: Counter"), "`Counter` is not a valid id: an id is lower-case words joined by dots, like `ui.counter`"},
		{good + "thresholds: { area: 30 }\n", "`30` is not a percentage: write it like `30%`"},
		{good + "thresholds: { shift: 5 }\n", "`5` is not a length in pixels: write it like `5px`"},
		{good + "thresholds: { timing: 20 }\n", "`20` is not a duration: write it in milliseconds, like `200ms`"},
	}
	for _, c := range cases {
		_, ps := parse(t, c.yaml)
		found := false
		for _, p := range ps {
			found = found || strings.Contains(p.Message, c.want)
		}
		if !found {
			t.Errorf("want %q in %v", c.want, ps)
		}
	}
	if camelCase("card_number") != "cardNumber" || camelCase("Card Number") != "cardNumber" || camelCase("9x") != "" {
		t.Error("camelCase")
	}
}

// R-CONTRACT-6: two contracts may not share an id.
func TestDuplicateIDs(t *testing.T) {
	dir := t.TempDir()
	os.WriteFile(filepath.Join(dir, "a.contract.yaml"), []byte(good), 0o644)
	os.WriteFile(filepath.Join(dir, "b.contract.yaml"), []byte(good), 0o644)
	cs, ps, _ := LoadDir(dir)
	if len(cs) != 1 || !HasErrors(ps) || !strings.Contains(ps[0].Message, "already used") {
		t.Errorf("got %d contracts, %v", len(cs), ps)
	}
}

func TestClosest(t *testing.T) {
	for _, tc := range []struct{ word, want string }{
		{"incremnt", "increment"}, {"nmae", "name"}, {"chekced", "checked"}, {"zzzz", ""}, {"name", ""},
	} {
		if got := closest(tc.word, []string{"increment", "name", "checked", "value"}); got != tc.want {
			t.Errorf("closest(%q) = %q, want %q", tc.word, got, tc.want)
		}
	}
}
