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
	a, _, note, err := VisualProgress(fadeFrames(t, filepath.Join(d, "a"), lin, 200, 0, 1000.0/60))
	if err != nil || note != "" || !a.Moved {
		t.Fatal(err, note, a)
	}
	// The same fade on another clock: 30 frames a second, starting 70 ms later.
	b, _, _, _ := VisualProgress(fadeFrames(t, filepath.Join(d, "b"), lin, 200, 70, 1000.0/30))
	if diff, _ := Differ(a, b); diff > 18 {
		t.Errorf("same fade, other clock: %.1f ms apart", diff)
	}
	want, _ := Expected(200, "linear")
	if diff, _ := Differ(a, want); diff > 18 {
		t.Errorf("a linear 200 ms fade against linear 200 ms: %.1f ms", diff)
	}
	slow, _, _, _ := VisualProgress(fadeFrames(t, filepath.Join(d, "c"), lin, 400, 0, 1000.0/60))
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
	h, _, _, _ := VisualProgress(sparse)
	if diff, _ := Differ(a, h); diff > 18 {
		t.Errorf("a fade after a held first frame: %.1f ms apart", diff)
	}
	// A round trip (fade in, then out) has no single direction; it says so.
	there := fadeFrames(t, filepath.Join(d, "d"), lin, 200, 0, 1000.0/60)
	back := append(there, Frame{MS: there[len(there)-1].MS + 16, File: there[0].File})
	if _, _, note, _ := VisualProgress(back); note == "" {
		t.Error("a round trip should be explained, not measured")
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
