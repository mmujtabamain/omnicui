package check

import (
	"fmt"
	"math"
	"os"
	"path/filepath"
	"strings"

	"omnicui/internal/compare"
	"omnicui/internal/contract"
)

// compareAll compares, for every test, each pair of targets' screenshots (per combination) and
// traces, and each trace with the test's named timing.
func (r *runner) compareAll(rep *Report) []Comparison {
	var out []Comparison
	byKey := map[string][]TestResult{}
	for _, res := range rep.Results {
		k := res.Contract + "\x00" + res.Test
		byKey[k] = append(byKey[k], res)
	}
	for _, c := range rep.Contracts {
		for _, test := range c.Tests {
			results := byKey[c.ID+"\x00"+test.Name]
			for _, res := range results {
				for _, note := range res.Invisible {
					out = append(out, Comparison{Contract: c.ID, Test: test.Name, Kind: "visibility", A: res.Target, Warning: true, Note: note})
				}
			}
			if test.Screenshot {
				out = append(out, r.comparePixels(c, test, results)...)
			}
			if len(test.Trace) > 0 {
				out = append(out, compareTraces(c, test, results)...)
			}
		}
	}
	return out
}

func (r *runner) comparePixels(c *contract.Contract, test contract.Test, results []TestResult) []Comparison {
	var out []Comparison
	diffDir := filepath.Join(r.runDir, "diff", c.Short(), slug(test.Name))
	for i := 0; i < len(results); i++ {
		for j := i + 1; j < len(results); j++ {
			a, b := results[i], results[j]
			var worst *Comparison
			for _, cb := range Combos(c) {
				pa, okA := a.Screenshots[cb.Key()]
				pb, okB := b.Screenshots[cb.Key()]
				if !okA || !okB {
					continue
				}
				os.MkdirAll(diffDir, 0o755)
				diff := filepath.Join(diffDir, fmt.Sprintf("%s-%s-%s.png", a.Target, b.Target, cb.Key()))
				m, note, err := compare.Measure(filepath.Join(r.runDir, pa), filepath.Join(r.runDir, pb), diff)
				if err != nil {
					note = err.Error()
					m = compare.Measures{Area: 100, Strength: 100, Unmatched: 100, Spot: 100}
				}
				m.Area, m.Strength, m.Unmatched, m.Spot = round(m.Area, 2), round(m.Strength, 1), round(m.Unmatched, 1), round(m.Spot, 2)
				relDiff, _ := filepath.Rel(r.runDir, diff)
				th := c.Pixels
				over := Over(th, m)
				cmp := Comparison{Contract: c.ID, Test: test.Name, Kind: "pixels", A: a.Target, B: b.Target, Combo: cb.Key(),
					Value: m.Area, Threshold: th.Area, Warning: len(over) > 0, Diff: relDiff, Note: note,
					Measures: &m, Thresholds: &th, Over: over}
				if len(m.Spots) > 0 {
					cmp.SpotPart = partAt(m.Spots[0], a.Bounds[cb.Key()], b.Bounds[cb.Key()])
				}
				r.log.Info("pixels", "contract", c.ID, "test", test.Name, "a", a.Target, "b", b.Target, "combo", cb.Key(),
					"area", m.Area, "strength", m.Strength, "shift", m.Shift, "spot", m.Spot, "over", strings.Join(over, ","))
				// The worst combination: the most measures over their thresholds, then the largest area.
				if worst == nil || len(cmp.Over) > len(worst.Over) || (len(cmp.Over) == len(worst.Over) && cmp.Value > worst.Value) {
					cp := cmp
					worst = &cp
				}
			}
			if worst != nil {
				out = append(out, *worst)
			}
		}
	}
	return out
}

func compareTraces(c *contract.Contract, test contract.Test, results []TestResult) []Comparison {
	var out []Comparison
	for _, spec := range test.Trace {
		curves := map[string]compare.Curve{}
		var order []string
		for _, res := range results {
			for _, tr := range res.Traces {
				if tr.Part == spec.Part && tr.Point == spec.Point {
					curves[res.Target] = tr.Curve
					order = append(order, res.Target)
				}
			}
		}
		for i := 0; i < len(order); i++ {
			for j := i + 1; j < len(order); j++ {
				d, note := compare.Differ(curves[order[i]], curves[order[j]])
				out = append(out, Comparison{Contract: c.ID, Test: test.Name, Kind: "timing", A: order[i], B: order[j], Part: spec.Part,
					Value: round(d, 1), Threshold: c.TimingThreshold, Warning: d > c.TimingThreshold, Note: note})
			}
		}
		if spec.Timing != nil {
			want, err := compare.Expected(spec.Timing.DurationMS, spec.Timing.Easing)
			if err != nil {
				continue
			}
			for _, name := range order {
				d, note := compare.Differ(curves[name], want)
				if !curves[name].Moved {
					note = fmt.Sprintf("%s didn't move", spec.Part)
				}
				out = append(out, Comparison{Contract: c.ID, Test: test.Name, Kind: "timing", A: name, B: "timing", Part: spec.Part,
					Value: round(d, 1), Threshold: c.TimingThreshold, Warning: d > c.TimingThreshold,
					Note: fmt.Sprintf("against %.0fms %s; %s", spec.Timing.DurationMS, spec.Timing.Easing, note)})
			}
		}
	}
	return out
}

func round(v float64, places int) float64 {
	if math.IsInf(v, 0) {
		return 1e9
	}
	p := math.Pow(10, float64(places))
	return math.Round(v*p) / p
}

// partAt names the smallest part, in either target, whose box holds the middle of a spot.
func partAt(spot compare.Box, a, b map[string]Rect) string {
	x, y := float64(spot.X)+float64(spot.W)/2, float64(spot.Y)+float64(spot.H)/2
	best, area := "", math.Inf(1)
	for _, boxes := range []map[string]Rect{a, b} {
		for name, r := range boxes {
			if r.contains(x, y) && r.Width*r.Height < area {
				best, area = name, r.Width*r.Height
			}
		}
	}
	return best
}
