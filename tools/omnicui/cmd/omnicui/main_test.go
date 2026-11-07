package main

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"omnicui/internal/testutil"
)

func omni(t *testing.T, dir string, args ...string) (int, string, string) {
	t.Helper()
	wd, _ := os.Getwd()
	os.Chdir(dir)
	defer os.Chdir(wd)
	var out, errb strings.Builder
	code := run(args, &out, &errb)
	return code, out.String(), errb.String()
}

// R-CLI-1: with no command, omnicui prints its usage and exits 2; `help` exits 0.
func TestUsage(t *testing.T) {
	if code, out, _ := omni(t, t.TempDir()); code != 2 || !strings.Contains(out, "usage:") {
		t.Fatalf("%d %s", code, out)
	}
	if code, _, _ := omni(t, t.TempDir(), "help"); code != 0 {
		t.Fatal(code)
	}
	if code, out, _ := omni(t, t.TempDir(), "version"); code != 0 || !strings.HasPrefix(out, "omnicui ") {
		t.Fatal(code, out)
	}
}

// R-CLI-2: outside a project, or with an unknown command or option, omnicui exits 2.
func TestUsageErrors(t *testing.T) {
	if code, _, errs := omni(t, t.TempDir(), "check"); code != 2 || !strings.Contains(errs, "no contracts/ folder") {
		t.Fatal(code, errs)
	}
	root := testutil.Project(t, map[string]testutil.Target{"a": {}})
	if code, _, errs := omni(t, root, "frobnicate"); code != 2 || !strings.Contains(errs, "unknown command") {
		t.Fatal(code, errs)
	}
	if code, _, errs := omni(t, root, "check", "--fast"); code != 2 || !strings.Contains(errs, "unknown option --fast") {
		t.Fatal(code, errs)
	}
	if code, _, errs := omni(t, root, "check", "nope"); code != 2 || !strings.Contains(errs, "`nope` is neither a target nor a component (known: a, counter, switch)") {
		t.Fatal(code, errs)
	}
}

// R-CLI-3: `check` exits 0 when nothing failed, 1 when something did, and prints the summary.
func TestCheckExitCodes(t *testing.T) {
	root := testutil.Project(t, map[string]testutil.Target{"a": {}, "b": {}})
	code, out, _ := omni(t, root, "check")
	if code != 0 || !strings.Contains(out, "14 pass · 0 fail · 0 broken · 0 not advertised · 0 warnings") {
		t.Fatalf("%d\n%s", code, out)
	}
	root = testutil.Project(t, map[string]testutil.Target{"a": {Env: map[string]string{"FAKE_BUG": "no-name"}}})
	code, out, _ = omni(t, root, "check", "counter")
	if code != 1 || !strings.Contains(out, `a: increment.name: expected "Increment", got nothing`) || strings.Contains(out, "ui.switch") {
		t.Fatalf("%d\n%s", code, out)
	}
}

// R-CLI-4: a contract error stops everything before any target runs, and exits 2.
func TestContractErrorsStop(t *testing.T) {
	root := testutil.Project(t, map[string]testutil.Target{"a": {}})
	path := filepath.Join(root, "contracts", "counter.contract.yaml")
	data, _ := os.ReadFile(path)
	os.WriteFile(path, []byte(strings.Replace(string(data), "part: increment, times: 3", "part: incremnt, times: 3", 1)), 0o644)
	code, _, errs := omni(t, root, "check")
	if code != 2 || !strings.Contains(errs, "contracts/counter.contract.yaml:24  error: part `incremnt` does not exist; did you mean `increment`?") ||
		!strings.Contains(errs, "no target was run") {
		t.Fatalf("%d %s", code, errs)
	}
	if _, err := os.Stat(filepath.Join(root, ".omnicui")); err == nil {
		t.Error("a run folder was written although no target ran")
	}
	if code, _, _ := omni(t, root, "contracts"); code != 2 {
		t.Error("contracts must exit 2 too")
	}
}

// R-CLI-5: `add` writes only target.yaml and lists the constructs to implement; it refuses
// names that exist or aren't valid; `remove` deletes the folder.
func TestAddRemove(t *testing.T) {
	root := testutil.Project(t, nil)
	code, out, _ := omni(t, root, "add", "javafx")
	data, err := os.ReadFile(filepath.Join(root, "targets", "javafx", "target.yaml"))
	if code != 0 || err != nil || !strings.HasPrefix(string(data), "name: javafx\n") || !strings.Contains(out, "screenshot") || !strings.Contains(out, "JSON-RPC 2.0") {
		t.Fatalf("%d %v\n%s", code, err, out)
	}
	entries, _ := os.ReadDir(filepath.Join(root, "targets", "javafx"))
	if len(entries) != 1 {
		t.Errorf("add must write only target.yaml, got %d files", len(entries))
	}
	if code, _, errs := omni(t, root, "add", "javafx"); code != 2 || !strings.Contains(errs, "already exists") {
		t.Error(code, errs)
	}
	if code, _, errs := omni(t, root, "add", "Java FX"); code != 2 || !strings.Contains(errs, "lower-case") {
		t.Error(code, errs)
	}
	if code, _, _ := omni(t, root, "targets"); code != 0 {
		t.Error("targets must cope with an empty adapter command")
	}
	if code, _, _ := omni(t, root, "remove", "javafx"); code != 0 {
		t.Error(code)
	}
	if _, err := os.Stat(filepath.Join(root, "targets", "javafx")); err == nil {
		t.Error("remove left the folder")
	}
	if code, _, _ := omni(t, root, "remove", "javafx"); code != 2 {
		t.Error("removing a missing target must fail")
	}
}

// R-CLI-6: `add --template` copies a finished target, renames it, and leaves out build output.
func TestAddTemplate(t *testing.T) {
	root := testutil.Project(t, map[string]testutil.Target{"base": {}})
	base := filepath.Join(root, "targets", "base")
	os.MkdirAll(filepath.Join(base, "node_modules", "x"), 0o755)
	os.WriteFile(filepath.Join(base, "node_modules", "x", "big.js"), []byte("x"), 0o644)
	os.WriteFile(filepath.Join(base, "Component.java"), []byte("class C {}"), 0o644)
	code, _, errs := omni(t, root, "add", "copy", "--template", base)
	if code != 0 {
		t.Fatal(errs)
	}
	data, _ := os.ReadFile(filepath.Join(root, "targets", "copy", "target.yaml"))
	if !strings.HasPrefix(string(data), "name: copy\n") {
		t.Errorf("target.yaml: %s", data)
	}
	if _, err := os.Stat(filepath.Join(root, "targets", "copy", "Component.java")); err != nil {
		t.Error(err)
	}
	if _, err := os.Stat(filepath.Join(root, "targets", "copy", "node_modules")); err == nil {
		t.Error("node_modules was copied")
	}
	if code, out, _ := omni(t, root, "check", "copy", "counter"); code != 0 || !strings.Contains(out, "3 pass") {
		t.Errorf("the copy must work as a target: %d\n%s", code, out)
	}
}

// R-CLI-7: `targets` shows each target's capabilities, with the reason for what it can't do.
func TestTargets(t *testing.T) {
	root := testutil.Project(t, map[string]testutil.Target{"a": {}, "b": {Env: map[string]string{"FAKE_CANNOT": "screenshot=no display"}}, "c": {Env: map[string]string{"FAKE_CRASH_ON": "1"}}})
	code, out, _ := omni(t, root, "targets")
	if code != 0 || !strings.Contains(out, "– no display") || !strings.Contains(out, "c         broken: the adapter exited with code 3 during advertise") {
		t.Fatalf("%d\n%s", code, out)
	}
}

// R-CLI-9: the project is found from a repository's root when it lives in omnicui/, and
// --project (or OMNICUI_PROJECT) names it from anywhere; --out puts the results elsewhere.
func TestProjectAndOut(t *testing.T) {
	root := testutil.Project(t, map[string]testutil.Target{"a": {}})
	repo := t.TempDir()
	if err := os.Rename(root, filepath.Join(repo, "omnicui")); err != nil {
		t.Fatal(err)
	}
	if code, out, errs := omni(t, repo, "contracts"); code != 0 || !strings.Contains(out, "contracts are fine") {
		t.Fatal(code, out, errs)
	}
	elsewhere := t.TempDir()
	out := filepath.Join(t.TempDir(), "results")
	code, stdout, errs := omni(t, elsewhere, "--project", repo, "check", "counter", "--out", out)
	if code != 0 {
		t.Fatal(code, stdout, errs)
	}
	if _, err := os.Stat(filepath.Join(out, "results.json")); err != nil {
		t.Error("no results.json in --out:", err)
	}
	if !strings.Contains(stdout, filepath.Join(out, "report.html")) {
		t.Error("the printed report path is not the --out folder:\n" + stdout)
	}
	t.Setenv("OMNICUI_PROJECT", filepath.Join(repo, "omnicui"))
	if code, _, errs := omni(t, elsewhere, "contracts"); code != 0 {
		t.Fatal(code, errs)
	}
	if code, _, errs := omni(t, elsewhere, "--project", elsewhere, "contracts"); code != 2 || !strings.Contains(errs, "is not an omnicui project") {
		t.Fatal(code, errs)
	}
	if code, out, _ := omni(t, elsewhere, "--version"); code != 0 || !strings.HasPrefix(out, "omnicui dev (protocol 1, ") {
		t.Fatal(code, out)
	}
	if code, out, _ := omni(t, repo, "check", "--help"); code != 0 || !strings.Contains(out, "--project <folder>") {
		t.Fatal(code, out)
	}
}
