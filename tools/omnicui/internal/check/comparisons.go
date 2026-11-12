package check

import (
	"fmt"
	"math"
	"path/filepath"
	"strings"

	"omnicui/internal/compare"
	"omnicui/internal/contract"
)

// compareAll compares, for every test, each pair of targets' screenshots (per combination) and
// traces, and each trace with the test's named timing, as each pair promised (`same:`).
func (r *runner) compareAll(rep *Report) []Comparison {
	var out []Comparison
	r.results = rep.Results
	plats := map[string]contract.Platform{}
	for _, t := range rep.Targets {
		p := t.Platform
		p.Target, p.Design = t.Name, t.Design
		plats[t.Name] = p
	}
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
				out = append(out, r.compareScreens(c, test, results, plats)...)
			}
			if len(test.Trace) > 0 {
				out = append(out, compareTraces(c, test, results, plats)...)
			}
			if test.Record {
				out = append(out, compareClips(c, test, results, plats)...)
			}
		}
	}
	return out
}

// palettes are every screenshot's palette a target took in this run at a colour scheme: the
// colours its design uses, which a colour of another design's screenshot is looked for in.
func (r *runner) palettes(target, scheme string) []compare.Palette {
	r.paletteMu.Lock()
	defer r.paletteMu.Unlock()
	key := target + "\x00" + scheme
	if ps, ok := r.paletteCache[key]; ok {
		return ps
	}
	var out []compare.Palette
	for _, res := range r.results {
		if res.Target != target {
			continue
		}
		for combo, path := range res.Screenshots {
			if !strings.HasSuffix(combo, "-"+scheme) {
				continue
			}
			if p, err := compare.ReadPalette(filepath.Join(r.runDir, path)); err == nil {
				out = append(out, p)
			}
		}
	}
	if r.paletteCache == nil {
		r.paletteCache = map[string][]compare.Palette{}
	}
	r.paletteCache[key] = out
	return out
}

func traceLabel(spec contract.TraceSpec) string {
	if spec.Property != "" && spec.Property != "position" {
		return spec.Part + " " + spec.Property
	}
	return spec.Part
}

func compareTraces(c *contract.Contract, test contract.Test, results []TestResult, plats map[string]contract.Platform) []Comparison {
	var out []Comparison
	for _, spec := range test.Trace {
		// One comparison set per combination the traces were taken at: the i-th of each target's.
		var combos []int
		curves := map[int]map[string]compare.Curve{}
		keys := map[int]map[string]string{}
		order := map[int][]string{}
		for _, res := range results {
			for _, tr := range res.Traces {
				if tr.Part != spec.Part || tr.Point != spec.Point || (tr.Property != "" && tr.Property != spec.Property) {
					continue
				}
				i := comboIndex(c, plats[res.Target], tr.Combo)
				if curves[i] == nil {
					curves[i], keys[i] = map[string]compare.Curve{}, map[string]string{}
					combos = append(combos, i)
				}
				curves[i][res.Target], keys[i][res.Target] = tr.Curve, tr.Combo
				order[i] = append(order[i], res.Target)
			}
		}
		label := traceLabel(spec)
		for _, ci := range combos {
			cs, names := curves[ci], order[ci]
			comboOf := func(a, b string) string {
				if b == "" || keys[ci][a] == keys[ci][b] {
					return keys[ci][a]
				}
				return keys[ci][a] + " / " + keys[ci][b]
			}
			for i := 0; i < len(names); i++ {
				for j := i + 1; j < len(names); j++ {
					// Only where both promised the part's motion.
					if !pairAspects(c, plats[names[i]], plats[names[j]]).parts[spec.Part].Has(contract.Motion) {
						continue
					}
					combo := comboOf(names[i], names[j])
					d, note := compare.Differ(cs[names[i]], cs[names[j]])
					out = append(out, Comparison{Contract: c.ID, Test: test.Name, Kind: "timing", A: names[i], B: names[j], Part: label, Combo: combo,
						Value: round(d, 1), Threshold: c.TimingThreshold, Warning: d > c.TimingThreshold, Note: note})
					// Where the point went: timing is measured along each one's own movement, so a
					// thumb sliding up against one sliding across has the same timing.
					if deg, ok := compare.Direction(cs[names[i]], cs[names[j]]); ok {
						out = append(out, Comparison{Contract: c.ID, Test: test.Name, Kind: "direction", A: names[i], B: names[j], Part: label, Combo: combo,
							Value: round(deg, 0), Threshold: c.DirectionThreshold, Warning: deg > c.DirectionThreshold,
							Note: fmt.Sprintf("%s went %s, %s %s", names[i], compare.Heading(cs[names[i]]), names[j], compare.Heading(cs[names[j]]))})
					}
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
					out = append(out, Comparison{Contract: c.ID, Test: test.Name, Kind: "timing", A: name, B: "timing", Part: label, Combo: comboOf(name, ""),
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
// changes, which have no position to trace. A clip of a part is compared where both promised
// that part's motion; a clip of the whole component where both promised motion as their base.
func compareClips(c *contract.Contract, test contract.Test, results []TestResult, plats map[string]contract.Platform) []Comparison {
	var out []Comparison
	for i := 0; i < len(results); i++ {
		for j := i + 1; j < len(results); j++ {
			ra, rb := results[i], results[j]
			ps := pairAspects(c, plats[ra.Target], plats[rb.Target])
			motion := ps.base.Has(contract.Motion)
			if test.RecordPart != "" {
				motion = ps.parts[test.RecordPart].Has(contract.Motion)
			}
			if !motion {
				continue
			}
			for _, cp := range comboPairs(c, plats[ra.Target], plats[rb.Target]) {
				a, okA := ra.Clips[cp.a.Key()]
				b, okB := rb.Clips[cp.b.Key()]
				if !okA || !okB {
					continue
				}
				cmp := Comparison{Contract: c.ID, Test: test.Name, Kind: "motion", A: ra.Target, B: rb.Target, Part: "clip", Combo: cp.label(), Threshold: c.MotionThreshold}
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
