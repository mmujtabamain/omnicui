// Package compare measures how much two targets' screenshots and traces differ.
package compare

import (
	"fmt"
	"image"
	"image/color"
	"image/png"
	"os"
)

// colourTolerance is how far apart two colours may be and still count as the same, on
// pixelmatch's YIQ scale (0 = identical, 1 = black against white).
const colourTolerance = 0.1

// Reach is how far, in pixels, a pixel may have moved and still count as matching. Two toolkits
// place and anti-alias the same glyph slightly differently; within this distance that is noise.
const Reach = 2

const maxYIQDelta = 35215.0

// Pixels compares two PNG screenshots of the same size and returns the share of the drawn area
// that differs, as a percentage.
//
//   - The drawn area is every pixel that is not the background colour (the top-left pixel) in
//     either image. Measuring against it, not against the whole screenshot, keeps a small
//     component's real differences from disappearing into a large empty viewport.
//   - A pixel differs when no pixel within Reach of it in the other image has its colour, in
//     either direction.
//
// It writes an image to diffPath (if not empty): differing pixels red, the rest faded.
func Pixels(aPath, bPath, diffPath string) (percent float64, note string, err error) {
	ai, err := readPNG(aPath)
	if err != nil {
		return 0, "", err
	}
	bi, err := readPNG(bPath)
	if err != nil {
		return 0, "", err
	}
	if ai.Bounds().Dx() != bi.Bounds().Dx() || ai.Bounds().Dy() != bi.Bounds().Dy() {
		return 100, fmt.Sprintf("sizes differ: %d×%d against %d×%d", ai.Bounds().Dx(), ai.Bounds().Dy(), bi.Bounds().Dx(), bi.Bounds().Dy()), nil
	}
	a, b := toYIQ(ai), toYIQ(bi)
	w, h := a.w, a.h
	limit := maxYIQDelta * colourTolerance * colourTolerance
	bgA, bgB := a.px[0], b.px[0]
	found := func(from, in *yiqImage, x, y int) bool {
		c := from.px[y*w+x]
		for dy := -Reach; dy <= Reach; dy++ {
			for dx := -Reach; dx <= Reach; dx++ {
				xx, yy := x+dx, y+dy
				if xx >= 0 && yy >= 0 && xx < w && yy < h && delta(c, in.px[yy*w+xx]) <= limit {
					return true
				}
			}
		}
		return false
	}
	diff := image.NewRGBA(image.Rect(0, 0, w, h))
	drawn, differ := 0, 0
	for y := 0; y < h; y++ {
		for x := 0; x < w; x++ {
			i := y*w + x
			inked := delta(a.px[i], bgA) > limit || delta(b.px[i], bgB) > limit
			if inked {
				drawn++
			}
			if inked && (!found(a, b, x, y) || !found(b, a, x, y)) {
				differ++
				diff.Set(x, y, color.RGBA{R: 230, G: 0, B: 40, A: 255})
			} else {
				g := uint8(255 + (a.px[i].y-255)*0.15)
				diff.Set(x, y, color.RGBA{R: g, G: g, B: g, A: 255})
			}
		}
	}
	if diffPath != "" {
		if err := writePNG(diffPath, diff); err != nil {
			return 0, "", err
		}
	}
	if drawn == 0 {
		return 0, "nothing is drawn in either image", nil
	}
	return 100 * float64(differ) / float64(drawn), "", nil
}

type yiq struct{ y, i, q float64 }

type yiqImage struct {
	w, h int
	px   []yiq
}

func toYIQ(img image.Image) *yiqImage {
	b := img.Bounds()
	out := &yiqImage{w: b.Dx(), h: b.Dy(), px: make([]yiq, b.Dx()*b.Dy())}
	for y := 0; y < b.Dy(); y++ {
		for x := 0; x < b.Dx(); x++ {
			r, g, bl := rgb(img.At(b.Min.X+x, b.Min.Y+y))
			out.px[y*b.Dx()+x] = yiq{
				y: r*0.29889531 + g*0.58662247 + bl*0.11448223,
				i: r*0.59597799 - g*0.27417610 - bl*0.32180189,
				q: r*0.21147017 - g*0.52261711 + bl*0.31114694,
			}
		}
	}
	return out
}

// delta is pixelmatch's perceptual colour difference in YIQ space.
func delta(a, b yiq) float64 {
	y, i, q := a.y-b.y, a.i-b.i, a.q-b.q
	return 0.5053*y*y + 0.299*i*i + 0.1957*q*q
}

func readPNG(path string) (image.Image, error) {
	f, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	defer f.Close()
	img, err := png.Decode(f)
	if err != nil {
		return nil, fmt.Errorf("%s: %v", path, err)
	}
	return img, nil
}

func writePNG(path string, img image.Image) error {
	f, err := os.Create(path)
	if err != nil {
		return err
	}
	defer f.Close()
	return png.Encode(f, img)
}

// rgb blends a colour onto white and returns 0–255 channels.
func rgb(c color.Color) (float64, float64, float64) {
	r, g, b, a := c.RGBA()
	if a == 0 {
		return 255, 255, 255
	}
	af := float64(a) / 65535
	un := func(v uint32) float64 { return float64(v) / float64(a) * 255 }
	blend := func(v float64) float64 { return 255 + (v-255)*af }
	return blend(un(r)), blend(un(g)), blend(un(b))
}
