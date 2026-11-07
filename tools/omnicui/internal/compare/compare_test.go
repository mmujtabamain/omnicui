package compare

import (
	"image"
	"image/color"
	"image/gif"
	"image/png"
	"math"
	"os"
	"path/filepath"
	"testing"
)

func solid(t *testing.T, path string, w, h int, c color.Color, paint func(img *image.RGBA)) string {
	t.Helper()
	img := image.NewRGBA(image.Rect(0, 0, w, h))
	for y := 0; y < h; y++ {
		for x := 0; x < w; x++ {
			img.Set(x, y, c)
		}
	}
	if paint != nil {
		paint(img)
	}
	f, err := os.Create(path)
	if err != nil {
		t.Fatal(err)
	}
	defer f.Close()
	png.Encode(f, img)
	return path
}

// R-PIXELS-1: identical screenshots differ by 0%.
func TestPixelsIdentical(t *testing.T) {
	d := t.TempDir()
	box := func(img *image.RGBA) { square(img, 10, 10, 20, color.Black) }
	a := solid(t, filepath.Join(d, "a.png"), 64, 64, color.White, box)
	b := solid(t, filepath.Join(d, "b.png"), 64, 64, color.White, box)
	if pct, note, err := Pixels(a, b, filepath.Join(d, "diff.png")); err != nil || pct != 0 || note != "" {
		t.Fatalf("%v %q %v", pct, note, err)
	}
}

func square(img *image.RGBA, x0, y0, size int, c color.Color) {
	for y := y0; y < y0+size; y++ {
		for x := x0; x < x0+size; x++ {
			img.Set(x, y, c)
		}
	}
}

// R-PIXELS-2: the share is taken over the drawn area, and a pixel that moved by up to Reach still
// matches; a diff image marks what differs.
func TestPixelsDrawnAreaAndReach(t *testing.T) {
	d := t.TempDir()
	a := solid(t, filepath.Join(d, "a.png"), 200, 200, color.White, func(img *image.RGBA) { square(img, 10, 10, 20, color.Black) })
	near := solid(t, filepath.Join(d, "near.png"), 200, 200, color.White, func(img *image.RGBA) { square(img, 12, 10, 20, color.Black) })
	far := solid(t, filepath.Join(d, "far.png"), 200, 200, color.White, func(img *image.RGBA) { square(img, 100, 100, 20, color.Black) })
	if pct, _, _ := Pixels(a, near, ""); pct != 0 {
		t.Errorf("moved by 2px: %.1f%%, want 0", pct)
	}
	// Moved far away: every drawn pixel differs, although they cover only 2% of the screenshot.
	pct, _, err := Pixels(a, far, filepath.Join(d, "diff.png"))
	if err != nil || pct != 100 {
		t.Fatalf("moved far: %.1f%%, %v", pct, err)
	}
	f, _ := os.Open(filepath.Join(d, "diff.png"))
	img, _ := png.Decode(f)
	if r, g, _, _ := img.At(15, 15).RGBA(); r>>8 != 230 || g != 0 {
		t.Errorf("a differing pixel should be red in the diff, got %v", img.At(15, 15))
	}
	// Half of the square recoloured: half of it differs.
	half := solid(t, filepath.Join(d, "half.png"), 200, 200, color.White, func(img *image.RGBA) {
		square(img, 10, 10, 20, color.Black)
		for y := 10; y < 30; y++ {
			for x := 20; x < 30; x++ {
				img.Set(x, y, color.RGBA{0, 90, 200, 255})
			}
		}
	})
	if pct, _, _ := Pixels(a, half, ""); pct < 40 || pct > 50 {
		t.Errorf("half recoloured: %.1f%%", pct)
	}
	blank := solid(t, filepath.Join(d, "blank.png"), 200, 200, color.White, nil)
	if pct, note, _ := Pixels(blank, blank, ""); pct != 0 || note == "" {
		t.Errorf("nothing drawn: %v %q", pct, note)
	}
}

// R-PIXELS-3: screenshots of different sizes are 100% different, with a note.
func TestPixelsSizes(t *testing.T) {
	d := t.TempDir()
	a := solid(t, filepath.Join(d, "a.png"), 10, 10, color.White, nil)
	b := solid(t, filepath.Join(d, "b.png"), 20, 10, color.White, nil)
	_ = b
	if pct, note, _ := Pixels(a, b, ""); pct != 100 || note == "" {
		t.Fatalf("%v %q", pct, note)
	}
	if m, _, _ := Measure(a, b, ""); m.Spot != 100 || m.Strength != 100 {
		t.Errorf("sizes differ: every measure is at its worst: %+v", m)
	}
}

// R-PIXELS-4: the four measures answer different questions. A square moved by 5 px is a shift;
// a recoloured block is strong and concentrated (spot); a faint recolour is weak.
func TestMeasures(t *testing.T) {
	d := t.TempDir()
	base := func(img *image.RGBA) { square(img, 40, 40, 30, color.Black) }
	a := solid(t, filepath.Join(d, "a.png"), 160, 160, color.White, base)

	moved := solid(t, filepath.Join(d, "moved.png"), 160, 160, color.White, func(img *image.RGBA) { square(img, 45, 40, 30, color.Black) })
	m, note, err := Measure(a, moved, "")
	if err != nil || note != "" {
		t.Fatal(err, note)
	}
	if m.Shift < 3 || m.Shift > 5 || m.Unmatched != 0 {
		t.Errorf("moved by 5 px: shift %v, unmatched %v; want 3–5 px, all matched", m.Shift, m.Unmatched)
	}

	recoloured := solid(t, filepath.Join(d, "recoloured.png"), 160, 160, color.White, func(img *image.RGBA) {
		base(img)
		square(img, 40, 40, 30, color.RGBA{200, 0, 0, 255})
	})
	m, _, _ = Measure(a, recoloured, filepath.Join(d, "diff.png"))
	if m.Area != 100 || m.Spot != 100 || m.Strength < StrongColour*100 || m.Unmatched != 100 {
		t.Errorf("recoloured: %+v", m)
	}
	if len(m.Spots) == 0 || m.Spots[0].W != SpotSize {
		t.Errorf("recoloured: the worst spot should be boxed: %+v", m.Spots)
	}
	f, _ := os.Open(filepath.Join(d, "diff.png"))
	img, _ := png.Decode(f)
	if c := color.RGBAModel.Convert(img.At(55, 55)).(color.RGBA); c != diffStrong {
		t.Errorf("a strongly differing pixel should be red, got %v", c)
	}

	// A faint recolour (black to a dark grey) differs everywhere, but weakly, so it is no spot.
	faint := solid(t, filepath.Join(d, "faint.png"), 160, 160, color.White, func(img *image.RGBA) {
		square(img, 40, 40, 30, color.RGBA{60, 60, 60, 255})
	})
	m, _, _ = Measure(a, faint, "")
	if m.Area != 100 || m.Strength >= StrongColour*100 || m.Spot != 0 {
		t.Errorf("faint recolour: %+v", m)
	}
}

// R-PIXELS-5: noise spread thinly over many edges scores a high area but a low spot, which is
// what tells it from one concentrated difference of the same area.
func TestSpotTellsNoiseFromAConcentratedDifference(t *testing.T) {
	d := t.TempDir()
	lines := func(img *image.RGBA) {
		for y := 10; y < 150; y += 10 {
			for x := 10; x < 150; x++ {
				img.Set(x, y, color.Black)
			}
		}
	}
	a := solid(t, filepath.Join(d, "a.png"), 160, 160, color.White, lines)
	// Every line one pixel lower and one shade lighter at its ends: thin, everywhere.
	noisy := solid(t, filepath.Join(d, "noisy.png"), 160, 160, color.White, func(img *image.RGBA) {
		for y := 13; y < 153; y += 10 {
			for x := 10; x < 150; x++ {
				img.Set(x, y, color.RGBA{90, 90, 90, 255})
			}
		}
	})
	// The same lines, with one small block where a word should be.
	block := solid(t, filepath.Join(d, "block.png"), 160, 160, color.White, func(img *image.RGBA) {
		lines(img)
		square(img, 60, 60, 16, color.Black)
	})
	n, _, _ := Measure(a, noisy, "")
	b, _, _ := Measure(a, block, "")
	if n.Spot >= b.Spot || b.Spot < 30 {
		t.Errorf("noise spot %.1f%% should be well under a block's %.1f%%", n.Spot, b.Spot)
	}
}

func samplesOf(fn func(t float64) float64, durMS, delayMS float64, dist float64) []Sample {
	var out []Sample
	for ms := 0.0; ms <= delayMS+durMS+100; ms += 1000.0 / 60 {
		p := math.Max(0, math.Min(1, (ms-delayMS)/durMS))
		out = append(out, Sample{MS: ms, X: 10 + dist*fn(p), Y: 5})
	}
	return out
}

// R-MOTION-1: traces are lined up before they are compared, so a target that reacts later still matches.
func TestAnalyzeAlignsStart(t *testing.T) {
	lin := func(p float64) float64 { return p }
	a := Analyze(samplesOf(lin, 200, 0, 20))
	b := Analyze(samplesOf(lin, 200, 80, 20))
	if !a.Moved || a.Distance != 20 {
		t.Fatalf("%+v", a)
	}
	if d, _ := Differ(a, b); d > 1 {
		t.Errorf("same curve, later start: differ by %.2f ms", d)
	}
	if math.Abs(a.Times[4]-100) > 1 || math.Abs(b.Times[4]-180) > 1 || math.Abs(a.Duration-192) > 1 {
		t.Errorf("linear 50%% at %.1f and %.1f ms, duration %.1f", a.Times[4], b.Times[4], a.Duration)
	}
}

// R-MOTION-2: a trace that follows a CSS timing function matches Expected for it.
func TestExpectedMatchesEasing(t *testing.T) {
	for _, e := range []string{"linear", "ease", "ease-in", "ease-out", "ease-in-out", "cubic-bezier(0.2, 0, 0, 1)"} {
		want, err := Expected(200, e)
		if err != nil {
			t.Fatal(err)
		}
		x1, y1, x2, y2, _ := bezier(e)
		fn := func(p float64) float64 { // y(x): find the bezier parameter for x = p
			lo, hi := 0.0, 1.0
			for i := 0; i < 60; i++ {
				m := (lo + hi) / 2
				if cubic(x1, x2, m) < p {
					lo = m
				} else {
					hi = m
				}
			}
			return cubic(y1, y2, (lo+hi)/2)
		}
		got := Analyze(samplesOf(fn, 200, 30, 20))
		// At 60 samples a second, a steep curve is measured to within a few ms.
		if d, _ := Differ(got, want); d > 4 {
			t.Errorf("%s: measured curve differs from Expected by %.2f ms", e, d)
		}
	}
	if _, err := Expected(200, "bouncy"); err == nil {
		t.Error("an unknown easing must be an error")
	}
}

// R-MOTION-3: a different easing or duration is measured as a difference in ms.
func TestDifferMeasures(t *testing.T) {
	lin := Analyze(samplesOf(func(p float64) float64 { return p }, 300, 0, 20))
	out, _ := Expected(200, "ease-out")
	if d, _ := Differ(lin, out); d < 50 {
		t.Errorf("300ms linear against 200ms ease-out should differ a lot, got %.1f ms", d)
	}
	still := Analyze([]Sample{{0, 1, 1}, {16, 1.2, 1}, {32, 1, 1}})
	if still.Moved {
		t.Error("under a pixel is not a movement")
	}
	if d, note := Differ(lin, still); !math.IsInf(d, 1) || note == "" {
		t.Errorf("moving against still: %v %q", d, note)
	}
	if d, _ := Differ(still, still); d != 0 {
		t.Errorf("still against still: %v", d)
	}
}

// R-RECORD-1: recorded frames become an animated GIF that keeps their timing.
func TestClip(t *testing.T) {
	d := t.TempDir()
	var frames []Frame
	for i, c := range []color.Color{color.White, color.Black, color.White} {
		frames = append(frames, Frame{MS: float64(i) * 50, File: solid(t, filepath.Join(d, string(rune('a'+i))+".png"), 8, 8, c, nil)})
	}
	out := filepath.Join(d, "clip.gif")
	if err := Clip(frames, out); err != nil {
		t.Fatal(err)
	}
	f, _ := os.Open(out)
	g, err := gif.DecodeAll(f)
	if err != nil || len(g.Image) != 3 || g.Delay[0] != 5 || g.Delay[2] != 100 {
		t.Fatalf("%v frames=%d delays=%v", err, len(g.Image), g.Delay)
	}
}
