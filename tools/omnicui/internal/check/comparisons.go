package check

import (
	"fmt"
	"math"
	"os"
	"path/filepath"

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
				pct, note, err := compare.Pixels(filepath.Join(r.runDir, pa), filepath.Join(r.runDir, pb), diff)
				if err != nil {
					note, pct = err.Error(), 100
				}
				relDiff, _ := filepath.Rel(r.runDir, diff)
				cmp := Comparison{Contract: c.ID, Test: test.Name, Kind: "pixels", A: a.Target, B: b.Target, Combo: cb.Key(),
					Value: round(pct, 2), Threshold: c.PixelThreshold, Warning: pct > c.PixelThreshold, Diff: relDiff, Note: note}
				r.log.Info("pixels", "contract", c.ID, "test", test.Name, "a", a.Target, "b", b.Target, "combo", cb.Key(), "percent", cmp.Value)
				if worst == nil || cmp.Value > worst.Value {
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
