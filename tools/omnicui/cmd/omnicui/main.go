// Command omnicui checks hand-written versions of the same UI component, in different stacks,
// against one contract.
package main

import (
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"log/slog"
	"os"
	"os/signal"
	"path/filepath"
	"runtime"
	"runtime/debug"
	"strings"
	"sync"
	"syscall"

	"omnicui/internal/check"
	"omnicui/internal/compare"
	"omnicui/internal/contract"
	"omnicui/internal/project"
	"omnicui/internal/rpc"
)

const usage = `omnicui checks hand-written versions of a UI component, in different stacks, against one contract.

usage:
  omnicui [--project <folder>] <command> …

  omnicui check [target…] [component…] [--test <name>]… [--failed] [--serial]
                [--require target:capability]… [--out <folder>] [--fail-on warnings|pixels|palette|timing|direction|motion|visibility]…
                                       run every test of every contract on every target (or only those named);
                                       results go to .omnicui/run/ in the project, or to --out. One progress
                                       line per test, with its time, goes to standard error, and a line every
                                       10 s while a target starts.
                                       --test: only tests whose name has this in it (or matches it, with *)
                                       --failed: only the tests that failed, broke or warned in the last run
                                       --serial: one target at a time (or exclusive: true in a target.yaml)
                                       --fail-on: those warnings fail the run (exit 1), for CI
  omnicui targets                      what each target advertises on this machine
  omnicui stubs [target…]              run the targets' stub generators
  omnicui contracts                    load and check the contracts only
  omnicui add <name> [--template <folder or git URL>]
                                       add a target
  omnicui remove <name>                remove a target
  omnicui compare <a.png> <b.png> [--diff <out.png>] [--json]
                                       measure two screenshots the way check does
  omnicui help contract                what a contract can say, its defaults, and how targets are compared
  omnicui version

The project is the folder that has contracts/ and targets/. omnicui finds it from the current
folder: that folder or any above it, or omnicui/ inside one of them (a repository that keeps its
project in omnicui/, next to its apps). --project <folder>, or OMNICUI_PROJECT, names it instead.

Exit status: 0 when nothing failed or broke, 1 when something did (or a --fail-on warning), 2 for a
usage or contract error, 130 when interrupted (every adapter it started is stopped first).`

// Version is set by release builds with -ldflags "-X main.Version=1.2.3". Without it, the module
// version that `go install …@v1.2.3` records is used, and otherwise "dev".
var Version = ""

func version() string {
	if Version != "" {
		return Version
	}
	if bi, ok := debug.ReadBuildInfo(); ok && bi.Main.Version != "" && bi.Main.Version != "(devel)" {
		return strings.TrimPrefix(bi.Main.Version, "v")
	}
	return "dev"
}

func main() {
	// Ctrl+C or a CI runner's SIGTERM: adapters run in process groups of their own, so they don't
	// get the signal. Stop every one of them (a browser, a JVM, cargo's child) before exiting.
	sig := make(chan os.Signal, 1)
	signal.Notify(sig, os.Interrupt, syscall.SIGTERM)
	go func() {
		s := <-sig
		n := rpc.KillAll()
		fmt.Fprintf(os.Stderr, "\nomnicui: %v; stopped %d adapter(s)\n", s, n)
		os.Exit(130)
	}()
	os.Exit(run(os.Args[1:], os.Stdout, os.Stderr))
}

func run(args []string, stdout, stderr io.Writer) int {
	projectDir := os.Getenv("OMNICUI_PROJECT")
	for len(args) > 0 && (args[0] == "--project" || strings.HasPrefix(args[0], "--project=")) {
		if v, ok := strings.CutPrefix(args[0], "--project="); ok {
			projectDir, args = v, args[1:]
		} else if len(args) > 1 {
			projectDir, args = args[1], args[2:]
		} else {
			fmt.Fprintln(stderr, "omnicui: --project needs a folder")
			return 2
		}
	}
	if len(args) > 0 && (args[0] == "--version" || args[0] == "-v") {
		args = []string{"version"}
	}
	if len(args) == 2 && args[0] == "help" && args[1] == "contract" {
		fmt.Fprint(stdout, contractHelp())
		return 0
	}
	if len(args) == 0 || args[0] == "help" || args[0] == "-h" || args[0] == "--help" {
		fmt.Fprintln(stdout, usage)
		if len(args) == 0 {
			return 2
		}
		return 0
	}
	cmd, rest := args[0], args[1:]
	if cmd == "version" {
		fmt.Fprintf(stdout, "omnicui %s (protocol %s, %s/%s, %s)\n", version(), rpc.Protocol, runtime.GOOS, runtime.GOARCH, runtime.Version())
		return 0
	}
	if len(rest) > 0 && (rest[0] == "-h" || rest[0] == "--help") {
		fmt.Fprintln(stdout, usage)
		return 0
	}
	if cmd == "compare" {
		return doCompare(rest, stdout, stderr)
	}
	var p project.Project
	var err error
	if projectDir != "" {
		p, err = project.Open(projectDir)
	} else {
		wd, _ := os.Getwd()
		p, err = project.Find(wd)
	}
	if err != nil {
		fmt.Fprintln(stderr, "omnicui:", err)
		return 2
	}
	switch cmd {
	case "check":
		return doCheck(p, rest, stdout, stderr)
	case "targets":
		return doTargets(p, stdout, stderr)
	case "stubs":
		return doStubs(p, rest, stdout, stderr)
	case "contracts":
		_, problems, code := loadContracts(p, stderr)
		if code == 0 {
			fmt.Fprintf(stdout, "contracts are fine (%d warnings)\n", len(problems))
		}
		return code
	case "add":
		return doAdd(p, rest, stdout, stderr)
	case "remove":
		if len(rest) != 1 {
			fmt.Fprintln(stderr, "usage: omnicui remove <name>")
			return 2
		}
		if err := p.Remove(rest[0]); err != nil {
			fmt.Fprintln(stderr, "omnicui:", err)
			return 2
		}
		fmt.Fprintf(stdout, "removed targets/%s\n", rest[0])
		return 0
	}
	fmt.Fprintf(stderr, "omnicui: unknown command %q\n\n%s\n", cmd, usage)
	return 2
}

func loadContracts(p project.Project, stderr io.Writer) ([]*contract.Contract, []contract.Problem, int) {
	cs, problems, err := contract.LoadDir(p.Contracts())
	if err != nil {
		fmt.Fprintln(stderr, "omnicui:", err)
		return nil, nil, 2
	}
	for _, pr := range problems {
		fmt.Fprintln(stderr, pr)
	}
	if contract.HasErrors(problems) {
		fmt.Fprintln(stderr, "omnicui: fix the contracts first; no target was run")
		return nil, problems, 2
	}
	if len(cs) == 0 {
		fmt.Fprintln(stderr, "omnicui: there are no *.contract.yaml files in contracts/")
		return nil, problems, 2
	}
	return cs, problems, 0
}

// selection splits names into targets and components; anything else is an error.
func selection(names []string, targets []project.Target, contracts []*contract.Contract) ([]project.Target, []*contract.Contract, error) {
	var ts []project.Target
	var cs []*contract.Contract
	for _, n := range names {
		matched := false
		for _, t := range targets {
			if t.Name == n {
				ts, matched = append(ts, t), true
			}
		}
		for _, c := range contracts {
			if !matched && (c.ID == n || c.Short() == n) {
				cs, matched = append(cs, c), true
			}
		}
		if !matched {
			var known []string
			for _, t := range targets {
				known = append(known, t.Name)
			}
			for _, c := range contracts {
				known = append(known, c.Short())
			}
			return nil, nil, fmt.Errorf("`%s` is neither a target nor a component (known: %s)", n, strings.Join(known, ", "))
		}
	}
	if len(ts) == 0 {
		ts = targets
	}
	if len(cs) == 0 {
		cs = contracts
	}
	return ts, cs, nil
}

func doCheck(p project.Project, args []string, stdout, stderr io.Writer) int {
	var names, require, failOn, tests []string
	failed, serial := false, false
	for i := 0; i < len(args); i++ {
		switch {
		case args[i] == "--test" && i+1 < len(args):
			tests = append(tests, args[i+1])
			i++
		case strings.HasPrefix(args[i], "--test="):
			tests = append(tests, strings.TrimPrefix(args[i], "--test="))
		case args[i] == "--failed":
			failed = true
		case args[i] == "--serial":
			serial = true
		case args[i] == "--fail-on" && i+1 < len(args):
			failOn = append(failOn, strings.Split(args[i+1], ",")...)
			i++
		case strings.HasPrefix(args[i], "--fail-on="):
			failOn = append(failOn, strings.Split(strings.TrimPrefix(args[i], "--fail-on="), ",")...)
		case args[i] == "--require" && i+1 < len(args):
			require = append(require, args[i+1])
			i++
		case strings.HasPrefix(args[i], "--require="):
			require = append(require, strings.TrimPrefix(args[i], "--require="))
		case args[i] == "--out" && i+1 < len(args):
			p.Out = absolute(args[i+1])
			i++
		case strings.HasPrefix(args[i], "--out="):
			p.Out = absolute(strings.TrimPrefix(args[i], "--out="))
		case strings.HasPrefix(args[i], "-"):
			fmt.Fprintf(stderr, "omnicui: unknown option %s\n", args[i])
			return 2
		default:
			names = append(names, args[i])
		}
	}
	for _, f := range failOn {
		if !check.FailOnKinds[f] {
			fmt.Fprintf(stderr, "omnicui: --fail-on %s: expected warnings, pixels, palette, timing, direction, motion or visibility\n", f)
			return 2
		}
	}
	contracts, problems, code := loadContracts(p, stderr)
	if code != 0 {
		return code
	}
	targets, err := p.Targets()
	if err != nil {
		fmt.Fprintln(stderr, "omnicui:", err)
		return 2
	}
	if len(targets) == 0 {
		fmt.Fprintln(stderr, "omnicui: there are no targets; add one with `omnicui add <name>`")
		return 2
	}
	// Platform selectors that match no form, OS, target or design: probably a typo.
	var known []string
	for _, t := range targets {
		known = append(known, t.Name)
		if t.Design != "" {
			known = append(known, t.Design)
		}
	}
	for _, pr := range contract.UnknownSelectors(contracts, known) {
		fmt.Fprintln(stderr, pr)
		problems = append(problems, pr)
	}
	ts, cs, err := selection(names, targets, contracts)
	if err != nil {
		fmt.Fprintln(stderr, "omnicui:", err)
		return 2
	}
	if failed {
		last, err := check.LastProblems(filepath.Join(p.RunDir(), "results.json"))
		if err != nil {
			fmt.Fprintln(stderr, "omnicui: --failed:", err)
			return 2
		}
		if len(last) == 0 {
			fmt.Fprintln(stdout, "nothing to rerun: in the last run no test failed, broke or warned")
			return 0
		}
		cs = check.FilterTests(cs, func(c *contract.Contract, t contract.Test) bool { return last[c.ID+"\x00"+t.Name] })
	}
	if len(tests) > 0 {
		all := cs
		cs = check.FilterTests(cs, func(c *contract.Contract, t contract.Test) bool { return check.TestMatches(t.Name, tests) })
		if len(cs) == 0 {
			fmt.Fprintf(stderr, "omnicui: --test %s matches no test%s\n", strings.Join(tests, ", "), check.NearestTests(all, tests))
			return 2
		}
	}
	if (failed || len(tests) > 0) && len(cs) == 0 {
		fmt.Fprintln(stdout, "nothing to rerun: the tests that failed last time aren't in this selection")
		return 0
	}
	rep, err := check.Run(check.Options{Project: p, Contracts: cs, All: contracts, Problems: problems, Targets: ts, Require: require, Progress: stderr, Serial: serial})
	if err != nil {
		fmt.Fprintln(stderr, "omnicui:", err)
		return 2
	}
	check.Print(stdout, rep, p.RunDir())
	if rep.Failed() {
		return 1
	}
	if n, kinds := rep.WarningsFailing(failOn); n > 0 {
		fmt.Fprintf(stdout, "exit 1: --fail-on %s, and there are %d such warning(s)\n", strings.Join(kinds, ","), n)
		return 1
	}
	return 0
}

func absolute(path string) string {
	if a, err := filepath.Abs(path); err == nil {
		return a
	}
	return path
}

func doTargets(p project.Project, stdout, stderr io.Writer) int {
	targets, err := p.Targets()
	if err != nil {
		fmt.Fprintln(stderr, "omnicui:", err)
		return 2
	}
	type row struct {
		name, broken, render string
		can                  map[string]bool
		cannot               map[string]string
	}
	// The render check runs at the contracts' largest viewport, as in a check.
	contracts, _, _ := contract.LoadDir(p.Contracts())
	vp := check.ProbeViewport(contracts)
	probeDir, _ := os.MkdirTemp("", "omnicui-targets-")
	defer os.RemoveAll(probeDir)
	rows := make([]row, len(targets))
	var wg sync.WaitGroup
	quiet := slog.New(slog.NewTextHandler(io.Discard, nil))
	for i, t := range targets {
		wg.Add(1)
		go func(i int, t project.Target) {
			defer wg.Done()
			rows[i] = row{name: t.Name, can: map[string]bool{}}
			if strings.TrimSpace(t.Adapter) == "" {
				rows[i].broken = "no adapter command in target.yaml yet"
				return
			}
			_, start := t.Timeouts()
			c, err := rpc.Start(rpc.Options{Target: t.Name, Dir: t.Dir, Command: t.Adapter, Log: quiet})
			if err != nil {
				rows[i].broken = err.Error()
				return
			}
			defer c.Stop()
			var ad struct {
				Can    []string          `json:"can"`
				Cannot map[string]string `json:"cannot"`
			}
			if err := c.Call("advertise", map[string]any{"protocol": rpc.Protocol}, start, &ad); err != nil {
				var b *rpc.BrokenError
				if errors.As(err, &b) {
					rows[i].broken = b.Reason
				} else {
					rows[i].broken = err.Error()
				}
				return
			}
			for _, x := range ad.Can {
				rows[i].can[x] = true
			}
			rows[i].cannot = ad.Cannot
			if rows[i].can["screenshot"] {
				if !rows[i].can["probe"] {
					rows[i].render = "not checked (the target doesn't advertise probe)"
				} else if err := check.RenderCheck(c, vp, filepath.Join(probeDir, t.Name+".png"), start); err != nil {
					rows[i].render = "FAILED: " + err.Error()
					rows[i].can["screenshot"] = false
					if rows[i].cannot == nil {
						rows[i].cannot = map[string]string{}
					}
					rows[i].cannot["screenshot"] = "render check failed"
				} else {
					rows[i].render = fmt.Sprintf("passed at %d×%d", vp.Width, vp.Height)
				}
			}
		}(i, t)
	}
	wg.Wait()
	cell := func(r row, capability string) string {
		if r.can[capability] {
			return "✓"
		}
		if why, ok := r.cannot[capability]; ok {
			return "– " + why
		}
		return "–"
	}
	fmt.Fprintf(stdout, "%-10s%-22s%-22s%-22s%s\n", "target", "screenshot", "trace", "record", "actions")
	for _, r := range rows {
		if r.broken != "" {
			fmt.Fprintf(stdout, "%-10sbroken: %s\n", r.name, r.broken)
			continue
		}
		var actions []string
		for _, a := range []string{"press", "focus", "key", "type", "drag", "hover"} {
			if r.can[a] {
				actions = append(actions, a)
			}
		}
		fmt.Fprintf(stdout, "%-10s%-22s%-22s%-22s%s\n", r.name, clip(cell(r, "screenshot")), clip(cell(r, "trace")), clip(cell(r, "record")), strings.Join(actions, " "))
	}
	for _, r := range rows {
		if r.render != "" {
			fmt.Fprintf(stdout, "%s: render check %s\n", r.name, r.render)
		}
	}
	return 0
}

func clip(s string) string {
	if r := []rune(s); len(r) > 20 {
		return string(r[:19]) + "…"
	}
	return s
}

func doStubs(p project.Project, names []string, stdout, stderr io.Writer) int {
	contracts, _, code := loadContracts(p, stderr)
	if code != 0 {
		return code
	}
	targets, err := p.Targets()
	if err != nil {
		fmt.Fprintln(stderr, "omnicui:", err)
		return 2
	}
	ts, _, err := selection(names, targets, contracts)
	if err != nil {
		fmt.Fprintln(stderr, "omnicui:", err)
		return 2
	}
	status := 0
	for _, t := range ts {
		if t.Linked && t.Stubs != "" {
			fmt.Fprintf(stdout, "%s: linked from elsewhere; its stub generator is not run here (it would rewrite that app from this project's contracts)\n", t.Name)
			continue
		}
		if t.Stubs == "" {
			fmt.Fprintf(stdout, "%s: no stub generator\n", t.Name)
			continue
		}
		if err := check.Stubs(t, contracts, nil); err != nil {
			fmt.Fprintf(stderr, "%s: %v\n", t.Name, err)
			status = 1
			continue
		}
		fmt.Fprintf(stdout, "%s: stubs written\n", t.Name)
	}
	return status
}

func doAdd(p project.Project, args []string, stdout, stderr io.Writer) int {
	var name, template string
	for i := 0; i < len(args); i++ {
		switch {
		case args[i] == "--template" && i+1 < len(args):
			template = args[i+1]
			i++
		case strings.HasPrefix(args[i], "--template="):
			template = strings.TrimPrefix(args[i], "--template=")
		case name == "" && !strings.HasPrefix(args[i], "-"):
			name = args[i]
		default:
			fmt.Fprintln(stderr, "usage: omnicui add <name> [--template <folder or git URL>]")
			return 2
		}
	}
	if name == "" {
		fmt.Fprintln(stderr, "usage: omnicui add <name> [--template <folder or git URL>]")
		return 2
	}
	dir, err := p.Add(name, template)
	if err != nil {
		fmt.Fprintln(stderr, "omnicui:", err)
		return 2
	}
	if template != "" {
		fmt.Fprintf(stdout, "created targets/%s from %s\n", name, template)
		return 0
	}
	fmt.Fprintf(stdout, "created targets/%s/target.yaml\n\n", name)
	fmt.Fprintf(stdout, "Next: set `adapter:` to the command that starts this target's adapter. The adapter reads\n")
	fmt.Fprintf(stdout, "JSON-RPC 2.0 requests, one per line, on standard input, answers each with one line on standard\n")
	fmt.Fprintf(stdout, "output, and logs only to standard error. It handles these methods:\n\n")
	for _, c := range project.Constructs {
		fmt.Fprintf(stdout, "  %-38s %s\n", c.Name, c.Does)
	}
	_ = dir
	return 0
}

// doCompare measures two screenshots the way check does, outside a run: for looking into one
// comparison, or for trying thresholds on saved screenshots.
func doCompare(args []string, stdout, stderr io.Writer) int {
	var files []string
	diff, asJSON := "", false
	for i := 0; i < len(args); i++ {
		switch {
		case args[i] == "--diff" && i+1 < len(args):
			diff = args[i+1]
			i++
		case strings.HasPrefix(args[i], "--diff="):
			diff = strings.TrimPrefix(args[i], "--diff=")
		case args[i] == "--json":
			asJSON = true
		case strings.HasPrefix(args[i], "-"):
			fmt.Fprintf(stderr, "omnicui: unknown option %s\n", args[i])
			return 2
		default:
			files = append(files, args[i])
		}
	}
	if len(files) != 2 {
		fmt.Fprintln(stderr, "usage: omnicui compare <a.png> <b.png> [--diff <out.png>] [--json]")
		return 2
	}
	m, note, err := compare.Measure(files[0], files[1], diff)
	if err != nil {
		fmt.Fprintln(stderr, "omnicui:", err)
		return 2
	}
	if asJSON {
		data, _ := json.Marshal(struct {
			compare.Measures
			Note string `json:"note,omitempty"`
		}{m, note})
		fmt.Fprintln(stdout, string(data))
		return 0
	}
	if note != "" {
		fmt.Fprintln(stdout, note)
	}
	th := contract.DefaultPixelThresholds
	fmt.Fprint(stdout, check.MeasuresTable([]check.MeasuresRow{{Label: "a ↔ b", Measures: m, Over: check.Over(th, m)}}, th, "  "))
	return 0
}
