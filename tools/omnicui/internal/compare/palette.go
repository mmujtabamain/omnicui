package compare

import (
	"fmt"
	"image"
	"math"
	"sort"
	"strings"
)

// Palettes: the design language of two screenshots that aren't meant to line up.
//
// Two targets of different designs (a phone layout and a desktop one, a redesign) can't be
// compared pixel by pixel: everything moved. What a design language keeps across them is its
// colours: the background, the surfaces, the accent, the text. ComparePalettes reduces each
// screenshot to the colours it fills areas with and measures how much of each one's drawing is
// in a colour the other doesn't use at all.
//
// A palette is made of solid pixels only, those whose 2×2 block is one colour: anti-aliased
// edges are blends that every toolkit makes differently and that belong to no palette. Whether
// the other screenshot uses a colour, though, is asked of all its pixels: a colour one design
// fills a box with and the other draws as 1 px lines and text is a colour both use. Colours
// within colourTolerance of each other are one colour.

// Swatch is one colour of a palette and its share of the drawing (the solid pixels that aren't
// the background), in percent.
type Swatch struct {
	Hex   string  `json:"hex"`
	Share float64 `json:"share"`
}

// PaletteResult says how two screenshots' palettes differ.
type PaletteResult struct {
	// Value is the larger of the two shares, in percent, of a screenshot's drawing that is in a
	// colour the other screenshot doesn't use (100 when their backgrounds differ).
	Value float64 `json:"value"`
	// BackgroundA and BackgroundB are each screenshot's background (its largest solid colour).
	BackgroundA string `json:"backgroundA"`
	BackgroundB string `json:"backgroundB"`
	// BackgroundsDiffer: the two backgrounds are not one colour.
	BackgroundsDiffer bool `json:"backgroundsDiffer,omitempty"`
	// OnlyA and OnlyB are the colours each uses and the other doesn't, largest first.
	OnlyA []Swatch `json:"onlyA,omitempty"`
	OnlyB []Swatch `json:"onlyB,omitempty"`
	// A and B are each screenshot's palette: the background, then its colours, largest first.
	A []Swatch `json:"a"`
	B []Swatch `json:"b"`
}

// paletteSignificant is the share of the drawing, in percent, a colour must have to be part of
// the palette, and paletteMinimum the share of the whole picture (a few solid blocks inside a
// glyph's stroke are a blend, not a colour of the palette); paletteUse is the share of the
// other picture's pixels, solid or not, at which it counts as using a colour too.
const paletteSignificant, paletteMinimum, paletteUse = 3.0, 0.1, 0.05

type cluster struct {
	r, g, b float64 // the mean colour
	n       int
}

func (c cluster) yiq() yiq { return toYIQColor(c.r, c.g, c.b) }

func (c cluster) hex() string {
	return fmt.Sprintf("#%02x%02x%02x", int(math.Round(c.r)), int(math.Round(c.g)), int(math.Round(c.b)))
}

func toYIQColor(r, g, b float64) yiq {
	return yiq{
		y: r*0.29889531 + g*0.58662247 + b*0.11448223,
		i: r*0.59597799 - g*0.27417610 - b*0.32180189,
		q: r*0.21147017 - g*0.52261711 + b*0.31114694,
	}
}

// Palette is an image's solid colours, largest first (the first is its background), and every
// colour it has at all, solid or not.
type Palette = palette

type palette struct {
	solid []cluster
	all   []cluster
	n     int // pixels looked at
	// more: the same design's other palettes, which count too when asking whether it has a colour.
	more []palette
}

// paletteOf returns an image's palette. Masked pixels (a View's masks) are left out.
func paletteOf(img image.Image, masked []bool) palette {
	b := img.Bounds()
	w, h := b.Dx(), b.Dy()
	type px struct{ r, g, b float64 }
	all := make([]px, w*h)
	ys := make([]yiq, w*h)
	for y := 0; y < h; y++ {
		for x := 0; x < w; x++ {
			r, g, bl := rgb(img.At(b.Min.X+x, b.Min.Y+y))
			all[y*w+x] = px{r, g, bl}
			ys[y*w+x] = toYIQColor(r, g, bl)
		}
	}
	same := maxYIQDelta * 0.03 * 0.03
	// Bins of 16 levels per channel, each with its pixels' mean colour.
	bins, every := map[int]*cluster{}, map[int]*cluster{}
	add := func(bins map[int]*cluster, c px) {
		k := int(c.r)/16<<8 | int(c.g)/16<<4 | int(c.b)/16
		bn := bins[k]
		if bn == nil {
			bn = &cluster{}
			bins[k] = bn
		}
		// A running mean.
		bn.n++
		f := 1 / float64(bn.n)
		bn.r += (c.r - bn.r) * f
		bn.g += (c.g - bn.g) * f
		bn.b += (c.b - bn.b) * f
	}
	n := 0
	for y := 0; y+1 < h; y++ {
		for x := 0; x+1 < w; x++ {
			i := y*w + x
			if masked != nil && masked[i] {
				continue
			}
			n++
			add(every, all[i])
			p := ys[i]
			if delta(p, ys[i+1]) > same || delta(p, ys[i+w]) > same || delta(p, ys[i+w+1]) > same {
				continue
			}
			add(bins, all[i])
		}
	}
	return palette{solid: clusters(bins), all: clusters(every), n: n}
}

// clusters merges bins into colours, largest bin first: a bin joins the first colour within
// colourTolerance. Largest first.
func clusters(bins map[int]*cluster) []cluster {
	sorted := make([]*cluster, 0, len(bins))
	for _, bn := range bins {
		sorted = append(sorted, bn)
	}
	sort.Slice(sorted, func(i, j int) bool {
		if sorted[i].n != sorted[j].n {
			return sorted[i].n > sorted[j].n
		}
		return sorted[i].hex() < sorted[j].hex()
	})
	// Greedy clustering, largest bin first: a bin joins the first cluster within the tolerance.
	limit := maxYIQDelta * colourTolerance * colourTolerance
	var out []cluster
	for _, bn := range sorted {
		joined := false
		for k := range out {
			if delta(out[k].yiq(), bn.yiq()) <= limit {
				out[k].n += bn.n
				joined = true
				break
			}
		}
		if !joined {
			out = append(out, *bn)
		}
	}
	sort.SliceStable(out, func(i, j int) bool { return out[i].n > out[j].n })
	return out
}

// has says whether a palette uses a colour anywhere: at least paletteUse of its pixels (solid or
// not) are within the tolerance of it.
func (p palette) has(c cluster) bool {
	limit := maxYIQDelta * colourTolerance * colourTolerance
	n := 0
	for _, o := range p.all {
		if delta(o.yiq(), c.yiq()) <= limit {
			n += o.n
		}
	}
	if share(n, p.n) >= paletteUse {
		return true
	}
	for _, m := range p.more {
		if m.has(c) {
			return true
		}
	}
	return false
}

// ReadPalette is a whole screenshot's palette.
func ReadPalette(path string) (Palette, error) {
	img, err := readPNG(path)
	if err != nil {
		return palette{}, err
	}
	return paletteOf(img, nil), nil
}

// ComparePalettes compares two screenshots' palettes, over the part of them a View says.
func ComparePalettes(aPath, bPath string, v View) (PaletteResult, error) {
	return ComparePalettesIn(aPath, bPath, v, nil, nil)
}

// ComparePalettesIn is ComparePalettes where a colour counts as one the other design uses when
// it is in the other screenshot or in any of the other design's palettes given (its other
// screenshots, at the same colour scheme): a design language is the colours an app uses
// anywhere, not only in one component.
func ComparePalettesIn(aPath, bPath string, v View, morA, morB []Palette) (PaletteResult, error) {
	ai, err := readPNG(aPath)
	if err != nil {
		return PaletteResult{}, err
	}
	bi, err := readPNG(bPath)
	if err != nil {
		return PaletteResult{}, err
	}
	var masked []bool
	if !v.Empty() {
		ai, bi, masked = applyView(ai, bi, v)
	}
	pa, pb := paletteOf(ai, masked), paletteOf(bi, masked)
	pa.more, pb.more = morA, morB
	return comparePalettes(pa, pb), nil
}

func comparePalettes(ppa, ppb palette) PaletteResult {
	pa, pb := ppa.solid, ppb.solid
	var res PaletteResult
	if len(pa) == 0 || len(pb) == 0 {
		res.Value = 100
		return res
	}
	limit := maxYIQDelta * colourTolerance * colourTolerance
	drawn := func(p []cluster) int {
		n := 0
		for _, c := range p[1:] {
			n += c.n
		}
		return n
	}
	da, db := drawn(pa), drawn(pb)
	all := func(p []cluster) int { return p[0].n + drawn(p) }
	ta, tb := all(pa), all(pb)
	// significant: a colour of the palette, not the background.
	significant := func(c cluster, drawnTotal, total int) bool {
		return share(c.n, drawnTotal) >= paletteSignificant && share(c.n, total) >= paletteMinimum
	}
	swatches := func(p []cluster, drawnTotal, total int) []Swatch {
		out := []Swatch{{Hex: p[0].hex()}}
		for _, c := range p[1:] {
			if significant(c, drawnTotal, total) {
				out = append(out, Swatch{Hex: c.hex(), Share: round1(share(c.n, drawnTotal))})
			}
		}
		return out
	}
	res.A, res.B = swatches(pa, da, ta), swatches(pb, db, tb)
	res.BackgroundA, res.BackgroundB = pa[0].hex(), pb[0].hex()
	only := func(p []cluster, total, all int, other palette) (float64, []Swatch) {
		sum := 0.0
		var out []Swatch
		for _, c := range p[1:] {
			s := share(c.n, total)
			if !significant(c, total, all) || other.has(c) {
				continue
			}
			sum += s
			out = append(out, Swatch{Hex: c.hex(), Share: round1(s)})
		}
		return sum, out
	}
	sa, oa := only(pa, da, ta, ppb)
	sb, ob := only(pb, db, tb, ppa)
	res.OnlyA, res.OnlyB = oa, ob
	res.Value = round1(math.Max(sa, sb))
	if delta(pa[0].yiq(), pb[0].yiq()) > limit {
		res.Value, res.BackgroundsDiffer = 100, true
	}
	return res
}

func share(n, total int) float64 {
	if total == 0 {
		return 0
	}
	return 100 * float64(n) / float64(total)
}

func round1(v float64) float64 { return math.Round(v*10) / 10 }

// Note describes a palette difference in words, naming each side.
func (p PaletteResult) Note(a, b string) string {
	var parts []string
	if p.BackgroundsDiffer {
		parts = append(parts, fmt.Sprintf("backgrounds differ: %s %s, %s %s", a, p.BackgroundA, b, p.BackgroundB))
	}
	list := func(s []Swatch) string {
		var out []string
		for _, x := range s {
			out = append(out, fmt.Sprintf("%s (%g%%)", x.Hex, x.Share))
		}
		return strings.Join(out, ", ")
	}
	if len(p.OnlyA) > 0 {
		parts = append(parts, fmt.Sprintf("%s uses %s, which %s doesn't use", a, list(p.OnlyA), b))
	}
	if len(p.OnlyB) > 0 {
		parts = append(parts, fmt.Sprintf("%s uses %s, which %s doesn't use", b, list(p.OnlyB), a))
	}
	if len(parts) == 0 {
		return "the same colours"
	}
	return strings.Join(parts, "; ")
}
