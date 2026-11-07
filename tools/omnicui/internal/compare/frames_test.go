package compare

import (
	"bytes"
	"fmt"
	"image"
	"image/color"
	"image/png"
	"math"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

// fadeFrames writes a clip of a grey square fading in on white with the given easing, sampled
// every frameMS, starting after delayMS.
func fadeFrames(t *testing.T, dir string, ease func(float64) float64, durMS, delayMS, frameMS float64) []Frame {
	t.Helper()
	os.MkdirAll(dir, 0o755)
	var out []Frame
	for i, ms := 0, 0.0; ms <= delayMS+durMS+100; i, ms = i+1, ms+frameMS {
		p := ease(math.Max(0, math.Min(1, (ms-delayMS)/durMS)))
		img := image.NewRGBA(image.Rect(0, 0, 60, 40))
		for y := 0; y < 40; y++ {
			for x := 0; x < 60; x++ {
				c := color.RGBA{255, 255, 255, 255}
				if x >= 10 && x < 40 && y >= 10 && y < 30 {
					v := uint8(255 - p*200)
					c = color.RGBA{v, v, v, 255}
				}
				img.Set(x, y, c)
			}
		}
		f := filepath.Join(dir, fmt.Sprintf("%d.png", i))
		w, _ := os.Create(f)
		png.Encode(w, img)
		w.Close()
		out = append(out, Frame{MS: ms, File: f})
	}
	return out
}

// R-MOTION-5: a fade has no position to trace, but its clip's visual progress curve compares
// like a trace: the same easing at different frame rates and start times agrees; another easing
// or duration doesn't.
func TestVisualProgress(t *testing.T) {
	d := t.TempDir()
	lin := func(p float64) float64 { return p }
	a, _, note, err := VisualProgress(fadeFrames(t, filepath.Join(d, "a"), lin, 200, 0, 1000.0/60), false)
	if err != nil || note != "" || !a.Moved {
		t.Fatal(err, note, a)
	}
	// The same fade on another clock: 30 frames a second, starting 70 ms later.
	b, _, _, _ := VisualProgress(fadeFrames(t, filepath.Join(d, "b"), lin, 200, 70, 1000.0/30), false)
	if diff, _ := Differ(a, b); diff > 18 {
		t.Errorf("same fade, other clock: %.1f ms apart", diff)
	}
	want, _ := Expected(200, "linear")
	if diff, _ := Differ(a, want); diff > 18 {
		t.Errorf("a linear 200 ms fade against linear 200 ms: %.1f ms", diff)
	}
	slow, _, _, _ := VisualProgress(fadeFrames(t, filepath.Join(d, "c"), lin, 400, 0, 1000.0/60), false)
	if diff, _ := Differ(a, slow); diff < 50 {
		t.Errorf("200 ms against 400 ms: only %.1f ms apart", diff)
	}
	// A screencast sends a frame only when the picture changes: a still start held for 300 ms
	// and then the same fade is the same motion, not a slow ramp from the first frame.
	held := fadeFrames(t, filepath.Join(d, "e"), lin, 200, 300, 1000.0/60)
	var sparse []Frame
	for _, f := range held {
		if f.MS == 0 || f.MS > 300 {
			sparse = append(sparse, f)
		}
	}
	h, _, _, _ := VisualProgress(sparse, true)
	if diff, _ := Differ(a, h); diff > 18 {
		t.Errorf("a fade after a held first frame: %.1f ms apart", diff)
	}
	// A target that sends a frame every pulse, with one slow pulse in the middle of the fade: the
	// fade went on during the gap, so levels are interpolated across it.
	var stalled []Frame
	for _, f := range fadeFrames(t, filepath.Join(d, "g"), lin, 200, 0, 1000.0/60) {
		if f.MS < 60 || f.MS > 110 {
			stalled = append(stalled, f)
		}
	}
	g, _, _, _ := VisualProgress(stalled, false)
	if diff, _ := Differ(a, g); diff > 18 {
		t.Errorf("a fade with a stalled pulse: %.1f ms apart", diff)
	}
	// A round trip (fade in, then straight out again) is measured out to its farthest frame and
	// back, and says so.
	there := fadeFrames(t, filepath.Join(d, "d"), lin, 200, 0, 1000.0/60)
	trip := func(frames []Frame) []Frame {
		out := append([]Frame{}, frames...)
		end := frames[len(frames)-1].MS
		for i := len(frames) - 2; i >= 0; i-- {
			out = append(out, Frame{MS: end + (end - frames[i].MS), File: frames[i].File})
		}
		return out
	}
	rt, _, note, err := VisualProgress(trip(there), false)
	if err != nil || !rt.Moved || !rt.RoundTrip || !strings.Contains(note, "round trip") {
		t.Fatalf("%+v %q %v", rt, note, err)
	}
	rt2, _, _, _ := VisualProgress(trip(fadeFrames(t, filepath.Join(d, "f"), lin, 200, 0, 1000.0/60)), false)
	if diff, _ := Differ(rt, rt2); diff > 5 {
		t.Errorf("two identical round trips: %.1f ms apart", diff)
	}
	slowTrip, _, _, _ := VisualProgress(trip(fadeFrames(t, filepath.Join(d, "h"), lin, 400, 0, 1000.0/60)), false)
	if diff, _ := Differ(rt, slowTrip); diff < 50 {
		t.Errorf("a 200 ms round trip against a 400 ms one: only %.1f ms apart", diff)
	}
}

// R-MOTION-12: a traced value or position that goes out and comes back (a star scaling 1 → 1.25
// → 1) is a round trip: measured, and compared by its timing.
func TestRoundTripTraces(t *testing.T) {
	spring := func(total float64) []Sample {
		var s []Sample
		for ms := 0.0; ms <= total+100; ms += 1000.0 / 60 {
			x := math.Min(1, ms/total)
			v := 1 + 0.25*math.Sin(math.Pi*x)
			s = append(s, Sample{MS: ms, Value: &v})
		}
		return s
	}
	a, b, slow := Analyze(spring(350)), Analyze(spring(350)), Analyze(spring(700))
	if !a.Moved || !a.RoundTrip || a.Distance < 0.24 {
		t.Fatalf("%+v", a)
	}
	if d, _ := Differ(a, b); d > 1 {
		t.Errorf("same spring: %.1f ms", d)
	}
	if d, _ := Differ(a, slow); d < 100 {
		t.Errorf("350 ms against 700 ms: only %.1f ms", d)
	}
	var pos []Sample
	for ms := 0.0; ms <= 300; ms += 1000.0 / 60 {
		pos = append(pos, Sample{MS: ms, X: 10 + 20*math.Sin(math.Pi*math.Min(1, ms/200)), Y: 5})
	}
	if c := Analyze(pos); !c.Moved || !c.RoundTrip {
		t.Fatalf("%+v", c)
	}
	still := 1.0
	if c := Analyze([]Sample{{MS: 0, Value: &still}, {MS: 16, Value: &still}}); c.Moved {
		t.Fatalf("a still value moved: %+v", c)
	}
}

// R-MOTION-6: opacity samples are analysed as values, and compare with the named easing.
func TestAnalyzeValues(t *testing.T) {
	var s []Sample
	for ms := 0.0; ms <= 300; ms += 1000.0 / 60 {
		v := math.Min(1, ms/200)
		s = append(s, Sample{MS: ms, Value: &v})
	}
	c := Analyze(s)
	want, _ := Expected(200, "linear")
	if d, _ := Differ(c, want); !c.Moved || d > 10 {
		t.Errorf("opacity 0→1 over 200 ms linear: %+v, %.1f ms", c, d)
	}
	flat := []Sample{{MS: 0, Value: ptr(1)}, {MS: 16, Value: ptr(0.99)}}
	if Analyze(flat).Moved {
		t.Error("an opacity change under 0.02 is no change")
	}
}

func ptr(v float64) *float64 { return &v }

// R-CLIP-2: clips are lossless animated PNGs: every frame, its timing, and no palette.
func TestAPNG(t *testing.T) {
	d := t.TempDir()
	frames := fadeFrames(t, filepath.Join(d, "f"), func(p float64) float64 { return p }, 100, 0, 50)
	out := filepath.Join(d, "clip.png")
	if err := APNG(frames, out); err != nil {
		t.Fatal(err)
	}
	data, _ := os.ReadFile(out)
	img, err := png.Decode(bytes.NewReader(data))
	if err != nil || img.Bounds().Dx() != 60 {
		t.Fatalf("not a PNG: %v", err)
	}
	chunks, _ := pngChunks(data)
	if len(chunks["acTL"]) != 1 || len(chunks["fcTL"]) != len(frames) || len(chunks["fdAT"]) < len(frames)-1 {
		t.Errorf("acTL %d, fcTL %d, fdAT %d for %d frames", len(chunks["acTL"]), len(chunks["fcTL"]), len(chunks["fdAT"]), len(frames))
	}
	if delay := int(chunks["fcTL"][0][20])<<8 | int(chunks["fcTL"][0][21]); delay != 50 {
		t.Errorf("first frame shown for %d ms, want 50", delay)
	}
	// Lossless: a pixel of the first frame is exactly what was drawn.
	if r, _, _, _ := img.At(20, 20).RGBA(); r>>8 != 255 {
		t.Errorf("first frame at 20,20: %v", img.At(20, 20))
	}
}

// R-MOTION-13: a round trip and a change that ends elsewhere are different motions.
func TestRoundTripAgainstOneWay(t *testing.T) {
	if d, note := Differ(Curve{Moved: true, RoundTrip: true, Times: make([]float64, len(Levels))}, Curve{Moved: true, Times: make([]float64, len(Levels))}); !math.IsInf(d, 1) || !strings.Contains(note, "came back") {
		t.Fatal(d, note)
	}
}
