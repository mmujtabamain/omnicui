package check

import (
	"bytes"
	"crypto/rand"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"log/slog"
	"os"
	"path/filepath"
	"regexp"
	"sort"
	"strings"
	"sync"
	"time"

	"omnicui/internal/compare"
	"omnicui/internal/contract"
	"omnicui/internal/project"
	"omnicui/internal/rpc"
)

// Options for one check.
type Options struct {
	Project   project.Project
	Contracts []*contract.Contract
	// All is every contract in the project. Stub generators always get all of them, even when
	// only some are checked, because they rewrite generated/ from scratch.
	All      []*contract.Contract
	Problems []contract.Problem
	Targets  []project.Target
	// Require lists "target:capability" pairs that must be advertised.
	Require []string
}

type runner struct {
	opts   Options
	runDir string
	runID  string
	log    *slog.Logger
	wire   *os.File
	wireMu sync.Mutex
}

func newRunID() string {
	b := make([]byte, 6)
	rand.Read(b)
	return time.Now().UTC().Format("20060102T150405Z") + "-" + hex.EncodeToString(b)
}

// Prepare replaces the run folder and opens its log files.
func prepare(p project.Project) (dir string, log *slog.Logger, logFile, wire *os.File, err error) {
	dir = p.RunDir()
	if err = os.RemoveAll(dir); err != nil {
		return
	}
	if err = os.MkdirAll(dir, 0o755); err != nil {
		return
	}
	logFile, err = os.Create(filepath.Join(dir, "omnicui.log"))
	if err != nil {
		return
	}
	wire, err = os.Create(filepath.Join(dir, "protocol.log"))
	if err != nil {
		return
	}
	log = slog.New(slog.NewJSONHandler(logFile, &slog.HandlerOptions{Level: slog.LevelDebug}))
	return
}

// Run performs the whole check and writes results.json and report.html into the run folder.
func Run(opts Options) (*Report, error) {
	dir, log, logFile, wire, err := prepare(opts.Project)
	if err != nil {
		return nil, err
	}
	defer logFile.Close()
	defer wire.Close()
	r := &runner{opts: opts, runDir: dir, runID: newRunID(), wire: wire}
	r.log = log.With("run", r.runID)
	r.log.Info("check started", "contracts", len(opts.Contracts), "targets", len(opts.Targets))
	rep := &Report{RunID: r.runID, Contracts: opts.Contracts, Problems: opts.Problems}

	infos := make([]TargetInfo, len(opts.Targets))
	results := make([][]TestResult, len(opts.Targets))
	var wg sync.WaitGroup
	for i, t := range opts.Targets {
		wg.Add(1)
		go func(i int, t project.Target) {
			defer wg.Done()
			infos[i], results[i] = r.runTarget(t)
		}(i, t)
	}
	wg.Wait()
	rep.Targets = infos
	for _, rs := range results {
		rep.Results = append(rep.Results, rs...)
	}
	rep.Comparisons = r.compareAll(rep)
	r.required(rep)
	rep.Summary = summarize(rep)
	r.log.Info("check finished", "pass", rep.Summary.Pass, "fail", rep.Summary.Fail, "broken", rep.Summary.Broken,
		"notAdvertised", rep.Summary.NotAdvertised, "warnings", rep.Summary.Warnings)
	if err := writeJSON(filepath.Join(dir, "results.json"), rep); err != nil {
		return rep, err
	}
	if err := writeReport(filepath.Join(dir, "report.html"), rep); err != nil {
		return rep, err
	}
	return rep, nil
}

func writeJSON(path string, v any) error {
	data, err := json.MarshalIndent(v, "", "  ")
	if err != nil {
		return err
	}
	return os.WriteFile(path, append(data, '\n'), 0o644)
}

// Stubs runs a target's stub generator with every contract as JSON on its standard input.
func Stubs(t project.Target, contracts []*contract.Contract, log *slog.Logger) error {
	if t.Stubs == "" {
		return nil
	}
	data, err := json.Marshal(contracts)
	if err != nil {
		return err
	}
	cmd := rpc.ShellCommand(t.Stubs)
	cmd.Dir = t.Dir
	cmd.Stdin = bytes.NewReader(data)
	out, err := cmd.CombinedOutput()
	if log != nil {
		log.Info("stubs", "target", t.Name, "ok", err == nil)
	}
	if err != nil {
		return fmt.Errorf("the stub generator `%s` failed: %v\n%s", t.Stubs, err, strings.TrimSpace(string(out)))
	}
	return nil
}

type advert struct {
	Protocol string            `json:"protocol"`
	Can      []string          `json:"can"`
	Cannot   map[string]string `json:"cannot"`
}

func (r *runner) runTarget(t project.Target) (TargetInfo, []TestResult) {
	info := TargetInfo{Name: t.Name}
	log := r.log.With("target", t.Name)
	brokenAll := func(reason string) []TestResult {
		var out []TestResult
		for _, c := range r.opts.Contracts {
			for _, test := range c.Tests {
				out = append(out, TestResult{Contract: c.ID, Test: test.Name, Target: t.Name, Status: Broken, Messages: []string{reason}})
			}
		}
		return out
	}
	all := r.opts.All
	if all == nil {
		all = r.opts.Contracts
	}
	if err := Stubs(t, all, log); err != nil {
		info.Broken = err.Error()
		return info, brokenAll(err.Error())
	}
	stepTimeout, startTimeout := t.Timeouts()
	client, err := rpc.Start(rpc.Options{Target: t.Name, Dir: t.Dir, Command: t.Adapter, Log: r.log, Wire: r.wire, WireMu: &r.wireMu,
		Env: []string{"OMNICUI_RUN_ID=" + r.runID}})
	if err != nil {
		info.Broken = err.Error()
		return info, brokenAll(err.Error())
	}
	defer client.Stop()
	var ad advert
	if err := client.Call("advertise", map[string]any{"protocol": rpc.Protocol}, startTimeout, &ad); err != nil {
		info.Broken = "did not answer advertise: " + err.Error()
		info.Stderr = client.StderrTail()
		return info, brokenAll(info.Broken)
	}
	info.Can, info.Cannot, info.Protocol = ad.Can, ad.Cannot, ad.Protocol
	sort.Strings(info.Can)
	can := map[string]bool{}
	for _, c := range ad.Can {
		can[c] = true
	}
	tr := &targetRun{r: r, t: t, client: client, can: can, cannot: ad.Cannot, step: stepTimeout, log: log}
	var out []TestResult
	for _, c := range r.opts.Contracts {
		for _, test := range c.Tests {
			res := tr.runTest(c, test)
			out = append(out, res)
		}
	}
	if b := client.Broken(); b != nil {
		info.Broken = b.Reason
		info.Stderr = b.StderrTail
	}
	return info, out
}

type targetRun struct {
	r      *runner
	t      project.Target
	client *rpc.Client
	can    map[string]bool
	cannot map[string]string
	step   time.Duration
	log    *slog.Logger
}

func (tr *targetRun) reason(capability string) string {
	if why, ok := tr.cannot[capability]; ok && why != "" {
		return why
	}
	return "not advertised"
}

var slugRE = regexp.MustCompile(`[^a-z0-9]+`)

func slug(s string) string {
	return strings.Trim(slugRE.ReplaceAllString(strings.ToLower(s), "-"), "-")
}

// stepParams turns contract steps into the requests a target receives, expanding `times`.
func stepParams(c *contract.Contract, steps []contract.Step) []map[string]any {
	var out []map[string]any
	for _, s := range steps {
		for i := 0; i < s.Times; i++ {
			p := map[string]any{"method": s.Action, "part": c.ID + "." + s.Part}
			switch s.Action {
			case "key":
				p["key"] = s.Key
			case "type":
				p["text"] = s.Text
			case "drag":
				p["dx"], p["dy"] = s.DX, s.DY
			}
			out = append(out, p)
		}
	}
	return out
}

func withoutMethod(p map[string]any) map[string]any {
	q := map[string]any{}
	for k, v := range p {
		if k != "method" {
			q[k] = v
		}
	}
	return q
}

type failure struct{ msg string }

func (f *failure) Error() string { return f.msg }

// call wraps a request: an error reply becomes a test failure, a broken adapter stays broken.
func (tr *targetRun) call(method string, params map[string]any, result any) error {
	err := tr.client.Call(method, params, tr.step, result)
	var re *rpc.RemoteError
	if errors.As(err, &re) {
		what := method
		if p, ok := params["part"].(string); ok && method != "mount" {
			what += " " + p
		}
		return &failure{msg: fmt.Sprintf("%s: %s", what, re.Message)}
	}
	return err
}

func (tr *targetRun) runTest(c *contract.Contract, test contract.Test) TestResult {
	res := TestResult{Contract: c.ID, Test: test.Name, Target: tr.t.Name, Status: Pass, Screenshots: map[string]string{}, Missing: map[string]string{}}
	log := tr.log.With("contract", c.ID, "test", test.Name)
	if b := tr.client.Broken(); b != nil {
		res.Status, res.Messages = Broken, []string{"not run: the target broke earlier (" + b.Reason + ")"}
		return res
	}
	// The constructs the behaviour needs; without them the test can't run here.
	needs := []string{"mount"}
	for _, s := range test.Steps {
		needs = append(needs, s.Action)
	}
	for _, e := range test.Expect {
		if e.Event != "" {
			needs = append(needs, "events")
		} else {
			needs = append(needs, "read")
		}
	}
	for _, n := range needs {
		if !tr.can[n] {
			res.Status = NotAdvertised
			res.Missing[n] = tr.reason(n)
			res.Messages = append(res.Messages, fmt.Sprintf("needs `%s`, which %s doesn't advertise (%s)", n, tr.t.Name, tr.reason(n)))
			return res
		}
	}
	dir := filepath.Join(tr.r.runDir, tr.t.Name, c.Short(), slug(test.Name))
	os.MkdirAll(dir, 0o755)
	relRun := func(p string) string { r, _ := filepath.Rel(tr.r.runDir, p); return r }
	combos := Combos(c)
	steps := stepParams(c, test.Steps)
	label := func(cb Combo) string {
		if len(combos) > 1 {
			return " (" + cb.Label() + ")"
		}
		return ""
	}
	mount := func(cb Combo) error {
		return tr.call("mount", map[string]any{"component": c.ID, "props": nonNil(test.Props),
			"viewport": map[string]int{"width": cb.Viewport.Width, "height": cb.Viewport.Height}, "colorScheme": cb.Scheme}, nil)
	}
	fail := func(msg string) {
		if res.Status != Broken {
			res.Status = Fail
		}
		res.Messages = append(res.Messages, msg)
	}
	handle := func(err error, suffix string) (stop bool) {
		var f *failure
		var b *rpc.BrokenError
		switch {
		case err == nil:
			return false
		case errors.As(err, &f):
			fail(f.msg + suffix)
			return true
		case errors.As(err, &b):
			res.Status = Broken
			res.Messages = append(res.Messages, b.Reason)
			return true
		default:
			fail(err.Error() + suffix)
			return true
		}
	}
	for ci, cb := range combos {
		log := log.With("combo", cb.Key())
		tr.client.SetLog(log)
		if err := mount(cb); err != nil {
			// A missing component is missing at every combination: say it once and stop.
			if f, ok := err.(*failure); ok && strings.Contains(f.msg, "no component") {
				fail(f.msg)
				break
			}
			if handle(err, label(cb)); res.Status == Broken {
				break
			}
			continue
		}
		traced := false
		if ci == 0 && len(test.Trace) > 0 {
			if tr.can["trace"] {
				spec := test.Trace[0]
				var samples []compare.Sample
				err := tr.call("trace", map[string]any{"part": c.ID + "." + spec.Part, "point": spec.Point, "steps": steps}, &samples)
				if handle(err, label(cb)) {
					if res.Status == Broken {
						break
					}
					continue
				}
				res.Traces = append(res.Traces, TraceResult{Part: spec.Part, Point: spec.Point, Samples: samples, Curve: compare.Analyze(samples)})
				traced = true
			} else {
				res.Missing["trace"] = tr.reason("trace")
			}
		}
		if !traced {
			stopped := false
			for _, s := range steps {
				if handle(tr.call(s["method"].(string), withoutMethod(s), nil), label(cb)) {
					stopped = true
					break
				}
			}
			if stopped {
				if res.Status == Broken {
					break
				}
				continue
			}
		}
		if err := tr.expect(c, test, label(cb), fail); handle(err, label(cb)) {
			if res.Status == Broken {
				break
			}
			continue
		}
		if test.Screenshot {
			if tr.can["screenshot"] {
				path := filepath.Join(dir, cb.Key()+".png")
				if handle(tr.call("screenshot", map[string]any{"path": path}, nil), label(cb)) {
					if res.Status == Broken {
						break
					}
				} else if _, err := os.Stat(path); err != nil {
					fail("screenshot: the target replied, but wrote no file" + label(cb))
				} else {
					res.Screenshots[cb.Key()] = relRun(path)
				}
			} else {
				res.Missing["screenshot"] = tr.reason("screenshot")
			}
		}
		if ci == 0 {
			// The other traced points, each from a fresh component.
			for _, spec := range test.Trace[min(1, len(test.Trace)):] {
				if !tr.can["trace"] {
					break
				}
				if handle(mount(cb), label(cb)) {
					break
				}
				var samples []compare.Sample
				if handle(tr.call("trace", map[string]any{"part": c.ID + "." + spec.Part, "point": spec.Point, "steps": steps}, &samples), label(cb)) {
					break
				}
				res.Traces = append(res.Traces, TraceResult{Part: spec.Part, Point: spec.Point, Samples: samples, Curve: compare.Analyze(samples)})
			}
			if test.Record && res.Status != Broken {
				if tr.can["record"] {
					framesDir := filepath.Join(dir, "frames")
					os.MkdirAll(framesDir, 0o755)
					var frames []compare.Frame
					if !handle(mount(cb), label(cb)) && !handle(tr.call("record", map[string]any{"path": framesDir, "steps": steps}, &frames), label(cb)) {
						clip := filepath.Join(dir, "clip.gif")
						if len(frames) == 0 {
							log.Warn("record returned no frames")
						} else if err := compare.Clip(frames, clip); err != nil {
							log.Warn("could not make the clip", "error", err.Error())
						} else {
							res.Clip = relRun(clip)
						}
					}
				} else {
					res.Missing["record"] = tr.reason("record")
				}
			}
		}
		if res.Status == Broken {
			break
		}
	}
	res.Messages = collapse(res.Messages, combos)
	if res.Status == Pass && len(res.Missing) > 0 {
		for _, k := range sortedKeys(res.Missing) {
			res.Messages = append(res.Messages, fmt.Sprintf("%s: not advertised by %s (%s)", k, tr.t.Name, res.Missing[k]))
		}
	}
	log.Info("test finished", "status", string(res.Status))
	return res
}

// collapse turns a message repeated for every combination into one message without the combination.
func collapse(msgs []string, combos []Combo) []string {
	if len(combos) < 2 {
		return msgs
	}
	count := map[string]int{}
	for _, m := range msgs {
		for _, cb := range combos {
			if base, ok := strings.CutSuffix(m, " ("+cb.Label()+")"); ok {
				count[base]++
			}
		}
	}
	var out []string
	done := map[string]bool{}
	for _, m := range msgs {
		base := m
		for _, cb := range combos {
			if b, ok := strings.CutSuffix(m, " ("+cb.Label()+")"); ok {
				base = b
			}
		}
		if count[base] == len(combos) {
			if !done[base] {
				done[base] = true
				out = append(out, base)
			}
			continue
		}
		out = append(out, m)
	}
	return out
}

func nonNil(m map[string]any) map[string]any {
	if m == nil {
		return map[string]any{}
	}
	return m
}

type event struct {
	Name   string         `json:"name"`
	Fields map[string]any `json:"fields"`
}

// expect reads every expected field and the events, and reports each mismatch through fail.
func (tr *targetRun) expect(c *contract.Contract, test contract.Test, suffix string, fail func(string)) error {
	var events []event
	gotEvents := false
	for _, e := range test.Expect {
		if e.Event != "" {
			if !gotEvents {
				if err := tr.call("events", map[string]any{}, &events); err != nil {
					return err
				}
				gotEvents = true
			}
			var matching []event
			for _, ev := range events {
				if ev.Name == e.Event {
					matching = append(matching, ev)
				}
			}
			if e.Times != nil && len(matching) != *e.Times {
				fail(fmt.Sprintf("%s: expected %s, got %s%s", e.Event, timesWord(*e.Times), timesWord(len(matching)), suffix))
			}
			if len(e.Last) > 0 {
				if len(matching) == 0 {
					fail(fmt.Sprintf("%s.last: expected an event, got none%s", e.Event, suffix))
					continue
				}
				last := matching[len(matching)-1]
				for _, k := range sortedKeys(e.Last) {
					if !same(e.Last[k], last.Fields[k]) {
						fail(fmt.Sprintf("%s.last.%s: expected %s, got %s%s", e.Event, k, show(e.Last[k]), show(last.Fields[k]), suffix))
					}
				}
			}
			continue
		}
		part := c.ID + "." + e.Part
		for _, ch := range e.Checks {
			var got any
			if err := tr.call("read", map[string]any{"part": part, "field": ch.Field}, &got); err != nil {
				return err
			}
			if !same(ch.Want, got) {
				fail(fmt.Sprintf("%s.%s: expected %s, got %s%s", e.Part, ch.Field, show(ch.Want), show(got), suffix))
			}
		}
	}
	return nil
}

func timesWord(n int) string {
	if n == 1 {
		return "1 time"
	}
	return fmt.Sprintf("%d times", n)
}

// same compares a contract value with a value read from a target; numbers compare as numbers.
func same(want, got any) bool {
	switch w := want.(type) {
	case int64:
		g, ok := got.(float64)
		return ok && g == float64(w)
	case float64:
		g, ok := got.(float64)
		return ok && g == w
	case string:
		g, ok := got.(string)
		return ok && g == w
	case bool:
		g, ok := got.(bool)
		return ok && g == w
	}
	return false
}

func show(v any) string {
	switch x := v.(type) {
	case nil:
		return "nothing"
	case string:
		return fmt.Sprintf("%q", x)
	case float64:
		if x == float64(int64(x)) {
			return fmt.Sprintf("%d", int64(x))
		}
	}
	return fmt.Sprint(v)
}

func sortedKeys[V any](m map[string]V) []string {
	keys := make([]string, 0, len(m))
	for k := range m {
		keys = append(keys, k)
	}
	sort.Strings(keys)
	return keys
}

// required turns --require target:capability into failures when not advertised.
func (r *runner) required(rep *Report) {
	for _, req := range r.opts.Require {
		name, capability, ok := strings.Cut(req, ":")
		if !ok {
			rep.Required = append(rep.Required, fmt.Sprintf("%s: expected target:capability", req))
			continue
		}
		found := false
		for _, t := range rep.Targets {
			if t.Name != name {
				continue
			}
			found = true
			has := false
			for _, c := range t.Can {
				has = has || c == capability
			}
			if !has {
				why := t.Cannot[capability]
				if why == "" {
					why = "not advertised"
				}
				rep.Required = append(rep.Required, fmt.Sprintf("%s must advertise `%s` (--require), but it doesn't: %s", name, capability, why))
			}
		}
		if !found {
			rep.Required = append(rep.Required, fmt.Sprintf("--require %s: there is no target `%s` in this run", req, name))
		}
	}
}

func summarize(rep *Report) Summary {
	var s Summary
	for _, res := range rep.Results {
		switch res.Status {
		case Pass:
			s.Pass++
		case Fail:
			s.Fail++
		case Broken:
			s.Broken++
		case NotAdvertised:
			s.NotAdvertised++
		}
		if res.Status == Pass {
			s.NotAdvertised += len(res.Missing)
		}
	}
	for _, c := range rep.Comparisons {
		if c.Warning {
			s.Warnings++
		}
	}
	s.Required = len(rep.Required)
	return s
}
