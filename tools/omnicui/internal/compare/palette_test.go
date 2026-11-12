package compare

import (
	"image"
	"image/color"
	"path/filepath"
	"strings"
	"testing"
)

// paint draws a 320×240 picture: a background, and boxes of colour.
func paint(t *testing.T, bg color.RGBA, boxes ...struct {
	r image.Rectangle
	c color.RGBA
}) string {
	t.Helper()
	img := image.NewRGBA(image.Rect(0, 0, 320, 240))
	for y := 0; y < 240; y++ {
		for x := 0; x < 320; x++ {
			img.Set(x, y, bg)
			for _, b := range boxes {
				if (image.Point{x, y}).In(b.r) {
					img.Set(x, y, b.c)
				}
			}
		}
	}
	path := filepath.Join(t.TempDir(), "p.png")
	if err := writePNG(path, img); err != nil {
		t.Fatal(err)
	}
	return path
}

type box = struct {
	r image.Rectangle
	c color.RGBA
}

// R-PALETTE-1: the same colours in another layout measure 0; a colour only one uses counts by
// its share of that one's drawing; another background is 100%; a line of a colour (a glyph's
// stroke, an anti-aliased edge) is no colour of the palette.
func TestComparePalettes(t *testing.T) {
	bg := color.RGBA{0xfb, 0xfa, 0xf7, 255}
	blue, purple, grey := color.RGBA{0x1f, 0x5f, 0xbf, 255}, color.RGBA{0x7b, 0x2c, 0xbf, 255}, color.RGBA{0x8a, 0x8f, 0x84, 255}
	a := paint(t, bg, box{image.Rect(20, 20, 80, 50), blue}, box{image.Rect(20, 60, 80, 90), grey})
	moved := paint(t, bg, box{image.Rect(150, 100, 210, 130), blue}, box{image.Rect(150, 140, 230, 170), grey})
	r, err := ComparePalettes(a, moved, View{})
	if err != nil || r.Value != 0 || r.Note("a", "b") != "the same colours" {
		t.Fatalf("%+v %v", r, err)
	}
	// purple in place of blue: half of each one's drawing is in a colour the other doesn't use.
	other := paint(t, bg, box{image.Rect(20, 20, 80, 50), purple}, box{image.Rect(20, 60, 80, 90), grey})
	r, _ = ComparePalettes(a, other, View{})
	if r.Value != 50 || len(r.OnlyA) != 1 || r.OnlyA[0].Hex != "#1f5fbf" || len(r.OnlyB) != 1 || r.OnlyB[0].Hex != "#7b2cbf" {
		t.Fatalf("%+v", r)
	}
	if n := r.Note("a", "b"); n != "a uses #1f5fbf (50%), which b doesn't use; b uses #7b2cbf (50%), which a doesn't use" {
		t.Fatal(n)
	}
	// Another background.
	dark := paint(t, color.RGBA{0x1b, 0x1d, 0x19, 255}, box{image.Rect(20, 20, 80, 50), blue}, box{image.Rect(20, 60, 80, 90), grey})
	r, _ = ComparePalettes(a, dark, View{})
	if r.Value != 100 || !r.BackgroundsDiffer || !strings.HasPrefix(r.Note("a", "b"), "backgrounds differ: a #fbfaf7, b #1b1d19") {
		t.Fatalf("%+v %s", r, r.Note("a", "b"))
	}
	// A one-pixel purple line beside the boxes: no 2×2 block of it, so not in the palette.
	line := paint(t, bg, box{image.Rect(20, 20, 80, 50), blue}, box{image.Rect(20, 60, 80, 90), grey}, box{image.Rect(100, 20, 101, 200), purple})
	if r, _ = ComparePalettes(a, line, View{}); r.Value != 0 {
		t.Fatalf("%+v", r)
	}
	// Masked: a box left out of both pictures is not compared.
	r, _ = ComparePalettes(a, other, View{MasksA: []Region{{X: 20, Y: 20, Width: 60, Height: 30}}})
	if r.Value != 0 {
		t.Fatalf("%+v", r)
	}
}

// R-TRACE-DIR: where a traced point went, as an angle between two traces and in words.
func TestDirection(t *testing.T) {
	across := Analyze([]Sample{{MS: 0, X: 0, Y: 0}, {MS: 100, X: 10, Y: 0}, {MS: 200, X: 20, Y: 0}})
	up := Analyze([]Sample{{MS: 0, X: 0, Y: 0}, {MS: 100, X: 0, Y: -10}, {MS: 200, X: 0, Y: -20}})
	back := Analyze([]Sample{{MS: 0, X: 0, Y: 0}, {MS: 100, X: -10, Y: 0}, {MS: 200, X: -20, Y: 0}})
	if d, ok := Direction(across, up); !ok || d != 90 {
		t.Fatal(d, ok)
	}
	if d, _ := Direction(across, back); d != 180 {
		t.Fatal(d)
	}
	if d, _ := Direction(across, across); d != 0 {
		t.Fatal(d)
	}
	if Heading(across) != "right 20 px" || Heading(up) != "up 20 px" || Heading(back) != "left 20 px" {
		t.Fatal(Heading(across), Heading(up), Heading(back))
	}
	// A round trip's direction is towards its farthest point.
	trip := Analyze([]Sample{{MS: 0, X: 0, Y: 0}, {MS: 100, X: 0, Y: 15}, {MS: 200, X: 0, Y: 0}})
	if Heading(trip) != "down 15 px" {
		t.Fatal(Heading(trip))
	}
	// A traced property has no direction.
	v := 0.5
	if _, ok := Direction(across, Analyze([]Sample{{MS: 0, Value: &v}, {MS: 100, Value: &v}})); ok {
		t.Fatal("a property has no direction")
	}
}
