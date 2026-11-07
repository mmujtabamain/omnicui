// Package project finds a project's contracts and targets, and adds or removes targets.
package project

import (
	"errors"
	"fmt"
	"io/fs"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"sort"
	"strings"
	"time"

	"gopkg.in/yaml.v3"
)

// Project is a folder holding contracts/ and targets/.
type Project struct {
	Root string
	// Out, when set, is where a check writes its results instead of .omnicui/run/.
	Out string
}

// Contracts is the folder of contract files.
func (p Project) Contracts() string { return filepath.Join(p.Root, "contracts") }

// TargetsDir is the folder of targets.
func (p Project) TargetsDir() string { return filepath.Join(p.Root, "targets") }

// RunDir is where a check writes its results. It is replaced by every run.
func (p Project) RunDir() string {
	if p.Out != "" {
		return p.Out
	}
	return filepath.Join(p.Root, ".omnicui", "run")
}

// at returns the project in folder d: d itself when it has contracts/, or d/omnicui when that has
// (a repository that keeps its omnicui project in omnicui/, next to its apps).
func at(d string) (Project, bool) {
	for _, root := range []string{d, filepath.Join(d, "omnicui")} {
		if st, err := os.Stat(filepath.Join(root, "contracts")); err == nil && st.IsDir() {
			return Project{Root: root}, true
		}
	}
	return Project{}, false
}

// Find walks up from dir to the first folder that has contracts/ (or omnicui/contracts/).
func Find(dir string) (Project, error) {
	d, err := filepath.Abs(dir)
	if err != nil {
		return Project{}, err
	}
	for {
		if p, ok := at(d); ok {
			return p, nil
		}
		parent := filepath.Dir(d)
		if parent == d {
			return Project{}, errors.New("no contracts/ folder (or omnicui/contracts/) here or in any parent folder; run omnicui inside a project, or name one with --project <folder>")
		}
		d = parent
	}
}

// Open is the project in dir exactly (dir, or dir/omnicui), as named with --project.
func Open(dir string) (Project, error) {
	d, err := filepath.Abs(dir)
	if err != nil {
		return Project{}, err
	}
	if p, ok := at(d); ok {
		return p, nil
	}
	return Project{}, fmt.Errorf("%s is not an omnicui project: it has no contracts/ folder (or omnicui/contracts/)", dir)
}

// Target is one targets/<name>/target.yaml.
type Target struct {
	Name     string `yaml:"name"`
	Language string `yaml:"language"`
	Adapter  string `yaml:"adapter"`
	Stubs    string `yaml:"stubs"`
	// Timeouts, as Go durations ("10s"). Start covers the adapter starting and answering advertise.
	StepTimeout  string `yaml:"stepTimeout"`
	StartTimeout string `yaml:"startTimeout"`

	Dir string `yaml:"-"`
}

// Timeouts with their defaults.
func (t Target) Timeouts() (step, start time.Duration) {
	step, start = 10*time.Second, 120*time.Second
	if d, err := time.ParseDuration(t.StepTimeout); err == nil && d > 0 {
		step = d
	}
	if d, err := time.ParseDuration(t.StartTimeout); err == nil && d > 0 {
		start = d
	}
	return
}

var nameRE = regexp.MustCompile(`^[a-z][a-z0-9-]*$`)

// Targets loads every targets/*/target.yaml, in name order.
func (p Project) Targets() ([]Target, error) {
	entries, err := os.ReadDir(p.TargetsDir())
	if errors.Is(err, fs.ErrNotExist) {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}
	var out []Target
	for _, e := range entries {
		if !e.IsDir() {
			continue
		}
		dir := filepath.Join(p.TargetsDir(), e.Name())
		data, err := os.ReadFile(filepath.Join(dir, "target.yaml"))
		if errors.Is(err, fs.ErrNotExist) {
			continue
		}
		if err != nil {
			return nil, err
		}
		var t Target
		if err := yaml.Unmarshal(data, &t); err != nil {
			return nil, fmt.Errorf("targets/%s/target.yaml: %v", e.Name(), err)
		}
		if t.Name == "" {
			t.Name = e.Name()
		}
		if t.Name != e.Name() {
			return nil, fmt.Errorf("targets/%s/target.yaml: name `%s` must match its folder `%s`", e.Name(), t.Name, e.Name())
		}
		t.Dir = dir
		out = append(out, t)
	}
	sort.Slice(out, func(i, j int) bool { return out[i].Name < out[j].Name })
	return out, nil
}

// Constructs is the fixture language, printed by `add` as the list to implement.
var Constructs = []struct{ Name, Does string }{
	{"advertise", "first request: reply with the constructs this target can perform on this run"},
	{"mount", "show a component, fresh (error -32001 if the target has no such component)"},
	{"press, focus, key, type, drag, hover", "perform the action on a part (error -32002 if there is no such part)"},
	{"read", "the value of one field of a part: text, name, disabled, checked, value"},
	{"events", "the events emitted since the last call"},
	{"screenshot", "write an image of what is shown to the given path (if the target can render)"},
	{"trace", "perform steps while sampling one point of a part; return its position over time"},
	{"record", "perform steps while writing a clip's frames to the given folder; never compared"},
}

// Add creates targets/<name>. With a template it copies that folder or clones that git URL.
func (p Project) Add(name, template string) (string, error) {
	if !nameRE.MatchString(name) {
		return "", fmt.Errorf("a target name is lower-case letters, digits and dashes, starting with a letter: `%s` is not", name)
	}
	dir := filepath.Join(p.TargetsDir(), name)
	if _, err := os.Stat(dir); err == nil {
		return "", fmt.Errorf("targets/%s already exists", name)
	}
	if err := os.MkdirAll(p.TargetsDir(), 0o755); err != nil {
		return "", err
	}
	if template != "" {
		if isGitURL(template) {
			cmd := exec.Command("git", "clone", "--depth", "1", template, dir)
			if out, err := cmd.CombinedOutput(); err != nil {
				return "", fmt.Errorf("git clone %s: %v\n%s", template, err, out)
			}
			os.RemoveAll(filepath.Join(dir, ".git"))
		} else if err := copyDir(template, dir); err != nil {
			return "", err
		}
		return dir, setName(dir, name)
	}
	if err := os.MkdirAll(dir, 0o755); err != nil {
		return "", err
	}
	body := fmt.Sprintf("name: %s\nlanguage: \nadapter:        # the shell command that starts this target's adapter\n# stubs:        # optional: a command that reads the contracts as JSON on stdin and writes types for your app\n", name)
	return dir, os.WriteFile(filepath.Join(dir, "target.yaml"), []byte(body), 0o644)
}

func isGitURL(s string) bool {
	return strings.HasPrefix(s, "https://") || strings.HasPrefix(s, "git@") || strings.HasSuffix(s, ".git")
}

func setName(dir, name string) error {
	path := filepath.Join(dir, "target.yaml")
	data, err := os.ReadFile(path)
	if err != nil {
		return fmt.Errorf("the template has no target.yaml")
	}
	re := regexp.MustCompile(`(?m)^name:.*$`)
	if re.Match(data) {
		data = re.ReplaceAll(data, []byte("name: "+name))
	} else {
		data = append([]byte("name: "+name+"\n"), data...)
	}
	return os.WriteFile(path, data, 0o644)
}

// copyDir copies a template, leaving out build output and installed dependencies.
func copyDir(src, dst string) error {
	skip := map[string]bool{"node_modules": true, "target": true, "generated": true, "dist": true, ".git": true, "build": true}
	return filepath.WalkDir(src, func(path string, d fs.DirEntry, err error) error {
		if err != nil {
			return err
		}
		r, _ := filepath.Rel(src, path)
		if d.IsDir() && skip[d.Name()] && r != "." {
			return filepath.SkipDir
		}
		out := filepath.Join(dst, r)
		if d.IsDir() {
			return os.MkdirAll(out, 0o755)
		}
		info, err := d.Info()
		if err != nil {
			return err
		}
		data, err := os.ReadFile(path)
		if err != nil {
			return err
		}
		return os.WriteFile(out, data, info.Mode().Perm())
	})
}

// Remove deletes targets/<name>.
func (p Project) Remove(name string) error {
	if !nameRE.MatchString(name) {
		return fmt.Errorf("`%s` is not a target name", name)
	}
	dir := filepath.Join(p.TargetsDir(), name)
	if _, err := os.Stat(filepath.Join(dir, "target.yaml")); err != nil {
		return fmt.Errorf("there is no target `%s`", name)
	}
	return os.RemoveAll(dir)
}
