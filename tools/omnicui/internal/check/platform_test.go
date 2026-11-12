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

// Platforms: what may differ by platform, per part and per test. These tests come from the
// per-platform design (a desktop title bar, iOS against Android, back on each OS).

const canBump = "mount,press,focus,key,type,drag,hover,read,events,screenshot,probe,bounds,do.bump"

// runProject runs a check of the counter contract, edited by edit, with platforms.yaml as given
// (none when empty).
func runProject(t *testing.T, targets map[string]T, platforms string, edit func(string) string) (*Report, project.Project) {
	t.Helper()
	root := testutil.Project(t, targets)
	if platforms != "" {
		os.WriteFile(filepath.Join(root, "platforms.yaml"), []byte(platforms), 0o644)
	}
	path := filepath.Join(root, "contracts", "counter.contract.yaml")
	data, _ := os.ReadFile(path)
	os.WriteFile(path, []byte(edit(string(data))), 0o644)
	p := project.Project{Root: root}
	cs, ps, err := contract.LoadDir(p.Contracts())
	if err != nil || contract.HasErrors(ps) {
		t.Fatal(err, ps)
	}
	var counter []*contract.Contract
	for _, c := range cs {
		if c.ID == "ui.counter" {
			counter = append(counter, c)
		}
	}
	ts, _ := p.Targets()
	rep, err := Run(Options{Project: p, Contracts: counter, Targets: ts})
	if err != nil {
		t.Fatal(err)
	}
	return rep, p
}

func addTest(s, test string) string {
	return strings.Replace(s, "\nkeyboard:", "\n"+test+"\nkeyboard:", 1)
}

func noKeyboard(s string) string {
	return strings.Replace(s, "keyboard:     true", "keyboard:     false", 1)
}

// R-PLATFORM-1: a target's forms come from target.yaml (`platform:`), its OS from advertise
// (or target.yaml `os:`); the console says what each runs on.
func TestPlatformTags(t *testing.T) {
	rep, p := runProject(t, map[string]T{
		"a": {},
		"b": {Env: map[string]string{"FAKE_OS": "android"}, Extra: "platform: [mobile]\n"},
		"c": {Extra: "platform: [desktop]\nos: macos\n"},
	}, "", noKeyboard)
	got := map[string]contract.Platform{}
	for _, ti := range rep.Targets {
		got[ti.Name] = ti.Platform
	}
	if got["b"].OS != "android" || strings.Join(got["b"].Forms, ",") != "mobile" || got["c"].OS != "macos" || got["a"].OS != "" {
		t.Fatalf("%+v", got)
	}
	var out bytes.Buffer
	Print(&out, rep, p.RunDir())
	if !strings.Contains(out.String(), "platforms: a · b (mobile, android) · c (desktop, macos)") {
		t.Fatal(out.String())
	}
}

// R-PLATFORM-2: `only:` and `except:` make a test not applicable on other platforms: not a
// failure, not a gap, and counted on its own.
func TestOnlyExcept(t *testing.T) {
	rep, p := runProject(t, map[string]T{
		"a": {Extra: "platform: [desktop]\n"},
		"b": {Env: map[string]string{"FAKE_OS": "android"}, Extra: "platform: [mobile]\n"},
	}, "", func(s string) string {
		return addTest(noKeyboard(s), `  - name: mobile only
    only: [mobile]
    expect: [ { part: value, text: "0" } ]
  - name: not on android
    except: [android]
    expect: [ { part: value, text: "0" } ]`)
	})
	for _, c := range []struct {
		target, test string
		want         Status
	}{{"a", "mobile only", NotApplicable}, {"b", "mobile only", Pass}, {"a", "not on android", Pass}, {"b", "not on android", NotApplicable}} {
		if r := result(t, rep, c.target, c.test); r.Status != c.want {
			t.Errorf("%s %s: %+v", c.target, c.test, r)
		}
	}
	if r := result(t, rep, "a", "mobile only"); !contains(r.Messages, "only: [mobile], and a (desktop) isn't") {
		t.Errorf("%+v", r.Messages)
	}
	if rep.Summary.NotApplicable != 2 || rep.Failed() {
		t.Errorf("%+v", rep.Summary)
	}
	var out bytes.Buffer
	Print(&out, rep, p.RunDir())
	if !strings.Contains(out.String(), " · 2 not applicable") {
		t.Fatal(out.String())
	}
}

const platformsYAML = `inputs:
  addOne:
    default: { action: press }
    mobile:  { do: bump, with: { by: 1 } }
    tablet:  none
`

// R-PLATFORM-3: a named input (platforms.yaml) is sent to each target in its platform's form,
// and each result says what it became; `none` makes the test not applicable there.
func TestNamedInputs(t *testing.T) {
	rep, _ := runProject(t, map[string]T{
		"a": {},
		"b": {Env: map[string]string{"FAKE_CAN": canBump}, Extra: "platform: [mobile]\n"},
		"c": {Extra: "platform: [tablet]\n"},
	}, platformsYAML, func(s string) string {
		return addTest(noKeyboard(s), `  - name: add one, the platform's way
    steps: [ { input: addOne, part: increment } ]
    expect: [ { part: value, text: "1" }, { event: changed, times: 1 } ]`)
	})
	const test = "add one, the platform's way"
	if r := result(t, rep, "a", test); r.Status != Pass || !contains(r.Inputs, "input `addOne`: press (platforms.yaml, `default`)") {
		t.Errorf("%+v", r)
	}
	if r := result(t, rep, "b", test); r.Status != Pass || !contains(r.Inputs, "input `addOne`: do bump (platforms.yaml, `mobile`)") {
		t.Errorf("%+v", r)
	}
	if r := result(t, rep, "c", test); r.Status != NotApplicable || !contains(r.Messages, "input `addOne` is none on c (platforms.yaml, `tablet`)") {
		t.Errorf("%+v", r)
	}
}

// R-PLATFORM-4: an input a contract names must be in platforms.yaml, and its forms are checked.
func TestNamedInputErrors(t *testing.T) {
	root := testutil.Project(t, map[string]T{})
	os.WriteFile(filepath.Join(root, "platforms.yaml"), []byte(platformsYAML+"  broken:\n    default: { key: A, do: back }\n"), 0o644)
	path := filepath.Join(root, "contracts", "counter.contract.yaml")
	data, _ := os.ReadFile(path)
	os.WriteFile(path, []byte(addTest(string(data), `  - name: typo
    steps: [ { input: addOn, part: increment } ]
    expect: [ { part: value, text: "1" } ]`)), 0o644)
	_, ps, _ := contract.LoadDir(filepath.Join(root, "contracts"))
	var msgs []string
	for _, p := range ps {
		msgs = append(msgs, p.Message)
	}
	all := strings.Join(msgs, "\n")
	if !strings.Contains(all, "input `addOn` is not in platforms.yaml; did you mean `addOne`?") ||
		!strings.Contains(all, "input `broken` on `default`: write one of { key: … }, { action: press | focus | hover }, { do: … } or none") {
		t.Fatal(all)
	}
}

// R-PLATFORM-5: an expected value may differ by platform, with a reason: the target is held to
// its platform's value, and each one is a platform exception, listed and counted. Without why:
// a contract error.
func TestPlatformExceptions(t *testing.T) {
	targets := map[string]T{"a": {}, "b": {Env: map[string]string{"FAKE_TEXT_SUFFIX": " taps"}, Extra: "platform: [mobile]\n"}}
	edit := func(why string) func(string) string {
		return func(s string) string {
			return strings.Replace(noKeyboard(s), `      - { part: value, text: "3" }`, `      - part: value
        text: { default: "3", mobile: "3 taps" }`+why, 1)
		}
	}
	rep, p := runProject(t, targets, "", edit("\n        why: the phone layout says what the number counts"))
	if r := result(t, rep, "b", "each press adds one"); r.Status != Pass ||
		!contains(r.Exceptions, `value.text: "3 taps" on b (`+"`mobile`"+`), not "3"; why: the phone layout says what the number counts`) {
		t.Fatalf("%+v", r)
	}
	if r := result(t, rep, "a", "each press adds one"); r.Status != Pass || len(r.Exceptions) != 0 {
		t.Fatalf("%+v", r)
	}
	if rep.Summary.Exceptions != 1 {
		t.Errorf("%+v", rep.Summary)
	}
	var out bytes.Buffer
	Print(&out, rep, p.RunDir())
	if !strings.Contains(out.String(), "1 platform exceptions") || !strings.Contains(out.String(), "b: platform exception: value.text") {
		t.Fatal(out.String())
	}
	// Without why.
	root := testutil.Project(t, targets)
	path := filepath.Join(root, "contracts", "counter.contract.yaml")
	data, _ := os.ReadFile(path)
	os.WriteFile(path, []byte(edit("")(string(data))), 0o644)
	_, ps, _ := contract.LoadDir(filepath.Join(root, "contracts"))
	found := false
	for _, pr := range ps {
		found = found || strings.Contains(pr.Message, "`text` differs by platform, so the expectation must say why")
	}
	if !found {
		t.Fatalf("%+v", ps)
	}
}

// R-PLATFORM-6: viewports per platform: each target runs at its most specific selector's sizes,
// and the i-th size of each is compared.
func TestViewportsPerPlatform(t *testing.T) {
	rep, _ := runProject(t, map[string]T{"a": {}, "b": {Extra: "platform: [mobile]\n"}}, "", func(s string) string {
		return strings.Replace(noKeyboard(s), "viewports:    [{ width: 320, height: 240 }]",
			"viewports:\n  default: [{ width: 320, height: 240 }]\n  mobile:  [{ width: 200, height: 300 }]", 1)
	})
	r := result(t, rep, "b", "starts at the start value")
	if _, ok := r.Screenshots["200x300-light"]; !ok {
		t.Fatalf("%+v", r.Screenshots)
	}
	if c := pixelsOf(rep, "starts at the start value"); c == nil || c.Combo != "320x240-light / 200x300-light" {
		t.Fatalf("%+v", c)
	}
}

// R-PLATFORM-7: `same:` per part. A pair compares what both promised: parts held to layout only
// are masked out of the pixels and compared by where they are; a part held to pixels across
// designs is compared as a crop; a part with no motion in common isn't traced against the other.
func TestSamePerPart(t *testing.T) {
	sameLayout := func(s string) string {
		s = strings.Replace(noKeyboard(s), "  value:     { role: status }", "  value:     { role: status, same: { default: pixels, mobile: [layout] } }", 1)
		return strings.Replace(s, "  increment: { role: button }", "  increment: { role: button, same: { default: pixels, mobile: [layout] } }", 1)
	}
	// b draws the block 100 px to the right, and its parts keep their places relative to each other.
	bounds := "value=124,24,26,30;increment=150,24,34,30"
	rep, _ := runProject(t, map[string]T{
		"a": {Env: map[string]string{"FAKE_BOUNDS": "value=24,24,26,30;increment=50,24,34,30"}},
		"b": {Env: map[string]string{"FAKE_SHIFT": "100", "FAKE_BOUNDS": bounds}, Extra: "platform: [mobile]\n"},
	}, "", sameLayout)
	px := pixelsOf(rep, "starts at the start value")
	if px == nil || px.Warning || !strings.Contains(px.Note, "masked: value, increment") ||
		!strings.Contains(px.Note, "compared as pixels, except value: layout · increment: layout") {
		t.Fatalf("%+v", px)
	}
	var layout *Comparison
	for i, c := range rep.Comparisons {
		if c.Kind == "layout" && c.Test == "starts at the start value" {
			layout = &rep.Comparisons[i]
		}
	}
	if layout == nil || layout.Warning || layout.Value != 0 {
		t.Fatalf("%+v", layout)
	}
	// The parts in another order on b: a layout warning, whatever the pixels.
	rep, _ = runProject(t, map[string]T{
		"a": {Env: map[string]string{"FAKE_BOUNDS": "value=24,24,26,30;increment=50,24,34,30"}},
		"b": {Env: map[string]string{"FAKE_SHIFT": "100", "FAKE_BOUNDS": "value=158,24,26,30;increment=124,24,34,30"}, Extra: "platform: [mobile]\n"},
	}, "", sameLayout)
	layout = nil
	for i, c := range rep.Comparisons {
		if c.Kind == "layout" && c.Test == "starts at the start value" {
			layout = &rep.Comparisons[i]
		}
	}
	if layout == nil || !layout.Warning || layout.Value != 100 || !strings.Contains(layout.Note, "the parts are in another order: value, increment on a, increment, value on b") {
		t.Fatalf("%+v", layout)
	}

	// Across designs (colours by default), a part held to pixels by both is compared as a crop.
	rep, _ = runProject(t, map[string]T{"a": {}, "b": {Extra: "design: phone\n"}}, "", func(s string) string {
		return strings.Replace(noKeyboard(s), "  increment: { role: button }", "  increment: { role: button, same: pixels }", 1)
	})
	var crop *Comparison
	for i, c := range rep.Comparisons {
		if c.Kind == "pixels" && c.Part == "increment" && c.Test == "starts at the start value" {
			crop = &rep.Comparisons[i]
		}
	}
	if crop == nil || !strings.Contains(crop.Note, "only `increment`, which this pair holds pixel-identical") {
		t.Fatalf("%+v", rep.Comparisons)
	}
}

// R-PLATFORM-8: a part's motion is compared only between targets that both promised it.
func TestSameMotion(t *testing.T) {
	root := testutil.Project(t, map[string]T{"a": {}, "b": {Env: map[string]string{"FAKE_VERTICAL": "1"}, Extra: "platform: [mobile]\n"}})
	path := filepath.Join(root, "contracts", "switch.contract.yaml")
	data, _ := os.ReadFile(path)
	os.WriteFile(path, []byte(strings.Replace(string(data), "  thumb:  { role: generic }", "  thumb:  { role: generic, same: { default: pixels, mobile: none } }", 1)), 0o644)
	p := project.Project{Root: root}
	cs, ps, err := contract.LoadDir(p.Contracts())
	if err != nil || contract.HasErrors(ps) {
		t.Fatal(err, ps)
	}
	ts, _ := p.Targets()
	var sw []*contract.Contract
	for _, c := range cs {
		if c.ID == "ui.switch" {
			sw = append(sw, c)
		}
	}
	rep, err := Run(Options{Project: p, Contracts: sw, Targets: ts})
	if err != nil {
		t.Fatal(err)
	}
	for _, c := range rep.Comparisons {
		if (c.Kind == "timing" || c.Kind == "direction") && c.B == "b" {
			t.Errorf("b promised no motion for the thumb: %+v", c)
		}
	}
}

// R-PLATFORM-9: regions a target's platform drew (a title bar) are left out of every
// comparison, without any contract naming them.
func TestSystemRegions(t *testing.T) {
	rep, _ := runProject(t, map[string]T{"a": {}, "b": {Env: map[string]string{"FAKE_REGIONS": "titlebar:0,0,320,16"}, Extra: "platform: [desktop]\n"}}, "", noKeyboard)
	c := pixelsOf(rep, "starts at the start value")
	if c == nil || c.Warning || c.Measures.Area != 0 || !strings.Contains(c.Note, "left out, drawn by the platform: b's titlebar") {
		t.Fatalf("%+v", c)
	}
	if r := result(t, rep, "b", "starts at the start value"); len(r.Regions["320x240-light"]) != 1 {
		t.Fatalf("%+v", r.Regions)
	}
}

// R-PLATFORM-10: a selector that names no form, OS, target or design is a warning, with the
// nearest name.
func TestUnknownSelectors(t *testing.T) {
	cs := []*contract.Contract{{ID: "ui.x", File: "x.contract.yaml", Tests: []contract.Test{{Name: "t", Only: []string{"iso", "desktop", "phone"}, Line: 7}}}}
	ps := contract.UnknownSelectors(cs, []string{"web", "phone"})
	if len(ps) != 1 || !ps[0].Warning || ps[0].Line != 7 || !strings.Contains(ps[0].Message, "`iso` (only/except) is not a platform, a target or a design; did you mean `ios`?") {
		t.Fatalf("%+v", ps)
	}
}
