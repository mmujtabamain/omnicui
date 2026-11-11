// Package check runs every test of every contract on every target and compares the results.
package check

import (
	"fmt"

	"omnicui/internal/compare"
	"omnicui/internal/contract"
)

// Status of one test on one target.
type Status string

const (
	Pass          Status = "pass"
	Fail          Status = "fail"
	Broken        Status = "broken"
	NotAdvertised Status = "not advertised"
	// NotMeasured: the test ran but couldn't measure what it is for, such as a generated keyboard
	// test on a part that isn't on screen when the component mounts. Not a failure; it is listed.
	NotMeasured Status = "not measured"
	Warning     Status = "warning"
)

// rank orders statuses from best to worst, for "the worst over all combinations".
func rank(s Status) int {
	switch s {
	case Pass:
		return 0
	case NotAdvertised, NotMeasured:
		return 1
	case Fail:
		return 2
	case Broken:
		return 3
	}
	return 0
}

// Combo is one viewport and colour scheme a test runs at.
type Combo struct {
	Viewport contract.Viewport `json:"viewport"`
	Scheme   string            `json:"colorScheme"`
}

func (c Combo) Key() string {
	return fmt.Sprintf("%dx%d-%s", c.Viewport.Width, c.Viewport.Height, c.Scheme)
}
func (c Combo) Label() string {
	return fmt.Sprintf("%d×%d %s", c.Viewport.Width, c.Viewport.Height, c.Scheme)
}

// Combos lists a contract's viewports × colour schemes, viewports first.
func Combos(c *contract.Contract) []Combo {
	var out []Combo
	for _, v := range c.Viewports {
		for _, s := range c.ColorSchemes {
			out = append(out, Combo{Viewport: v, Scheme: s})
		}
	}
	return out
}

// TraceResult is one traced point on one target.
type TraceResult struct {
	Part     string           `json:"part"`
	Point    string           `json:"point,omitempty"`
	Property string           `json:"property,omitempty"`
	Combo    string           `json:"combo,omitempty"`
	Samples  []compare.Sample `json:"samples"`
	Curve    compare.Curve    `json:"curve"`
}

// TestResult is one test on one target.
type TestResult struct {
	Contract string   `json:"contract"`
	Test     string   `json:"test"`
	Target   string   `json:"target"`
	Status   Status   `json:"status"`
	Messages []string `json:"messages,omitempty"`
	// Screenshots by combo key, as paths relative to the run folder.
	Screenshots map[string]string `json:"screenshots,omitempty"`
	Traces      []TraceResult     `json:"traces,omitempty"`
	// Clip is the first combination's clip; Clips has every combination's, by combo key.
	Clip  string                `json:"clip,omitempty"`
	Clips map[string]ClipResult `json:"clips,omitempty"`
	// Missing lists capabilities a test needed but the target didn't advertise, with the reason.
	Missing map[string]string `json:"notAdvertised,omitempty"`
	// Bounds are the parts' boxes on screen when each screenshot was taken, by combo key and part
	// name (targets that advertise `bounds`).
	Bounds map[string]map[string]Rect `json:"bounds,omitempty"`
	// Invisible lists the parts whose text, name or value was read while they had no size on
	// screen or were outside the viewport: what a user can't see passed anyway.
	Invisible []string `json:"invisible,omitempty"`
	// Absent says what was skipped because this target's design has no such part (an optional
	// part): "no `thumb` (an optional part): its trace is skipped".
	Absent []string `json:"absent,omitempty"`
	// measured counts what the test actually checked here (expectations, traces, clips,
	// screenshots); a test that skipped everything for absent parts is not measured.
	measured int
	// Ms is how long the test took on this target, mounts and captures included.
	Ms int64 `json:"ms"`
	// Stderr holds what the adapter wrote to standard error while a failing or broken test ran.
	Stderr []string `json:"stderr,omitempty"`
}

// ClipResult is one recorded clip: its frames, the animated PNG made of them, when each input
// was delivered (ms in the frames' timeline, when the target says), and its visual progress.
type ClipResult struct {
	File     string          `json:"file,omitempty"`
	Frames   []compare.Frame `json:"frames"`
	Inputs   []float64       `json:"inputs,omitempty"`
	Progress *compare.Curve  `json:"progress,omitempty"`
	// Values is the progress (0→1) at each frame.
	Values []float64 `json:"values,omitempty"`
	Note   string    `json:"note,omitempty"`
}

// Rect is a part's box on screen, in CSS pixels.
type Rect struct {
	X      float64 `json:"x"`
	Y      float64 `json:"y"`
	Width  float64 `json:"width"`
	Height float64 `json:"height"`
}

// Visible: it has a size, and some of it is inside a w×h viewport.
func (r *Rect) Visible(w, h int) bool {
	return r != nil && r.Width > 0 && r.Height > 0 && r.X < float64(w) && r.Y < float64(h) && r.X+r.Width > 0 && r.Y+r.Height > 0
}

func (r Rect) contains(x, y float64) bool {
	return x >= r.X && y >= r.Y && x < r.X+r.Width && y < r.Y+r.Height
}

// Comparison is one measured difference between two targets, or between a target and a timing.
type Comparison struct {
	Contract string `json:"contract"`
	Test     string `json:"test"`
	// Kind: "pixels" (targets of one design), "palette" (targets of different designs),
	// "timing", "direction", "motion" or "visibility".
	Kind  string `json:"kind"`
	A     string `json:"a"`
	B     string `json:"b"` // a target, or "timing" for the test's named timing
	Combo string `json:"combo,omitempty"`
	Part  string `json:"part,omitempty"`
	// Value is the area for pixels, the share in colours only one uses for palette, the largest
	// difference in ms for timing and motion, the angle in degrees for direction.
	Value     float64 `json:"value"`
	Threshold float64 `json:"threshold"`
	Warning   bool    `json:"warning"`
	// Pixels only: all four measures, their thresholds, and those above their threshold.
	Measures   *compare.Measures         `json:"measures,omitempty"`
	Thresholds *contract.PixelThresholds `json:"thresholds,omitempty"`
	Over       []string                  `json:"over,omitempty"`
	// Raw: the whole screenshots' measures, when a mask or a screenshot part narrowed the comparison.
	Raw *compare.Measures `json:"raw,omitempty"`
	// Palette: for kind palette, each screenshot's colours and those only one uses.
	Palette *compare.PaletteResult `json:"palette,omitempty"`
	// SpotPart is the part the worst spot is in, when the targets report bounds.
	SpotPart string `json:"spotPart,omitempty"`
	Diff     string `json:"diff,omitempty"`
	Note     string `json:"note,omitempty"`
}

// TargetInfo is what a target advertised.
type TargetInfo struct {
	Name string `json:"name"`
	// Design is target.yaml's design: empty for the default one.
	Design   string            `json:"design,omitempty"`
	Can      []string          `json:"can"`
	Cannot   map[string]string `json:"cannot,omitempty"`
	Broken   string            `json:"broken,omitempty"`
	Stderr   []string          `json:"stderr,omitempty"`
	Protocol string            `json:"protocol,omitempty"`
	// Keys: what the target said activates each role, when it said (advertise keys).
	Keys map[string][]string `json:"keys,omitempty"`
	// Probe says how the render check went: "passed", "not advertised", or empty when the
	// target doesn't take screenshots.
	Probe string `json:"probe,omitempty"`
	// Ms is how long the target took in all; StartMs how long it took to answer advertise.
	Ms      int64 `json:"ms"`
	StartMs int64 `json:"startMs"`
}

// Summary counts every status.
type Summary struct {
	Pass          int `json:"pass"`
	Fail          int `json:"fail"`
	Broken        int `json:"broken"`
	NotAdvertised int `json:"notAdvertised"`
	NotMeasured   int `json:"notMeasured"`
	Warnings      int `json:"warnings"`
	Required      int `json:"requiredMissing"`
}

// Report is everything one check found. It is written to results.json.
type Report struct {
	RunID       string               `json:"runId"`
	Contracts   []*contract.Contract `json:"contracts"`
	Targets     []TargetInfo         `json:"targets"`
	Results     []TestResult         `json:"results"`
	Comparisons []Comparison         `json:"comparisons"`
	Problems    []contract.Problem   `json:"problems,omitempty"`
	Required    []string             `json:"requiredMissing,omitempty"`
	Summary     Summary              `json:"summary"`
}

// Failed reports whether the run should exit non-zero.
func (r *Report) Failed() bool {
	return r.Summary.Fail > 0 || r.Summary.Broken > 0 || r.Summary.Required > 0
}

// FailOnKinds are the values --fail-on accepts: every warning, or one kind of comparison.
var FailOnKinds = map[string]bool{"warnings": true, "pixels": true, "palette": true, "timing": true, "direction": true, "motion": true, "visibility": true}

// WarningsFailing counts the warnings that --fail-on turns into a failed run.
func (r *Report) WarningsFailing(failOn []string) (int, []string) {
	if len(failOn) == 0 {
		return 0, nil
	}
	kinds := map[string]bool{}
	for _, f := range failOn {
		kinds[f] = true
	}
	n := 0
	for _, c := range r.Comparisons {
		if c.Warning && (kinds["warnings"] || kinds[c.Kind]) {
			n++
		}
	}
	return n, failOn
}
