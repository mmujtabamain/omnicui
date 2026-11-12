package check

import (
	"fmt"
	"math"
	"os"
	"path/filepath"
	"sort"
	"strings"

	"omnicui/internal/compare"
	"omnicui/internal/contract"
)

// What two targets hold the same. A contract says, per part and per platform, what that part
// holds the same between targets (`same:`): pixels, layout, colours, motion or none. Each target
// of a pair promises what its most specific platform selector says, or the base: the contract's
// `same` (default pixels) between targets of one design, colours and motion between designs.
// The pair is held to what both promised, part by part, and nothing more.

// pairSame is what one pair of targets compares: the base, for the whole screen, and per part.
type pairSame struct {
	base  contract.Aspects
	parts map[string]contract.Aspects
	// explicit: some part's `same` applied to this pair, so the comparison says what it compared.
	explicit bool
}

func pairAspects(c *contract.Contract, pa, pb contract.Platform) pairSame {
	base := c.Same
	if pa.Design != pb.Design {
		base = contract.Colours | contract.Motion
	}
	ps := pairSame{base: base, parts: map[string]contract.Aspects{}}
	for _, p := range c.Parts {
		a, okA := c.PartSame(p.Name, pa)
		if !okA {
			a = base
		}
		b, okB := c.PartSame(p.Name, pb)
		if !okB {
			b = base
		}
		ps.parts[p.Name] = a & b
		ps.explicit = ps.explicit || okA || okB
	}
	return ps
}

// note says, for a report, each part compared otherwise than the base: "title: layout + colours".
func (ps pairSame) note(c *contract.Contract) string {
	var out []string
	for _, p := range c.Parts {
		if a := ps.parts[p.Name]; a != ps.base {
			out = append(out, fmt.Sprintf("%s: %s", p.Name, a))
		}
	}
	if len(out) == 0 {
		return ""
	}
	return fmt.Sprintf("compared as %s, except %s", ps.base, strings.Join(out, " · "))
}

// comboPair is one combination of each target to compare: the same when both run at the same
// sizes, and otherwise the i-th viewport of each, at the same colour scheme.
type comboPair struct{ a, b Combo }

func (p comboPair) label() string {
	if p.a.Key() == p.b.Key() {
		return p.a.Key()
	}
	return p.a.Key() + " / " + p.b.Key()
}

func comboPairs(c *contract.Contract, pa, pb contract.Platform) []comboPair {
	ca, cb := CombosFor(c, pa), CombosFor(c, pb)
	var out []comboPair
	for i := 0; i < len(ca) && i < len(cb); i++ {
		if ca[i].Scheme == cb[i].Scheme {
			out = append(out, comboPair{ca[i], cb[i]})
		}
	}
	return out
}

// comboIndex is where a combination key is in a platform's list (-1 if it isn't): traces and
// clips taken at the first combination of each are compared with each other.
func comboIndex(c *contract.Contract, p contract.Platform, key string) int {
	for i, cb := range CombosFor(c, p) {
		if cb.Key() == key {
			return i
		}
	}
	return -1
}

// compareScreens compares each pair of targets' screenshots, part by part, as the pair promised.
func (r *runner) compareScreens(c *contract.Contract, test contract.Test, results []TestResult, plats map[string]contract.Platform) []Comparison {
	var out []Comparison
	for i := 0; i < len(results); i++ {
		for j := i + 1; j < len(results); j++ {
			out = append(out, r.comparePair(c, test, results[i], results[j], plats)...)
		}
	}
	return out
}

func (r *runner) comparePair(c *contract.Contract, test contract.Test, a, b TestResult, plats map[string]contract.Platform) []Comparison {
	pa, pb := plats[a.Target], plats[b.Target]
	ps := pairAspects(c, pa, pb)
	designName := func(p contract.Platform) string {
		if p.Design == "" {
			return "the default design"
		}
		return "design " + p.Design
	}
	designNote := ""
	if pa.Design != pb.Design {
		designNote = fmt.Sprintf("%s is %s and %s %s, so their colours are compared, not their pixels", a.Target, designName(pa), b.Target, designName(pb))
	}
	// The worst combination of each comparison (kind and part), in the order first seen.
	worst := map[string]*Comparison{}
	var order []string
	keep := func(cmp Comparison) {
		k := cmp.Kind + "\x00" + cmp.Part
		w := worst[k]
		better := w == nil || len(cmp.Over) > len(w.Over) || (len(cmp.Over) == len(w.Over) && cmp.Value > w.Value)
		if w == nil {
			order = append(order, k)
		}
		if better {
			cp := cmp
			worst[k] = &cp
		}
	}
	pixels := func(p contract.Aspects) bool { return p.Has(contract.Pixels) }
	for _, cp := range comboPairs(c, pa, pb) {
		ka, kb := cp.a.Key(), cp.b.Key()
		shotA, okA := a.Screenshots[ka]
		shotB, okB := b.Screenshots[kb]
		if !okA || !okB {
			continue
		}
		// The whole screen, at the base level, with every part the base doesn't cover masked;
		// then each such part as the pair promised it.
		var masked, layout []string
		var cropPixels, cropColours []string
		for _, p := range c.Parts {
			want := ps.parts[p.Name]
			covered := (pixels(ps.base) && pixels(want)) || (!pixels(ps.base) && ps.base.Has(contract.Colours) && want.Has(contract.Colours))
			if covered && !(pixels(want) && !pixels(ps.base)) {
				continue
			}
			if ps.base.Has(contract.Pixels) || ps.base.Has(contract.Colours) {
				if !(ps.base.Has(contract.Colours) && !pixels(ps.base) && want.Has(contract.Colours)) {
					masked = append(masked, p.Name)
				}
			}
			switch {
			case pixels(want):
				cropPixels = append(cropPixels, p.Name)
			case want.Has(contract.Colours) && pixels(ps.base):
				cropColours = append(cropColours, p.Name)
			case want.Has(contract.Colours) && !ps.base.Has(contract.Colours):
				cropColours = append(cropColours, p.Name)
			}
			if want.Has(contract.Layout) && !pixels(want) {
				layout = append(layout, p.Name)
			}
		}
		masks := append(append(append([]string{}, c.Mask...), test.Mask...), masked...)
		switch {
		case pixels(ps.base):
			view, viewNote := viewFor(masks, test.ScreenshotPart, a, b, ka, kb)
			cmp := r.pixelComparison(c, test, a, b, shotA, shotB, cp, view, "", viewNote)
			cmp.Note = joinNotes(cmp.Note, ps.note(c))
			keep(cmp)
		case ps.base.Has(contract.Colours):
			view, viewNote := viewFor(masks, test.ScreenshotPart, a, b, ka, kb)
			cmp := r.paletteComparison(c, test, a, b, shotA, shotB, cp, view, "")
			if designNote != "" {
				cmp.Note = designNote + ": " + cmp.Note
			}
			cmp.Note = joinNotes(joinNotes(cmp.Note, viewNote), ps.note(c))
			keep(cmp)
		}
		for _, p := range cropPixels {
			if view, _ := viewFor(nil, p, a, b, ka, kb); view.CropA != nil {
				keep(r.pixelComparison(c, test, a, b, shotA, shotB, cp, view, p, ""))
			}
		}
		for _, p := range cropColours {
			if view, _ := viewFor(nil, p, a, b, ka, kb); view.CropA != nil {
				keep(r.paletteComparison(c, test, a, b, shotA, shotB, cp, view, p))
			}
		}
		if len(layout) > 0 {
			if cmp, ok := layoutComparison(c, test, a, b, cp, layout); ok {
				keep(cmp)
			}
		}
	}
	var out []Comparison
	for _, k := range order {
		out = append(out, *worst[k])
	}
	return out
}

// pixelComparison measures two screenshots (or a part of each) pixel by pixel.
func (r *runner) pixelComparison(c *contract.Contract, test contract.Test, a, b TestResult, shotA, shotB string, cp comboPair, view compare.View, part, viewNote string) Comparison {
	diffDir := filepath.Join(r.runDir, "diff", c.Short(), slug(test.Name))
	os.MkdirAll(diffDir, 0o755)
	name := fmt.Sprintf("%s-%s-%s.png", a.Target, b.Target, strings.ReplaceAll(cp.label(), " / ", "-"))
	if part != "" {
		name = fmt.Sprintf("%s-%s-%s-%s.png", a.Target, b.Target, part, strings.ReplaceAll(cp.label(), " / ", "-"))
	}
	diff := filepath.Join(diffDir, name)
	pa, pb := filepath.Join(r.runDir, shotA), filepath.Join(r.runDir, shotB)
	m, note, err := compare.MeasureView(pa, pb, diff, view)
	if err != nil {
		note = err.Error()
		m = compare.Measures{Area: 100, Strength: 100, Unmatched: 100, Spot: 100}
	}
	m.Area, m.Strength, m.Unmatched, m.Spot = round(m.Area, 2), round(m.Strength, 1), round(m.Unmatched, 1), round(m.Spot, 2)
	var raw *compare.Measures
	if !view.Empty() && err == nil && part == "" {
		// The whole screenshots too, so that what the view left out stays visible.
		if rm, _, rerr := compare.Measure(pa, pb, ""); rerr == nil {
			rm.Area, rm.Strength, rm.Unmatched, rm.Spot = round(rm.Area, 2), round(rm.Strength, 1), round(rm.Unmatched, 1), round(rm.Spot, 2)
			rm.Spots = nil
			raw = &rm
			viewNote += fmt.Sprintf(" (whole screenshots: area %g%%, strength %g%%, shift %g px, spot %g%%)", rm.Area, rm.Strength, rm.Shift, rm.Spot)
		}
	}
	note = joinNotes(note, viewNote)
	if part != "" {
		note = joinNotes(fmt.Sprintf("only `%s`, which this pair holds pixel-identical", part), note)
	}
	relDiff, _ := filepath.Rel(r.runDir, diff)
	th := c.Pixels
	over := Over(th, m)
	cmp := Comparison{Contract: c.ID, Test: test.Name, Kind: "pixels", A: a.Target, B: b.Target, Combo: cp.label(), Part: part,
		Value: m.Area, Threshold: th.Area, Warning: len(over) > 0, Diff: relDiff, Note: note,
		Measures: &m, Thresholds: &th, Over: over, Raw: raw}
	if len(m.Spots) > 0 {
		if view.CropA != nil || view.CropB != nil {
			cmp.SpotPart = partAt(shiftBox(m.Spots[0], view.CropA), a.Bounds[cp.a.Key()], nil)
			if cmp.SpotPart == "" {
				cmp.SpotPart = partAt(shiftBox(m.Spots[0], view.CropB), nil, b.Bounds[cp.b.Key()])
			}
		} else {
			cmp.SpotPart = partAt(m.Spots[0], a.Bounds[cp.a.Key()], b.Bounds[cp.b.Key()])
		}
	}
	r.log.Info("pixels", "contract", c.ID, "test", test.Name, "a", a.Target, "b", b.Target, "combo", cp.label(), "part", part,
		"area", m.Area, "strength", m.Strength, "shift", m.Shift, "spot", m.Spot, "over", strings.Join(over, ","))
	return cmp
}

// paletteComparison compares the colours two screenshots (or a part of each) are drawn with.
func (r *runner) paletteComparison(c *contract.Contract, test contract.Test, a, b TestResult, shotA, shotB string, cp comboPair, view compare.View, part string) Comparison {
	p, err := compare.ComparePalettesIn(filepath.Join(r.runDir, shotA), filepath.Join(r.runDir, shotB), view,
		r.palettes(a.Target, cp.a.Scheme), r.palettes(b.Target, cp.b.Scheme))
	cmp := Comparison{Contract: c.ID, Test: test.Name, Kind: "palette", A: a.Target, B: b.Target, Combo: cp.label(), Part: part,
		Threshold: c.PaletteThreshold}
	if err != nil {
		cmp.Value, cmp.Warning, cmp.Note = 100, true, err.Error()
	} else {
		cmp.Value, cmp.Warning, cmp.Palette = p.Value, p.Value > c.PaletteThreshold, &p
		cmp.Note = p.Note(a.Target, b.Target)
		if part != "" {
			cmp.Note = fmt.Sprintf("only `%s`: %s", part, cmp.Note)
		}
	}
	r.log.Info("palette", "contract", c.ID, "test", test.Name, "a", a.Target, "b", b.Target, "combo", cp.label(), "part", part, "value", cmp.Value)
	return cmp
}

// viewFor is what of two screenshots a comparison looks at: everything but the masked parts and
// the regions each target's platform drew (a title bar, a status bar), or only one part's box.
func viewFor(masks []string, crop string, a, b TestResult, ka, kb string) (compare.View, string) {
	var v compare.View
	var notes []string
	region := func(r Rect) compare.Region { return compare.Region{X: r.X, Y: r.Y, Width: r.Width, Height: r.Height} }
	// The platform's own drawing is never the app's: always left out.
	var sys []string
	for _, x := range []struct {
		res TestResult
		key string
		add func(compare.Region)
	}{{a, ka, func(r compare.Region) { v.MasksA = append(v.MasksA, r) }}, {b, kb, func(r compare.Region) { v.MasksB = append(v.MasksB, r) }}} {
		for _, s := range x.res.Regions[x.key] {
			x.add(compare.Region{X: s.X, Y: s.Y, Width: s.Width, Height: s.Height})
			sys = append(sys, fmt.Sprintf("%s's %s", x.res.Target, s.Kind))
		}
	}
	if len(sys) > 0 {
		notes = append(notes, "left out, drawn by the platform: "+strings.Join(sys, ", "))
	}
	if len(masks) == 0 && crop == "" {
		return v, strings.Join(notes, "; ")
	}
	for _, x := range []TestResult{a, b} {
		key := ka
		if x.Target == b.Target {
			key = kb
		}
		if x.Bounds[key] == nil {
			return v, joinNotes(strings.Join(notes, "; "), fmt.Sprintf("not masked or cropped: %s reports no bounds (advertise bounds)", x.Target))
		}
	}
	shown := func(x TestResult, key, part string) (Rect, bool) {
		r, ok := x.Bounds[key][part]
		return r, ok && r.Width > 0 && r.Height > 0
	}
	if crop != "" {
		ra, okA := shown(a, ka, crop)
		rb, okB := shown(b, kb, crop)
		if !okA || !okB {
			return compare.View{}, fmt.Sprintf("`%s` isn't on screen on both, so the whole screenshots are compared", crop)
		}
		ca, cb := region(ra), region(rb)
		v.CropA, v.CropB = &ca, &cb
		notes = append(notes, fmt.Sprintf("only `%s`", crop))
	}
	var left []string
	for _, m := range masks {
		found := false
		if r, ok := shown(a, ka, m); ok {
			v.MasksA = append(v.MasksA, region(r))
			found = true
		}
		if r, ok := shown(b, kb, m); ok {
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

// pixelView is the view of a test's own masks and screenshot part, at one combination.
func pixelView(c *contract.Contract, test contract.Test, a, b TestResult, combo string) (compare.View, string) {
	masks := append(append([]string{}, c.Mask...), test.Mask...)
	return viewFor(masks, test.ScreenshotPart, a, b, combo, combo)
}

// layoutComparison compares where parts are, relative to the component, on two targets that
// may draw them with their own materials and controls: each part's box is taken relative to the
// box around every part both show, and scaled to its width. The largest difference in position
// or size, in percent of the width, is the measure; parts in another order are 100%.
func layoutComparison(c *contract.Contract, test contract.Test, a, b TestResult, cp comboPair, parts []string) (Comparison, bool) {
	ba, bb := a.Bounds[cp.a.Key()], b.Bounds[cp.b.Key()]
	cmp := Comparison{Contract: c.ID, Test: test.Name, Kind: "layout", A: a.Target, B: b.Target, Combo: cp.label(), Part: "layout",
		Threshold: c.LayoutThreshold}
	if ba == nil || bb == nil {
		return cmp, false
	}
	var both []string
	for _, p := range c.Parts {
		ra, okA := ba[p.Name]
		rb, okB := bb[p.Name]
		if okA && okB && ra.Width > 0 && ra.Height > 0 && rb.Width > 0 && rb.Height > 0 {
			both = append(both, p.Name)
		}
	}
	if len(both) == 0 {
		return cmp, false
	}
	frame := func(boxes map[string]Rect) Rect {
		x0, y0, x1, y1 := math.Inf(1), math.Inf(1), math.Inf(-1), math.Inf(-1)
		for _, p := range both {
			r := boxes[p]
			x0, y0, x1, y1 = math.Min(x0, r.X), math.Min(y0, r.Y), math.Max(x1, r.X+r.Width), math.Max(y1, r.Y+r.Height)
		}
		return Rect{X: x0, Y: y0, Width: x1 - x0, Height: y1 - y0}
	}
	fa, fb := frame(ba), frame(bb)
	if fa.Width <= 0 || fb.Width <= 0 {
		return cmp, false
	}
	norm := func(r, f Rect) [4]float64 {
		return [4]float64{(r.X - f.X) / f.Width * 100, (r.Y - f.Y) / f.Width * 100, r.Width / f.Width * 100, r.Height / f.Width * 100}
	}
	worstPart, worst, what := "", 0.0, ""
	for _, p := range parts {
		if !contains(both, p) {
			continue
		}
		na, nb := norm(ba[p], fa), norm(bb[p], fb)
		for k, name := range []string{"left", "top", "width", "height"} {
			if d := math.Abs(na[k] - nb[k]); d > worst {
				worst, worstPart, what = d, p, name
			}
		}
	}
	// The order parts are read in: top to bottom, then left to right.
	ordered := func(boxes map[string]Rect) []string {
		xs := append([]string{}, parts...)
		xs = filterIn(xs, both)
		sort.SliceStable(xs, func(i, j int) bool {
			ri, rj := boxes[xs[i]], boxes[xs[j]]
			ci, cj := ri.Y+ri.Height/2, rj.Y+rj.Height/2
			if math.Abs(ci-cj) > math.Min(ri.Height, rj.Height)/2 {
				return ci < cj
			}
			return ri.X < rj.X
		})
		return xs
	}
	oa, ob := ordered(ba), ordered(bb)
	cmp.Value = round(worst, 1)
	if strings.Join(oa, ",") != strings.Join(ob, ",") {
		cmp.Value = 100
		cmp.Note = fmt.Sprintf("the parts are in another order: %s on %s, %s on %s", strings.Join(oa, ", "), a.Target, strings.Join(ob, ", "), b.Target)
	} else if worstPart != "" {
		cmp.Note = fmt.Sprintf("largest: `%s`'s %s, %.1f%% of the width apart (layout of %s)", worstPart, what, worst, strings.Join(filterIn(parts, both), ", "))
	}
	cmp.Warning = cmp.Value > c.LayoutThreshold
	return cmp, true
}

func filterIn(xs, in []string) []string {
	var out []string
	for _, x := range xs {
		if contains(in, x) {
			out = append(out, x)
		}
	}
	return out
}
