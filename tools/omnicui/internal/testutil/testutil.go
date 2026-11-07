// Package testutil builds the fake adapter and temporary projects for omnicui's tests.
package testutil

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"sort"
	"strings"
	"sync"
	"testing"
)

var (
	buildOnce sync.Once
	fakePath  string
	buildErr  error
)

// ModuleRoot is the omnicui module's folder.
func ModuleRoot() string {
	_, file, _, _ := runtime.Caller(0)
	return filepath.Clean(filepath.Join(filepath.Dir(file), "..", ".."))
}

// RepoRoot is the reference repository (app/): apps/, omnicui/, shared/, tools/omnicui/.
func RepoRoot() string { return filepath.Clean(filepath.Join(ModuleRoot(), "..", "..")) }

// ProjectRoot is the reference repository's omnicui project, with the example contracts and targets.
func ProjectRoot() string { return filepath.Join(RepoRoot(), "omnicui") }

// FakeAdapter builds testdata/fakeadapter once per test process and returns its path.
func FakeAdapter(t testing.TB) string {
	t.Helper()
	buildOnce.Do(func() {
		dir, err := os.MkdirTemp("", "omnicui-fake-")
		if err != nil {
			buildErr = err
			return
		}
		fakePath = filepath.Join(dir, "fakeadapter")
		if runtime.GOOS == "windows" {
			fakePath += ".exe"
		}
		cmd := exec.Command("go", "build", "-o", fakePath, "./testdata/fakeadapter")
		cmd.Dir = ModuleRoot()
		if out, err := cmd.CombinedOutput(); err != nil {
			buildErr = fmt.Errorf("build the fake adapter: %v\n%s", err, out)
		}
	})
	if buildErr != nil {
		t.Fatal(buildErr)
	}
	return fakePath
}

// Target describes one fake target in a temporary project.
type Target struct {
	Env   map[string]string
	Extra string // more target.yaml lines; FAKE is replaced by the fake adapter's quoted path
}

// Command is the shell command that runs the fake adapter with these settings, on any OS.
func Command(t testing.TB, dir string, env map[string]string) string {
	t.Helper()
	var lines []string
	for k, v := range env {
		lines = append(lines, k+"="+v)
	}
	sort.Strings(lines)
	config := filepath.Join(dir, "fake-config.txt")
	if err := os.WriteFile(config, []byte(strings.Join(lines, "\n")+"\n"), 0o644); err != nil {
		t.Fatal(err)
	}
	return fmt.Sprintf("exec %q %q", FakeAdapter(t), config)
}

// Project creates a temporary project with the example contracts and the given fake targets.
func Project(t testing.TB, targets map[string]Target) string {
	t.Helper()
	root := t.TempDir()
	os.MkdirAll(filepath.Join(root, "contracts"), 0o755)
	files, _ := filepath.Glob(filepath.Join(ProjectRoot(), "contracts", "*.contract.yaml"))
	for _, f := range files {
		data, err := os.ReadFile(f)
		if err != nil {
			t.Fatal(err)
		}
		os.WriteFile(filepath.Join(root, "contracts", filepath.Base(f)), data, 0o644)
	}
	for name, tg := range targets {
		dir := filepath.Join(root, "targets", name)
		os.MkdirAll(dir, 0o755)
		adapter := Command(t, dir, tg.Env)
		extra := strings.ReplaceAll(tg.Extra, "FAKE", fmt.Sprintf("%q", FakeAdapter(t)))
		body := fmt.Sprintf("name: %s\nlanguage: go\nadapter: '%s'\nstartTimeout: 5s\n%s", name, adapter, extra)
		if !strings.Contains(tg.Extra, "stepTimeout") {
			body += "stepTimeout: 2s\n"
		}
		if err := os.WriteFile(filepath.Join(dir, "target.yaml"), []byte(body), 0o644); err != nil {
			t.Fatal(err)
		}
	}
	return root
}
