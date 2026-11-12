package check

import (
	"bytes"
	"crypto/rand"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"log/slog"
	"math"
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
	// Progress, if set, gets one line per finished test, `[web 3/38] counter › each press adds one  pass  0.4s`,
	// and a line every 10 s while a target starts.
	Progress io.Writer
	// Serial runs one target at a time, as if every target were `exclusive: true`.
	Serial bool
}

type runner struct {
	opts       Options
	progressMu sync.Mutex
	runDir     string
	runID      string
	log        *slog.Logger
	wire       *os.File
	wireMu     sync.Mutex
	// For palettes across designs: every result, and each target's palettes per colour scheme.
	results      []TestResult
	paletteMu    sync.Mutex
	paletteCache map[string][]compare.Palette
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
	// Targets run side by side, except an exclusive one, which runs with no other target.
	var wg sync.WaitGroup
	var machine sync.RWMutex
	for i, t := range opts.Targets {
		wg.Add(1)
		go func(i int, t project.Target) {
			defer wg.Done()
			alone := t.Exclusive || opts.Serial
			r.take(&machine, alone, t.Name)
			if alone {
				defer machine.Unlock()
			} else {
				defer machine.RUnlock()
			}
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

// take locks the machine for a target: on its own (alone) or shared with the other targets. It
// says so when the target has to wait.
func (r *runner) take(m *sync.RWMutex, alone bool, name string) {
	ok := false
	if alone {
		ok = m.TryLock()
	} else {
		ok = m.TryRLock()
	}
	if ok {
		return
	}
	if alone {
		r.say("[%s] waiting to run on its own (exclusive)", name)
		m.Lock()
	} else {
		r.say("[%s] waiting for an exclusive target to finish", name)
		m.RLock()
	}
}

// say prints a progress line, if progress is wanted.
func (r *runner) say(format string, args ...any) {
	if r.opts.Progress == nil {
		return
	}
	r.progressMu.Lock()
	defer r.progressMu.Unlock()
	fmt.Fprintf(r.opts.Progress, format+"\n", args...)
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
	// Keys, optional: per role, the keys that activate it on this target's platform, for the
	// tests that `keyboard: true` adds. A role it doesn't name gets the usual keys.
	Keys map[string][]string `json:"keys,omitempty"`
	// Platform, optional: the OS the target is running on, as it found it.
	Platform struct {
		OS      string `json:"os"`
		Version string `json:"version,omitempty"`
	} `json:"platform"`
}

func (r *runner) runTarget(t project.Target) (info TargetInfo, out []TestResult) {
	began := time.Now()
	defer func() { info.Ms = time.Since(began).Milliseconds() }()
	info = TargetInfo{Name: t.Name, Design: t.Design}
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
	if t.Linked && t.Stubs != "" {
		log.Info("stubs skipped: the target is linked from elsewhere")
	} else if err := Stubs(t, all, log); err != nil {
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
	stopWatch := r.watchStart(t.Name, client, startTimeout)
	err = client.Call("advertise", map[string]any{"protocol": rpc.Protocol}, startTimeout, &ad)
	stopWatch()
	info.StartMs = time.Since(began).Milliseconds()
	if err != nil {
		info.Broken = "did not answer advertise: " + err.Error() + tr0hint(err, t, "startTimeout", startTimeout)
		info.Stderr = client.StderrTail()
		return info, brokenAll(info.Broken)
	}
	info.Can, info.Cannot, info.Protocol, info.Keys = ad.Can, ad.Cannot, ad.Protocol, ad.Keys
	sort.Strings(info.Can)
	can := map[string]bool{}
	for _, c := range ad.Can {
		can[c] = true
	}
	if info.Cannot == nil {
		info.Cannot = map[string]string{}
	}
	// What the target runs on: its forms (target.yaml), and its OS as the adapter found it (or as
	// target.yaml says, for an adapter that doesn't report one).
	plat := contract.Platform{Target: t.Name, Design: t.Design, Forms: t.Platform, OS: ad.Platform.OS}
	if plat.OS == "" {
		plat.OS = t.OS
	}
	info.Platform, info.OSVersion = plat, ad.Platform.Version
	tr := &targetRun{r: r, t: t, client: client, can: can, cannot: info.Cannot, step: stepTimeout, log: log, keys: ad.Keys, plat: plat}
	if why := tr.probe(&info); why != "" {
		// Screenshots from this target would be wrong; behaviour can still be checked.
		delete(can, "screenshot")
		info.Cannot["screenshot"] = "the render check failed: " + why
		info.Can = without(info.Can, "screenshot")
	}
	total := 0
	for _, c := range r.opts.Contracts {
		total += len(c.Tests)
	}
	for _, c := range r.opts.Contracts {
		for _, test := range c.Tests {
			started, mark := time.Now(), client.StderrMark()
			res := tr.runTest(c, test)
			res.Ms = time.Since(started).Milliseconds()
			if res.Status == Fail || res.Status == Broken {
				res.Stderr = tailN(client.StderrSince(mark), 20)
			}
			out = append(out, res)
			r.progress(t.Name, len(out), total, c, test, res)
		}
	}
	if b := client.Broken(); b != nil {
		info.Broken = b.Reason
		info.Stderr = b.StderrTail
	}
	return info, out
}

// watchStart prints a line every 10 s while a target starts, with the time left and the last
// thing its adapter wrote to standard error, so that a slow start (a cold build, a simulator
// booting) doesn't look like a hang. The returned function stops it.
func (r *runner) watchStart(name string, client *rpc.Client, limit time.Duration) func() {
	done := make(chan struct{})
	finished := make(chan struct{})
	started := time.Now()
	go func() {
		defer close(finished)
		tick := time.NewTicker(startTick)
		defer tick.Stop()
		said := false
		for {
			select {
			case <-done:
				if said {
					r.say("[%s] started in %s", name, roundSecond(time.Since(started)))
				}
				return
			case <-tick.C:
				said = true
				line := fmt.Sprintf("[%s] starting: %s of startTimeout %s", name, roundSecond(time.Since(started)), limit)
				if l := tailN(client.StderrTail(), 1); len(l) == 1 && strings.TrimSpace(l[0]) != "" {
					line += " · adapter: " + clipLine(strings.TrimSpace(l[0]), 100)
				}
				r.say("%s", line)
			}
		}
	}()
	return func() { close(done); <-finished }
}

// startTick is how often watchStart reports (a variable, so tests can shorten it).
var startTick = 10 * time.Second

func roundSecond(d time.Duration) time.Duration { return d.Round(time.Second) }

func clipLine(s string, n int) string {
	if r := []rune(s); len(r) > n {
		return string(r[:n]) + "…"
	}
	return s
}

func without(xs []string, x string) []string {
	var out []string
	for _, y := range xs {
		if y != x {
			out = append(out, y)
		}
	}
	return out
}

// progress prints one line per finished test, so that a long check shows it is moving.
func (r *runner) progress(target string, n, total int, c *contract.Contract, test contract.Test, res TestResult) {
	if r.opts.Progress == nil {
		return
	}
	r.progressMu.Lock()
	defer r.progressMu.Unlock()
	fmt.Fprintf(r.opts.Progress, "[%s %d/%d] %s › %s  %s  %s\n", target, n, total, c.Short(), test.Name, res.Status, Seconds(res.Ms))
}

// tr0hint adds what to change when a request timed out: the timeout and where it is set.
func tr0hint(err error, t project.Target, setting string, d time.Duration) string {
	var b *rpc.BrokenError
	if !errors.As(err, &b) || !strings.Contains(b.Reason, "timed out") {
		return ""
	}
	return fmt.Sprintf(" (if it needs longer, raise %s, now %s, in targets/%s/target.yaml)", setting, d, t.Name)
}

type targetRun struct {
	// lastRemote is the last call's error as the client returned it, before it became a failure:
	// so that its JSON-RPC code can be looked at.
	lastRemote error
	// keys: the keys that activate each role on this target, when it says (advertise keys).
	keys   map[string][]string
	r      *runner
	t      project.Target
	client *rpc.Client
	can    map[string]bool
	cannot map[string]string
	step   time.Duration
	log    *slog.Logger
	// plat: what the target runs on, for the contracts' platform selectors.
	plat contract.Platform
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

// stepParams turns contract steps into the requests a target receives, expanding `times`, on
// the default platform (named inputs as their default form).
func stepParams(c *contract.Contract, steps []contract.Step) []map[string]any {
	out, _, _ := stepParamsFor(c, steps, contract.Platform{})
	return out
}

// stepParamsFor is stepParams on one platform: a named input becomes that platform's form of it.
// sent says what each input became ("back: key Alt+Left"); none names an input the platform
// doesn't have, which makes the test not applicable there.
func stepParamsFor(c *contract.Contract, steps []contract.Step, plat contract.Platform) (out []map[string]any, sent []string, none string) {
	for _, s := range steps {
		if s.Action == "input" {
			in, sel, ok := contract.Pick(c.Inputs[s.Input], plat)
			switch {
			case !ok:
				return nil, sent, fmt.Sprintf("input `%s` has no form for %s (and no default) in platforms.yaml", s.Input, plat)
			case in.None:
				return nil, sent, fmt.Sprintf("input `%s` is none on %s (platforms.yaml, `%s`)", s.Input, plat.Target, sel)
			}
			note := fmt.Sprintf("input `%s`: %s (platforms.yaml, `%s`)", s.Input, in, sel)
			if !contains(sent, note) {
				sent = append(sent, note)
			}
			s = contract.Step{Action: in.Action, Part: s.Part, Times: s.Times}
			switch {
			case in.Key != "":
				s.Action, s.Key = "key", in.Key
			case in.Do != "":
				s.Action, s.Do, s.With = "do", in.Do, in.With
			}
		}
		for i := 0; i < s.Times; i++ {
			p := map[string]any{"method": s.Action, "part": c.ID + "." + s.Part}
			if s.Part == "" {
				delete(p, "part") // a key with no part goes to what has focus
			}
			switch s.Action {
			case "do":
				p = map[string]any{"method": "do", "name": s.Do, "with": s.With}
				if s.With == nil {
					p["with"] = map[string]any{}
				}
				if s.Part != "" {
					p["part"] = c.ID + "." + s.Part
				}
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
	return out, sent, ""
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
	tr.lastRemote = err
	var re *rpc.RemoteError
	if errors.As(err, &re) {
		what := method
		if p, ok := params["part"].(string); ok && method != "mount" {
			what += " " + p
		}
		msg := fmt.Sprintf("%s: %s", what, re.Message)
		// A part this target's design doesn't have at all (a trace says more itself).
		if p, ok := params["part"].(string); ok && re.Code == -32002 && method != "trace" && method != "bounds" {
			msg += fmt.Sprintf(" (if %s's design has no `%s`, mark the part `optional: true` in the contract)", tr.t.Name, p[strings.LastIndex(p, ".")+1:])
		}
		return &failure{msg: msg}
	}
	return err
}

// noPart says whether the last reply was "no part" (-32002).
func (tr *targetRun) noPart() bool {
	var re *rpc.RemoteError
	return errors.As(tr.lastRemote, &re) && re.Code == -32002
}

// optionalPart says whether the contract marks a part optional: some designs have it, some don't.
func optionalPart(c *contract.Contract, part string) bool {
	p := c.PartByName(part)
	return p != nil && p.Optional
}

// absent notes, once, what a test skips because this target's design has no such part.
func (tr *targetRun) absent(part, what string, res *TestResult) {
	note := fmt.Sprintf("no `%s` in %s's design (an optional part): %s", part, tr.t.Name, what)
	if !contains(res.Absent, note) {
		res.Absent = append(res.Absent, note)
	}
}

// missingOptional: the last request failed because the target has no such part, and the
// contract says that is allowed. It notes what is skipped.
func (tr *targetRun) missingOptional(c *contract.Contract, part, what string, res *TestResult) bool {
	if !optionalPart(c, part) || !tr.noPart() {
		return false
	}
	tr.absent(part, what, res)
	return true
}

// partName is a step's part, as the contract names it ("" when it has none).
func partName(c *contract.Contract, step map[string]any) string {
	p, _ := step["part"].(string)
	return strings.TrimPrefix(p, c.ID+".")
}

func (tr *targetRun) runTest(c *contract.Contract, test contract.Test) TestResult {
	res := TestResult{Contract: c.ID, Test: test.Name, Target: tr.t.Name, Status: Pass, Screenshots: map[string]string{}, Missing: map[string]string{}}
	log := tr.log.With("contract", c.ID, "test", test.Name)
	tr.client.SetLog(log) // so every call, generated tests' too, is logged under its own test
	if b := tr.client.Broken(); b != nil {
		res.Status, res.Messages = Broken, []string{"not run: the target broke earlier (" + b.Reason + ")"}
		return res
	}
	if test.Keyboard != nil {
		return tr.runKeyboard(c, test, res)
	}
	// Not for this platform: only / except, or an input it doesn't have.
	if !test.Applies(tr.plat) {
		res.Status = NotApplicable
		if len(test.Only) > 0 {
			res.Messages = append(res.Messages, fmt.Sprintf("only: [%s], and %s isn't", strings.Join(test.Only, ", "), tr.plat))
		} else {
			res.Messages = append(res.Messages, fmt.Sprintf("except: [%s], and %s is", strings.Join(test.Except, ", "), tr.plat))
		}
		return res
	}
	steps, sent, none := stepParamsFor(c, test.Steps, tr.plat)
	res.Inputs = sent
	if none != "" {
		res.Status = NotApplicable
		res.Messages = append(res.Messages, none)
		return res
	}
	// The constructs the behaviour needs; without them the test can't run here.
	needs := []string{"mount"}
	for _, s := range steps {
		if s["method"] == "do" {
			needs = append(needs, "do."+s["name"].(string))
			continue
		}
		needs = append(needs, s["method"].(string))
	}
	for _, e := range test.Expect {
		if e.Event != "" {
			needs = append(needs, "events")
			continue
		}
		needs = append(needs, "read")
		for _, ch := range e.Checks {
			switch ch.Field {
			case "role", "focused", "selected":
				needs = append(needs, ch.Field)
			case "visible":
				needs = append(needs, "bounds")
			}
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
	combos := CombosFor(c, tr.plat)
	label := func(cb Combo) string {
		if len(combos) > 1 {
			return " (" + cb.Label() + ")"
		}
		return ""
	}
	mount := func(cb Combo) error {
		return tr.call("mount", map[string]any{"component": c.ID, "props": mergeProps(cb.Viewport.Props, test.Props),
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
			res.Messages = append(res.Messages, b.Reason+tr0hint(err, tr.t, "stepTimeout", tr.step))
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
		// Traces and the clip: at the first combination, or at every one if the test asks.
		motionHere := ci == 0 || test.AllCombinations
		traced := false
		if motionHere && len(test.Trace) > 0 {
			spec := test.Trace[0]
			if capability := traceCapability(spec); tr.can[capability] {
				var samples []compare.Sample
				err := tr.trace(c, spec, steps, &samples)
				if err != nil && tr.missingOptional(c, spec.Part, "its trace is skipped", &res) {
					// The steps ran during the trace: run them again, plainly, on a fresh component.
					if handle(mount(cb), label(cb)) {
						if res.Status == Broken {
							break
						}
						continue
					}
				} else if handle(err, label(cb)) {
					if res.Status == Broken {
						break
					}
					continue
				} else {
					res.Traces = append(res.Traces, TraceResult{Part: spec.Part, Point: spec.Point, Property: spec.Property, Combo: cb.Key(), Samples: samples, Curve: compare.Analyze(samples)})
					res.measured++
					traced = true
				}
			} else {
				res.Missing[capability] = tr.reason(capability)
			}
		}
		if !traced {
			stopped, unmeasured := false, false
			for _, s := range steps {
				err := tr.call(s["method"].(string), withoutMethod(s), nil)
				if err != nil && tr.missingOptional(c, partName(c, s), "the test acts on it, so there is nothing to measure here", &res) {
					unmeasured = true
					break
				}
				if handle(err, label(cb)) {
					stopped = true
					break
				}
			}
			if unmeasured {
				res.Status = NotMeasured
				break
			}
			if stopped {
				if res.Status == Broken {
					break
				}
				continue
			}
		}
		if err := tr.expect(c, test, cb, label(cb), fail, &res); handle(err, label(cb)) {
			if res.Status == Broken {
				break
			}
			continue
		}
		if test.Screenshot {
			if tr.can["screenshot"] {
				path := filepath.Join(dir, cb.Key()+".png")
				var shot shotReply
				if handle(tr.call("screenshot", map[string]any{"path": path}, &shot), label(cb)) {
					if res.Status == Broken {
						break
					}
				} else if _, err := os.Stat(path); err != nil {
					fail("screenshot: the target replied, but wrote no file" + label(cb))
				} else {
					res.Screenshots[cb.Key()] = relRun(path)
					res.measured++
					// What the platform drew there, not the app (a title bar, a status bar): never compared.
					if len(shot.Regions) > 0 {
						if res.Regions == nil {
							res.Regions = map[string][]SystemRegion{}
						}
						res.Regions[cb.Key()] = shot.Regions
					}
					// Where each part was, so that a difference can be put down to a part.
					if tr.can["bounds"] {
						boxes := map[string]Rect{}
						for _, p := range c.Parts {
							var r *Rect
							if err := tr.call("bounds", map[string]any{"part": p.ID}, &r); err == nil && r != nil {
								boxes[p.Name] = *r
							} else if _, ok := err.(*failure); !ok && err != nil {
								handle(err, label(cb))
								break
							}
						}
						if res.Bounds == nil {
							res.Bounds = map[string]map[string]Rect{}
						}
						res.Bounds[cb.Key()] = boxes
					}
				}
			} else {
				res.Missing["screenshot"] = tr.reason("screenshot")
			}
		}
		if motionHere {
			// The other traced points, each from a fresh component.
			for _, spec := range test.Trace[min(1, len(test.Trace)):] {
				capability := traceCapability(spec)
				if !tr.can[capability] {
					res.Missing[capability] = tr.reason(capability)
					continue
				}
				if handle(mount(cb), label(cb)) {
					break
				}
				var samples []compare.Sample
				if err := tr.trace(c, spec, steps, &samples); err != nil && tr.missingOptional(c, spec.Part, "its trace is skipped", &res) {
					continue
				} else if handle(err, label(cb)) {
					break
				}
				res.Traces = append(res.Traces, TraceResult{Part: spec.Part, Point: spec.Point, Property: spec.Property, Combo: cb.Key(), Samples: samples, Curve: compare.Analyze(samples)})
				res.measured++
			}
			if test.Record && res.Status != Broken {
				if tr.can["record"] {
					tr.record(c, test.RecordPart, cb, dir, steps, mount, handle, label, &res, ci == 0)
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
	// Everything the test checks is on parts this design doesn't have.
	if res.Status == Pass && res.measured == 0 && len(res.Absent) > 0 {
		res.Status = NotMeasured
	}
	if res.Status == Pass && len(res.Missing) > 0 {
		for _, k := range sortedKeys(res.Missing) {
			res.Messages = append(res.Messages, fmt.Sprintf("%s: not advertised by %s (%s)", k, tr.t.Name, res.Missing[k]))
		}
	}
	log.Info("test finished", "status", string(res.Status))
	return res
}

// traceCapability is what a target must advertise to take a trace: `trace` for a position,
// `trace.<property>` for a property.
// trace asks for a trace. A part the target can't find is often one that appears during the
// steps (a sheet sliding in, a hero fading in): the message says the target should look it up
// at each frame.
func (tr *targetRun) trace(c *contract.Contract, spec contract.TraceSpec, steps []map[string]any, samples *[]compare.Sample) error {
	err := tr.call("trace", traceParams(c, spec, steps), samples)
	var re *rpc.RemoteError
	var f *failure
	if errors.As(tr.lastRemote, &re) && re.Code == -32002 && errors.As(err, &f) {
		return &failure{msg: f.msg + fmt.Sprintf(" (if `%s` appears during the steps, %s's trace should look it up at each frame and start its samples when it is there: see the protocol's trace; if %s's design has no `%s`, mark the part `optional: true` in the contract)", spec.Part, tr.t.Name, tr.t.Name, spec.Part)}
	}
	if err == nil && len(*samples) == 0 {
		return &failure{msg: fmt.Sprintf("trace %s: no samples (the part wasn't on screen at any frame)", c.ID+"."+spec.Part)}
	}
	return err
}

func traceCapability(spec contract.TraceSpec) string {
	if spec.Property == "" || spec.Property == "position" {
		return "trace"
	}
	return "trace." + spec.Property
}

func traceParams(c *contract.Contract, spec contract.TraceSpec, steps []map[string]any) map[string]any {
	p := map[string]any{"part": c.ID + "." + spec.Part, "point": spec.Point, "steps": steps}
	if spec.Property != "" && spec.Property != "position" {
		p["property"] = spec.Property
		delete(p, "point")
	}
	return p
}

// mergeProps: the viewport's props, then the test's, which win.
func mergeProps(viewport, test map[string]any) map[string]any {
	out := map[string]any{}
	for k, v := range viewport {
		out[k] = v
	}
	for k, v := range test {
		out[k] = v
	}
	return out
}

// recording is what `record` replies: the frames, and (newer adapters) when each step's input
// was delivered, in the frames' timeline. Older adapters reply with the frames alone.
type recording struct {
	Frames []compare.Frame `json:"frames"`
	Inputs []float64       `json:"inputs"`
	// Held: the target sends a frame only when the picture changes (Chromium's screencast), so a
	// gap between frames is a still picture, not a slow change.
	Held bool `json:"held"`
}

func (r *recording) UnmarshalJSON(b []byte) error {
	if len(b) > 0 && b[0] == '[' {
		return json.Unmarshal(b, &r.Frames)
	}
	type plain recording
	return json.Unmarshal(b, (*plain)(r))
}

// record takes the clip at one combination: the frames, a lossless animated PNG of them, and
// the clip's visual progress curve, which is what clips are compared by.
// clipMargin is how far around a part's box a clip of the part is measured, in pixels: room for
// a shadow, a ripple or a scale beyond the box.
const clipMargin = 8

func (tr *targetRun) record(c *contract.Contract, part string, cb Combo, dir string, steps []map[string]any, mount func(Combo) error,
	handle func(error, string) bool, label func(Combo) string, res *TestResult, first bool) {
	framesDir := filepath.Join(dir, "frames-"+cb.Key())
	os.MkdirAll(framesDir, 0o755)
	var rec recording
	if handle(mount(cb), label(cb)) {
		return
	}
	// record: { part: x }: the clip is measured over that part's box, before and after the steps,
	// and a margin around it.
	var box *compare.Region
	missing := 0 // how many times the part wasn't there
	partBox := func() {
		if part == "" || !tr.can["bounds"] {
			return
		}
		var r *Rect
		if err := tr.call("bounds", map[string]any{"part": c.ID + "." + part}, &r); err != nil || r == nil || r.Width <= 0 || r.Height <= 0 {
			if err == nil || tr.noPart() {
				missing++
			}
			return
		}
		b := compare.Region{X: r.X - clipMargin, Y: r.Y - clipMargin, Width: r.Width + 2*clipMargin, Height: r.Height + 2*clipMargin}
		if box != nil {
			x0, y0 := math.Min(box.X, b.X), math.Min(box.Y, b.Y)
			x1, y1 := math.Max(box.X+box.Width, b.X+b.Width), math.Max(box.Y+box.Height, b.Y+b.Height)
			b = compare.Region{X: x0, Y: y0, Width: x1 - x0, Height: y1 - y0}
		}
		box = &b
	}
	partBox()
	if handle(tr.call("record", map[string]any{"path": framesDir, "steps": steps}, &rec), label(cb)) {
		return
	}
	partBox()
	if missing == 2 && optionalPart(c, part) {
		// Neither before nor after the steps: this design has no such part.
		tr.absent(part, "its clip is skipped", res)
		return
	}
	if len(rec.Frames) == 0 {
		tr.log.Warn("record returned no frames")
		return
	}
	rel := func(p string) string { r, _ := filepath.Rel(tr.r.runDir, p); return r }
	clip := ClipResult{Inputs: rec.Inputs}
	for _, f := range rec.Frames {
		clip.Frames = append(clip.Frames, compare.Frame{MS: f.MS, File: rel(f.File)})
	}
	file := filepath.Join(dir, "clip-"+cb.Key()+".png")
	if err := compare.APNG(rec.Frames, file); err != nil {
		tr.log.Warn("could not make the clip", "error", err.Error())
	} else {
		clip.File = rel(file)
	}
	if part != "" && box == nil {
		clip.Note = fmt.Sprintf("`%s` had no box on screen (or %s doesn't advertise bounds), so the whole frames are measured; ", part, tr.t.Name)
	}
	curve, progress, note, err := compare.VisualProgressIn(rec.Frames, rec.Held, box)
	switch {
	case err != nil:
		clip.Note += err.Error()
	default:
		clip.Note += note
		if curve.Moved {
			clip.Progress = &curve
			clip.Values = progress
		}
	}
	clip.Note = strings.TrimSuffix(clip.Note, "; ")
	if res.Clips == nil {
		res.Clips = map[string]ClipResult{}
	}
	res.Clips[cb.Key()] = clip
	res.measured++
	if first {
		res.Clip = clip.File
	}
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
func (tr *targetRun) expect(c *contract.Contract, test contract.Test, cb Combo, suffix string, fail func(string), res *TestResult) error {
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
			res.measured++
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
		var box *Rect
		gotBox := false
		bounds := func() error {
			if !gotBox {
				gotBox = true
				return tr.call("bounds", map[string]any{"part": part}, &box)
			}
			return nil
		}
		readsContent, skipped := false, false
		for _, ch := range e.Checks {
			var got any
			if ch.Field == "visible" {
				if err := bounds(); err != nil {
					if tr.missingOptional(c, e.Part, "what the test expects of it is skipped", res) {
						skipped = true
						break
					}
					return err
				}
				got = box.Visible(cb.Viewport.Width, cb.Viewport.Height)
			} else if err := tr.call("read", map[string]any{"part": part, "field": ch.Field}, &got); err != nil {
				if tr.missingOptional(c, e.Part, "what the test expects of it is skipped", res) {
					skipped = true
					break
				}
				return err
			}
			res.measured++
			readsContent = readsContent || ch.Field == "text" || ch.Field == "name" || ch.Field == "value" || ch.Field == "checked"
			want := ch.Want
			// A value of this platform's own: a platform exception, with its reason.
			if v, sel, ok := contract.Pick(ch.ByPlatform, tr.plat); ok {
				want = v
				if sel != "default" {
					note := fmt.Sprintf("%s.%s: %s on %s (`%s`), not %s; why: %s", e.Part, ch.Field, show(v), tr.t.Name, sel, show(ch.Want), e.Why)
					if !contains(res.Exceptions, note) {
						res.Exceptions = append(res.Exceptions, note)
					}
				}
			}
			if !same(want, got) {
				fail(fmt.Sprintf("%s.%s: expected %s, got %s%s", e.Part, ch.Field, show(want), show(got), suffix))
			}
		}
		if skipped {
			continue
		}
		// A part whose content passed but that nobody can see: the content check says nothing about
		// what is on screen. Only when the target reports bounds, and the test doesn't say whether
		// the part should be visible.
		if readsContent && tr.can["bounds"] && !e.Has("visible") {
			if err := bounds(); err != nil {
				return err
			}
			if !box.Visible(cb.Viewport.Width, cb.Viewport.Height) {
				what := "no size on screen"
				if box != nil && box.Width > 0 && box.Height > 0 {
					what = "is outside the viewport"
				} else if box != nil {
					what = fmt.Sprintf("no size on screen (%g×%g)", box.Width, box.Height)
				}
				note := fmt.Sprintf("`%s` was read but has %s%s", e.Part, what, suffix)
				if strings.HasPrefix(what, "is ") {
					note = fmt.Sprintf("`%s` was read but %s%s", e.Part, what, suffix)
				}
				if !contains(res.Invisible, note) {
					res.Invisible = append(res.Invisible, note)
				}
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
	case contract.TextMatch:
		g, ok := got.(string)
		return ok && w.Match(g)
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
	case contract.TextMatch:
		return x.String()
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
		case NotMeasured:
			s.NotMeasured++
		case NotApplicable:
			s.NotApplicable++
		}
		if res.Status == Pass {
			s.NotAdvertised += len(res.Missing)
		}
		s.Exceptions += len(res.Exceptions)
	}
	for _, c := range rep.Comparisons {
		if c.Warning {
			s.Warnings++
		}
	}
	s.Required = len(rep.Required)
	return s
}

func contains(xs []string, x string) bool {
	for _, y := range xs {
		if y == x {
			return true
		}
	}
	return false
}

// runKeyboard runs a test that `keyboard: true` added: the key on the part must leave the
// component as a pointer press does, with the same events. It needs no expectation of its own,
// so it catches a toolkit whose control ignores a key the others accept (a JavaFX Button fires on
// Space, but on Enter only when it is the default button).
func (tr *targetRun) runKeyboard(c *contract.Contract, test contract.Test, res TestResult) TestResult {
	k := test.Keyboard
	for _, n := range []string{"mount", "press", "focus", "key", "read", "events"} {
		if !tr.can[n] {
			res.Status = NotAdvertised
			res.Missing[n] = tr.reason(n)
			res.Messages = append(res.Messages, fmt.Sprintf("needs `%s`, which %s doesn't advertise (%s)", n, tr.t.Name, tr.reason(n)))
			return res
		}
	}
	cb := CombosFor(c, tr.plat)[0]
	tr.client.SetLog(tr.log.With("contract", c.ID, "test", test.Name, "combo", cb.Key()))
	part := c.ID + "." + k.Part
	// The target may say which keys activate a role on its platform (Compose ignores Space on a
	// button, for one). A key it leaves out isn't checked there.
	role := ""
	for _, p := range c.Parts {
		if p.Name == k.Part {
			role = p.Role
		}
	}
	if keys, ok := tr.keys[role]; ok && !contains(keys, k.Key) {
		res.Status = NotAdvertised
		why := fmt.Sprintf("%s says %s doesn't activate a %s there (advertise keys: %s: %s)", tr.t.Name, k.Key, role, role, strings.Join(keys, ", "))
		if len(keys) == 0 {
			why = fmt.Sprintf("%s says no key activates a %s there (advertise keys: %s: [])", tr.t.Name, role, role)
		}
		res.Missing["key "+k.Key] = why
		res.Messages = append(res.Messages, why)
		return res
	}
	mount := func() error {
		return tr.call("mount", map[string]any{"component": c.ID, "props": mergeProps(cb.Viewport.Props, nil),
			"viewport": map[string]int{"width": cb.Viewport.Width, "height": cb.Viewport.Height}, "colorScheme": cb.Scheme}, nil)
	}
	// read is what the component looks like to a test now: its events, and every part's checkable
	// state (not names, which don't change on a press).
	read := func() ([]string, error) {
		var out []string
		var events []event
		if err := tr.call("events", map[string]any{}, &events); err != nil {
			return nil, err
		}
		for _, e := range events {
			fields, _ := json.Marshal(e.Fields)
			out = append(out, fmt.Sprintf("event %s %s", e.Name, fields))
		}
		for _, p := range c.Parts {
			for _, f := range []string{"text", "checked", "value", "disabled", "selected"} {
				if _, ok := contract.Roles[p.Role].Fields[f]; !ok || (f == "selected" && !tr.can["selected"]) {
					continue
				}
				var got any
				err := tr.call("read", map[string]any{"part": p.ID, "field": f}, &got)
				var fl *failure
				if errors.As(err, &fl) {
					continue // a part that isn't shown now
				} else if err != nil {
					return nil, err
				}
				out = append(out, fmt.Sprintf("%s.%s %s", p.Name, f, show(got)))
			}
		}
		return out, nil
	}
	notMeasured := func(why string) TestResult {
		res.Status = NotMeasured
		res.Messages = append(res.Messages, why)
		tr.log.Info("test finished", "contract", c.ID, "test", test.Name, "status", string(res.Status))
		return res
	}
	// With the pointer first: the part must be on screen when the component mounts, and the press
	// must change something a test can read, or there is nothing to compare the key with.
	err := mount()
	shown := true
	if err == nil && tr.can["bounds"] {
		var box *Rect
		if e := tr.call("bounds", map[string]any{"part": part}, &box); e == nil {
			shown = box != nil && box.Width > 0 && box.Height > 0
		} else if _, ok := e.(*failure); ok {
			shown = false
		} else {
			err = e
		}
	}
	if err == nil && !shown {
		if optionalPart(c, k.Part) {
			return notMeasured(fmt.Sprintf("no `%s` in %s's design (an optional part), so there is no key to compare with a press", k.Part, tr.t.Name))
		}
		return notMeasured(notShown(c, k.Part))
	}
	var before, pressed []string
	if err == nil {
		before, err = read()
	}
	if err == nil {
		err = tr.call("press", map[string]any{"part": part}, nil)
		var re *rpc.RemoteError
		if errors.As(tr.lastRemote, &re) && re.Code == -32002 {
			return notMeasured(notShown(c, k.Part))
		}
	}
	if err == nil {
		pressed, err = read()
	}
	if err == nil && stateDiff(before, pressed) == "" {
		return notMeasured(fmt.Sprintf("a press on `%s` changed nothing a test can read when %s mounts (no event, and no part's text, checked, value, disabled or selected), so there is nothing to compare %s with: a test that sets it up (props) and expects the change says more", k.Part, c.ID, k.Key))
	}
	if err == nil {
		var keyed []string
		err = mount()
		if err == nil {
			err = tr.call("focus", map[string]any{"part": part}, nil)
		}
		if err == nil {
			err = tr.call("key", map[string]any{"part": part, "key": k.Key}, nil)
		}
		if err == nil {
			keyed, err = read()
		}
		if err == nil {
			if diff := stateDiff(pressed, keyed); diff != "" {
				res.Status = Fail
				res.Messages = append(res.Messages, fmt.Sprintf("%s on `%s` didn't do what a press does: %s", k.Key, k.Part, diff))
			}
		}
	}
	var f *failure
	var b *rpc.BrokenError
	switch {
	case errors.As(err, &b):
		res.Status = Broken
		res.Messages = append(res.Messages, b.Reason)
	case errors.As(err, &f):
		res.Status = Fail
		res.Messages = append(res.Messages, f.msg)
	case err != nil:
		res.Status = Fail
		res.Messages = append(res.Messages, err.Error())
	}
	tr.log.Info("test finished", "contract", c.ID, "test", test.Name, "status", string(res.Status))
	return res
}

// stateDiff says how two states differ: "a press gave …; the key gave …".
func stateDiff(pressed, keyed []string) string {
	count := map[string]int{}
	for _, x := range pressed {
		count[x]++
	}
	for _, x := range keyed {
		count[x]--
	}
	var onlyPress, onlyKey []string
	for _, x := range pressed {
		if count[x] > 0 {
			onlyPress = append(onlyPress, x)
			count[x]--
		}
	}
	for k, n := range count {
		for ; n < 0; n++ {
			onlyKey = append(onlyKey, k)
		}
	}
	sort.Strings(onlyKey)
	if len(onlyPress) == 0 && len(onlyKey) == 0 {
		return ""
	}
	say := func(xs []string) string {
		if len(xs) == 0 {
			return "nothing else"
		}
		return strings.Join(xs, ", ")
	}
	return fmt.Sprintf("a press gave %s; the key gave %s", say(onlyPress), say(onlyKey))
}

// notShown says why a generated keyboard test can't run: its part isn't on screen when the
// component mounts (it is in a sheet, a dialog or a later state), and what to do about it.
func notShown(c *contract.Contract, part string) string {
	return fmt.Sprintf("`%s` isn't on screen when %s mounts, so keyboard: true can't press it there. "+
		"If it is shown by a sheet, a dialog or another state, make that a component of its own (or a prop that "+
		"mounts it shown), and give that component keyboard: true", part, c.ID)
}

// shotReply is what `screenshot` replies: the path, or (newer adapters) the path and the regions
// of the picture its platform drew rather than the app.
type shotReply struct {
	Path    string         `json:"path"`
	Regions []SystemRegion `json:"regions,omitempty"`
}

func (r *shotReply) UnmarshalJSON(b []byte) error {
	if len(b) > 0 && b[0] == '"' {
		return json.Unmarshal(b, &r.Path)
	}
	type plain shotReply
	return json.Unmarshal(b, (*plain)(r))
}
