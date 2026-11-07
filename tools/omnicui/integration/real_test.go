//go:build real

// Package integration runs the built omnicui against the real targets (web in Chromium, egui,
// JavaFX) in the reference repository's omnicui/ project, each mounting its app's components. Run it with `make test-real` after `make install`. Every deliberate defect is
// switched on in turn, and the check must catch it; with none switched on, everything passes.
//
// Set OMNICUI_EVIDENCE to a folder to keep each scenario's console output and results.json.
package integration

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"

	"omnicui/internal/check"
	"omnicui/internal/testutil"
)

func bin(t *testing.T) string {
	if b := os.Getenv("OMNICUI_BIN"); b != "" {
		return b
	}
	return filepath.Join(testutil.RepoRoot(), "bin", "omnicui")
}

type outcome struct {
	code int
	out  string
	rep  check.Report
}

func omnicui(t *testing.T, name string, env []string, args ...string) outcome {
	t.Helper()
	cmd := exec.Command(bin(t), args...)
	cmd.Dir = testutil.ProjectRoot()
	cmd.Env = append(os.Environ(), env...)
	out, _ := cmd.CombinedOutput()
	o := outcome{code: cmd.ProcessState.ExitCode(), out: string(out)}
	data, err := os.ReadFile(filepath.Join(testutil.ProjectRoot(), ".omnicui", "run", "results.json"))
	if err == nil {
		json.Unmarshal(data, &o.rep)
	}
	if dir := os.Getenv("OMNICUI_EVIDENCE"); dir != "" {
		os.MkdirAll(dir, 0o755)
		os.WriteFile(filepath.Join(dir, name+".txt"), []byte(fmt.Sprintf("$ %s omnicui %s\n%s\nexit status %d\n", strings.Join(env, " "), strings.Join(args, " "), out, o.code)), 0o644)
		os.WriteFile(filepath.Join(dir, name+".results.json"), data, 0o644)
	}
	t.Logf("%s: exit %d\n%s", name, o.code, out)
	return o
}

func status(o outcome, target, test string) check.Status {
	for _, r := range o.rep.Results {
		if r.Target == target && r.Test == test {
			return r.Status
		}
	}
	return "missing"
}

func warned(o outcome, kind, target, test string) bool {
	for _, c := range o.rep.Comparisons {
		if c.Kind == kind && c.Warning && c.Test == test && (c.A == target || c.B == target) {
			return true
		}
	}
	return false
}

func onlyWarningsInvolve(t *testing.T, o outcome, target string) {
	t.Helper()
	for _, c := range o.rep.Comparisons {
		if c.Warning && c.A != target && c.B != target {
			t.Errorf("a warning that doesn't involve %s: %+v", target, c)
		}
	}
}

const (
	counterStart = "starts at the start value"
	counterPress = "each press adds one"
	switchOff    = "starts off, with its label as its name"
	switchOn     = "starts on when asked"
	switchPress  = "a press turns it on, and the thumb slides across"
)

// R-REAL-1: with no defect, every test passes on all three targets, with no warning.
func TestAllTargetsAgree(t *testing.T) {
	o := omnicui(t, "00-baseline", nil, "check")
	s := o.rep.Summary
	// 7 written tests and 3 keyboard tests (keyboard: true), on three targets.
	if o.code != 0 || s.Pass != 30 || s.Fail != 0 || s.Broken != 0 || s.NotAdvertised != 0 || s.Warnings != 0 {
		t.Fatalf("summary %+v, exit %d", s, o.code)
	}
	if len(o.rep.Targets) != 3 {
		t.Fatalf("targets: %+v", o.rep.Targets)
	}
	// Every construct, on every target; trace.opacity where the toolkit has an opacity to read.
	for _, ti := range o.rep.Targets {
		want := 16
		if ti.Name != "egui" {
			want = 17
		} else if !strings.Contains(ti.Cannot["trace.opacity"], "no per-widget opacity") {
			t.Errorf("egui should say why it can't trace opacity: %v", ti.Cannot)
		}
		if len(ti.Can) != want || ti.Probe != "passed" {
			t.Errorf("%s advertises %v, render check %q", ti.Name, ti.Can, ti.Probe)
		}
	}
}

// R-REAL-2 (the same request twice): a second run gives the same statuses and the same warnings.
func TestRepeatable(t *testing.T) {
	a := omnicui(t, "01-repeat-a", nil, "check")
	b := omnicui(t, "01-repeat-b", nil, "check")
	if a.rep.Summary != b.rep.Summary {
		t.Fatalf("%+v against %+v", a.rep.Summary, b.rep.Summary)
	}
	for i := range a.rep.Results {
		if a.rep.Results[i].Status != b.rep.Results[i].Status {
			t.Errorf("%+v against %+v", a.rep.Results[i], b.rep.Results[i])
		}
	}
}

// R-REAL-3: each deliberate defect is caught, on the right target and test, and nothing else moves.
func TestDefectsAreCaught(t *testing.T) {
	cases := []struct {
		defect string
		args   []string
		check  func(t *testing.T, o outcome)
	}{
		{"javafx.switch.no-label", []string{"check", "switch"}, func(t *testing.T, o outcome) {
			if status(o, "javafx", switchOff) != check.Fail || status(o, "javafx", switchOn) != check.Fail || o.code != 1 {
				t.Error("javafx's unnamed switch must fail both naming tests")
			}
			if status(o, "web", switchOff) != check.Pass || status(o, "egui", switchOff) != check.Pass {
				t.Error("the other targets must still pass")
			}
			if !strings.Contains(o.out, `javafx: switch.name: expected "Dark mode", got nothing`) {
				t.Error("the message must say what was expected and what came back")
			}
		}},
		{"egui.switch.no-name", []string{"check", "switch"}, func(t *testing.T, o outcome) {
			if status(o, "egui", switchOff) != check.Fail || status(o, "egui", switchOn) != check.Fail || status(o, "javafx", switchOff) != check.Pass {
				t.Error("only egui's naming tests must fail")
			}
			if !strings.Contains(o.out, `egui: switch.name: expected "Dark mode", got nothing`) {
				t.Error("message")
			}
		}},
		{"egui.counter.small-text", []string{"check", "counter"}, func(t *testing.T, o outcome) {
			if !warned(o, "pixels", "egui", counterStart) || !warned(o, "pixels", "egui", counterPress) || o.rep.Summary.Fail != 0 || o.code != 0 {
				t.Error("smaller text on egui must be a pixel warning on both counter screenshots, and not a failure")
			}
			onlyWarningsInvolve(t, o, "egui")
		}},
		{"web.counter.padding", []string{"check", "counter"}, func(t *testing.T, o outcome) {
			if !warned(o, "pixels", "web", counterStart) || !warned(o, "pixels", "web", counterPress) {
				t.Error("a wider web button must be a pixel warning")
			}
			onlyWarningsInvolve(t, o, "web")
			// The worst spot is put down to the part it is in: the wider button.
			for _, c := range o.rep.Comparisons {
				if c.Kind == "pixels" && c.Warning && c.Test == counterStart && c.SpotPart != "increment" {
					t.Errorf("the worst spot should be in `increment`: %+v", c)
				}
			}
		}},
		// The readiness hook: a component that appears 400 ms after mount fails when the page
		// doesn't say it is busy, and passes when it does (window.omnicuiBusy).
		{"web.harness.late", []string{"check", "web", "counter"}, func(t *testing.T, o outcome) {
			if status(o, "web", counterStart) != check.Fail || !strings.Contains(o.out, "no part ui.counter.value") {
				t.Error("a late component, not declared busy, must fail")
			}
		}},
		{"web.harness.late-busy", []string{"check", "web", "counter"}, func(t *testing.T, o outcome) {
			if o.rep.Summary.Pass != 5 || o.code != 0 {
				t.Errorf("a late component declared busy must be waited for: %+v", o.rep.Summary)
			}
		}},
		{"javafx.host.stretch", []string{"check"}, func(t *testing.T, o outcome) {
			for _, test := range []string{counterStart, counterPress, switchOff, switchOn, switchPress} {
				if !warned(o, "pixels", "javafx", test) {
					t.Errorf("a component drawn in the wrong place must be a warning on %q", test)
				}
			}
			onlyWarningsInvolve(t, o, "javafx")
		}},
		{"web.switch.slow", []string{"check", "switch"}, func(t *testing.T, o outcome) {
			if !warned(o, "timing", "web", switchPress) {
				t.Error("a 300ms linear slide on web must be a timing warning")
			}
			if !warned(o, "motion", "web", switchPress) {
				t.Error("and its clip must move differently too")
			}
			onlyWarningsInvolve(t, o, "web")
		}},
		// A fade is no movement: the position trace can't see it, the clip can.
		{"web.switch.dim", []string{"check", "switch"}, func(t *testing.T, o outcome) {
			if warned(o, "timing", "web", switchPress) {
				t.Error("the thumb still slides the same way: no trace warning")
			}
			if !warned(o, "motion", "web", switchPress) {
				t.Error("a 600 ms fade on web only must make its clip move differently")
			}
			onlyWarningsInvolve(t, o, "web")
		}},
		{"javafx.switch.dim", []string{"check", "switch"}, func(t *testing.T, o outcome) {
			if warned(o, "timing", "javafx", switchPress) || !warned(o, "motion", "javafx", switchPress) {
				t.Error("a fade on javafx only: a motion warning from the clip, no trace warning")
			}
			onlyWarningsInvolve(t, o, "javafx")
		}},
		// A control that ignores Enter: the keyboard test that keyboard: true adds catches it,
		// with no expectation written for it.
		{"javafx.counter.no-enter", []string{"check", "counter"}, func(t *testing.T, o outcome) {
			if status(o, "javafx", "keyboard: Enter on `increment` does what a press does") != check.Fail ||
				status(o, "javafx", "keyboard: Space on `increment` does what a press does") != check.Pass ||
				status(o, "web", "keyboard: Enter on `increment` does what a press does") != check.Pass {
				t.Error("only javafx's Enter test must fail")
			}
			if !strings.Contains(o.out, "javafx: Enter on `increment` didn't do what a press does: a press gave event changed {\"count\":1}, value.text \"1\"; the key gave value.text \"0\"") {
				t.Error("the message must say how the key and the press differ")
			}
		}},
		{"javafx.switch.linear", []string{"check", "switch"}, func(t *testing.T, o outcome) {
			if !warned(o, "timing", "javafx", switchPress) {
				t.Error("timing warning")
			}
			onlyWarningsInvolve(t, o, "javafx")
		}},
		{"egui.switch.linear", []string{"check", "switch"}, func(t *testing.T, o outcome) {
			if !warned(o, "timing", "egui", switchPress) {
				t.Error("timing warning")
			}
			onlyWarningsInvolve(t, o, "egui")
		}},
		{"egui.render.cursor", []string{"check"}, func(t *testing.T, o outcome) {
			if !warned(o, "pixels", "egui", counterPress) {
				t.Error("a mouse pointer painted into egui's screenshot must be a warning")
			}
			onlyWarningsInvolve(t, o, "egui")
		}},
		{"javafx.adapter.stdout", []string{"check"}, func(t *testing.T, o outcome) {
			if o.rep.Summary.Broken != 10 || o.code != 1 || !strings.Contains(o.out, `javafx: broken: stray output on standard output during mount ui.counter: "[javafx] mounting ui.counter"`) {
				t.Errorf("printing on standard output must break javafx, and only javafx: %+v", o.rep.Summary)
			}
			if status(o, "web", counterPress) != check.Pass || status(o, "egui", counterPress) != check.Pass {
				t.Error("the other targets must still pass")
			}
		}},
	}
	for i, tc := range cases {
		t.Run(tc.defect, func(t *testing.T) {
			o := omnicui(t, fmt.Sprintf("%02d-%s", i+2, tc.defect), []string{"OMNICUI_DEFECTS=" + tc.defect}, tc.args...)
			tc.check(t, o)
		})
	}
	// And with the defects switched off again, everything passes once more.
	o := omnicui(t, "99-baseline-again", nil, "check")
	if o.rep.Summary.Pass != 30 || o.rep.Summary.Warnings != 0 {
		t.Fatalf("after the defects: %+v", o.rep.Summary)
	}
}

// R-REAL-4: requiring a capability a target lacks fails the run; requiring what it has doesn't.
func TestRequire(t *testing.T) {
	o := omnicui(t, "20-require", nil, "check", "counter", "--require", "egui:screenshot", "--require", "web:record")
	if o.code != 0 {
		t.Fatalf("all advertised, exit %d", o.code)
	}
	o = omnicui(t, "21-require-missing", []string{"OMNICUI_EGUI_RENDER=off"}, "check", "counter", "egui", "--require", "egui:screenshot")
	if o.code != 1 || !strings.Contains(o.out, "egui must advertise `screenshot` (--require), but it doesn't: rendering switched off") {
		t.Fatalf("exit %d", o.code)
	}
}

// R-REAL-5: a contract change that a component no longer fits is a compile error in the app, in
// both languages that have stubs, before any test runs.
func TestStubsCatchContractChanges(t *testing.T) {
	repo := testutil.RepoRoot()
	tmp := t.TempDir()
	copyTree := func(src, dst string, skip ...string) {
		filepath.WalkDir(src, func(p string, d os.DirEntry, err error) error {
			r, _ := filepath.Rel(src, p)
			for _, s := range skip {
				if r == s {
					return filepath.SkipDir
				}
			}
			out := filepath.Join(dst, r)
			if d.IsDir() {
				return os.MkdirAll(out, 0o755)
			}
			data, _ := os.ReadFile(p)
			return os.WriteFile(out, data, 0o644)
		})
	}
	// A copy of the repository whose counter contract renames `start` to `initial`.
	copyTree(filepath.Join(repo, "omnicui", "contracts"), filepath.Join(tmp, "omnicui", "contracts"))
	counter := filepath.Join(tmp, "omnicui", "contracts", "counter.contract.yaml")
	data, _ := os.ReadFile(counter)
	os.WriteFile(counter, []byte(strings.ReplaceAll(string(data), "start:", "initial:")), 0o644)
	for _, d := range []string{"omnicui/targets/web", "omnicui/targets/egui", "apps/web", "apps/desktop", "shared/fonts"} {
		copyTree(filepath.Join(repo, d), filepath.Join(tmp, d), "node_modules", "dist", "target")
	}
	if err := os.Symlink(filepath.Join(repo, "apps", "web", "node_modules"), filepath.Join(tmp, "apps", "web", "node_modules")); err != nil {
		t.Skipf("can't link node_modules into the copy (on Windows this needs developer mode): %v", err)
	}

	stubs := exec.Command(bin(t), "stubs", "web", "egui")
	stubs.Dir = filepath.Join(tmp, "omnicui")
	stubs.Env = append(os.Environ(), "CARGO_TARGET_DIR="+filepath.Join(repo, "omnicui", "targets", "egui", "target"))
	if out, err := stubs.CombinedOutput(); err != nil {
		t.Fatalf("stubs: %v\n%s", err, out)
	}
	tsc := exec.Command("npx", "tsc", "--noEmit")
	tsc.Dir = filepath.Join(tmp, "apps", "web")
	tscOut, err := tsc.CombinedOutput()
	if err == nil || !strings.Contains(string(tscOut), "start") {
		t.Errorf("TypeScript must reject Counter.tsx: %v\n%s", err, tscOut)
	}
	cargo := exec.Command("cargo", "check", "--release", "--locked", "--lib", "--no-default-features")
	cargo.Dir = filepath.Join(tmp, "apps", "desktop")
	cargo.Env = append(os.Environ(), "CARGO_TARGET_DIR="+filepath.Join(repo, "apps", "desktop", "target"))
	cargoOut, err := cargo.CombinedOutput()
	if err == nil || !strings.Contains(string(cargoOut), "no field `start`") {
		t.Errorf("rustc must reject counter.rs: %v\n%s", err, cargoOut)
	}
	if dir := os.Getenv("OMNICUI_EVIDENCE"); dir != "" {
		os.WriteFile(filepath.Join(dir, "30-stubs-catch-contract-changes.txt"), []byte(
			"The counter contract's prop `start` renamed to `initial`, then `omnicui stubs web egui`.\n\n"+
				"$ npx tsc --noEmit                                       (apps/web)\n"+tail(string(tscOut))+
				"\n$ cargo check --release --lib --no-default-features  (apps/desktop)\n"+tail(string(cargoOut))), 0o644)
	}
}

func tail(s string) string {
	lines := strings.Split(strings.TrimSpace(s), "\n")
	if len(lines) > 30 {
		lines = lines[len(lines)-30:]
	}
	return strings.Join(lines, "\n") + "\n"
}
