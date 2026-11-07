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
			if test.Record {
				out = append(out, compareClips(c, test, results)...)
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
				view, viewNote := pixelView(c, test, a, b, cb.Key())
				m, note, err := compare.MeasureView(filepath.Join(r.runDir, pa), filepath.Join(r.runDir, pb), diff, view)
				if err != nil {
					note = err.Error()
					m = compare.Measures{Area: 100, Strength: 100, Unmatched: 100, Spot: 100}
				}
				m.Area, m.Strength, m.Unmatched, m.Spot = round(m.Area, 2), round(m.Strength, 1), round(m.Unmatched, 1), round(m.Spot, 2)
				var raw *compare.Measures
				if !view.Empty() && err == nil {
					// The whole screenshots too, so that what the view left out stays visible.
					if rm, _, rerr := compare.Measure(filepath.Join(r.runDir, pa), filepath.Join(r.runDir, pb), ""); rerr == nil {
						rm.Area, rm.Strength, rm.Unmatched, rm.Spot = round(rm.Area, 2), round(rm.Strength, 1), round(rm.Unmatched, 1), round(rm.Spot, 2)
						rm.Spots = nil
						raw = &rm
						viewNote += fmt.Sprintf(" (whole screenshots: area %g%%, strength %g%%, shift %g px, spot %g%%)", rm.Area, rm.Strength, rm.Shift, rm.Spot)
					}
				}
				note = joinNotes(note, viewNote)
				relDiff, _ := filepath.Rel(r.runDir, diff)
				th := c.Pixels
				over := Over(th, m)
				cmp := Comparison{Contract: c.ID, Test: test.Name, Kind: "pixels", A: a.Target, B: b.Target, Combo: cb.Key(),
					Value: m.Area, Threshold: th.Area, Warning: len(over) > 0, Diff: relDiff, Note: note,
					Measures: &m, Thresholds: &th, Over: over, Raw: raw}
				if len(m.Spots) > 0 {
					if view.CropA != nil || view.CropB != nil {
						cmp.SpotPart = partAt(shiftBox(m.Spots[0], view.CropA), a.Bounds[cb.Key()], nil)
						if cmp.SpotPart == "" {
							cmp.SpotPart = partAt(shiftBox(m.Spots[0], view.CropB), nil, b.Bounds[cb.Key()])
						}
					} else {
						cmp.SpotPart = partAt(m.Spots[0], a.Bounds[cb.Key()], b.Bounds[cb.Key()])
					}
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

func traceLabel(spec contract.TraceSpec) string {
	if spec.Property != "" && spec.Property != "position" {
		return spec.Part + " " + spec.Property
	}
	return spec.Part
}

func compareTraces(c *contract.Contract, test contract.Test, results []TestResult) []Comparison {
	var out []Comparison
	for _, spec := range test.Trace {
		// One comparison set per combination the traces were taken at.
		var combos []string
		curves := map[string]map[string]compare.Curve{}
		order := map[string][]string{}
		for _, res := range results {
			for _, tr := range res.Traces {
				if tr.Part != spec.Part || tr.Point != spec.Point || (tr.Property != "" && tr.Property != spec.Property) {
					continue
				}
				if curves[tr.Combo] == nil {
					curves[tr.Combo] = map[string]compare.Curve{}
					combos = append(combos, tr.Combo)
				}
				curves[tr.Combo][res.Target] = tr.Curve
				order[tr.Combo] = append(order[tr.Combo], res.Target)
			}
		}
		label := traceLabel(spec)
		for _, combo := range combos {
			cs, names := curves[combo], order[combo]
			for i := 0; i < len(names); i++ {
				for j := i + 1; j < len(names); j++ {
					d, note := compare.Differ(cs[names[i]], cs[names[j]])
					out = append(out, Comparison{Contract: c.ID, Test: test.Name, Kind: "timing", A: names[i], B: names[j], Part: label, Combo: combo,
						Value: round(d, 1), Threshold: c.TimingThreshold, Warning: d > c.TimingThreshold, Note: note})
				}
			}
			if spec.Timing != nil {
				want, err := compare.Expected(spec.Timing.DurationMS, spec.Timing.Easing)
				if err != nil {
					continue
				}
				for _, name := range names {
					d, note := compare.Differ(cs[name], want)
					if !cs[name].Moved {
						note = fmt.Sprintf("%s didn't change", label)
					}
					out = append(out, Comparison{Contract: c.ID, Test: test.Name, Kind: "timing", A: name, B: "timing", Part: label, Combo: combo,
						Value: round(d, 1), Threshold: c.TimingThreshold, Warning: d > c.TimingThreshold,
						Note: fmt.Sprintf("against %.0fms %s; %s", spec.Timing.DurationMS, spec.Timing.Easing, note)})
				}
			}
		}
	}
	return out
}

// compareClips compares each pair of targets' clips, at each combination both recorded, by their
// visual progress curves: how the picture moved from its first state to its last, lined up and
// measured at levels of progress like a trace. This is what compares fades, blurs and colour
// changes, which have no position to trace.
func compareClips(c *contract.Contract, test contract.Test, results []TestResult) []Comparison {
	var out []Comparison
	for _, cb := range Combos(c) {
		key := cb.Key()
		for i := 0; i < len(results); i++ {
			for j := i + 1; j < len(results); j++ {
				a, okA := results[i].Clips[key]
				b, okB := results[j].Clips[key]
				if !okA || !okB {
					continue
				}
				cmp := Comparison{Contract: c.ID, Test: test.Name, Kind: "motion", A: results[i].Target, B: results[j].Target, Part: "clip", Combo: key, Threshold: c.MotionThreshold}
				switch {
				case a.Progress == nil || b.Progress == nil:
					why := a.Note
					if why == "" {
						why = b.Note
					}
					cmp.Note = "not compared: " + why
				default:
					d, note := compare.Differ(*a.Progress, *b.Progress)
					cmp.Value, cmp.Warning, cmp.Note = round(d, 1), d > c.MotionThreshold, strings.Replace(note, "of the movement", "of the change", 1)
				}
				out = append(out, cmp)
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
// pixelView is what of two targets' screenshots a test compares: its contract's and its own
// masks, and its screenshot part, from the boxes each target reported. With no boxes from a
// target (it doesn't advertise bounds), the whole screenshots are compared, and the note says so.
func pixelView(c *contract.Contract, test contract.Test, a, b TestResult, combo string) (compare.View, string) {
	masks := append(append([]string{}, c.Mask...), test.Mask...)
	if len(masks) == 0 && test.ScreenshotPart == "" {
		return compare.View{}, ""
	}
	for _, x := range []TestResult{a, b} {
		if x.Bounds[combo] == nil {
			return compare.View{}, fmt.Sprintf("not masked or cropped: %s reports no bounds (advertise bounds)", x.Target)
		}
	}
	region := func(r Rect) compare.Region { return compare.Region{X: r.X, Y: r.Y, Width: r.Width, Height: r.Height} }
	shown := func(x TestResult, part string) (Rect, bool) {
		r, ok := x.Bounds[combo][part]
		return r, ok && r.Width > 0 && r.Height > 0
	}
	var v compare.View
	var notes []string
	if p := test.ScreenshotPart; p != "" {
		ra, okA := shown(a, p)
		rb, okB := shown(b, p)
		if !okA || !okB {
			missing := a.Target
			if okA {
				missing = b.Target
			}
			return compare.View{}, fmt.Sprintf("`%s` isn't on screen on %s, so the whole screenshots are compared", p, missing)
		}
		ca, cb := region(ra), region(rb)
		v.CropA, v.CropB = &ca, &cb
		notes = append(notes, fmt.Sprintf("only `%s`", p))
	}
	var left []string
	for _, m := range masks {
		found := false
		if r, ok := shown(a, m); ok {
			v.MasksA = append(v.MasksA, region(r))
			found = true
		}
		if r, ok := shown(b, m); ok {
			v.MasksB = append(v.MasksB, region(r))
			found = true
		}
		if found && !contains(left, m) {
			left = append(left, m)
		}
	}
	if len(left) > 0 {
		notes = append(notes, "masked: "+strings.Join(left, ", "))
	}
	return v, strings.Join(notes, "; ")
}

func shiftBox(b compare.Box, by *compare.Region) compare.Box {
	if by != nil {
		b.X += int(by.X)
		b.Y += int(by.Y)
	}
	return b
}

func joinNotes(a, b string) string {
	switch {
	case a == "":
		return b
	case b == "":
		return a
	}
	return a + "; " + b
}

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
