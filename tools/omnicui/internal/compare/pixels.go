// Package compare measures how much two targets' screenshots and traces differ.
package compare

import (
	"fmt"
	"image"
	"image/color"
	"image/png"
	"math"
	"os"
	"sort"
)

// colourTolerance is how far apart two colours may be and still count as the same, on
// pixelmatch's YIQ scale (0 = identical, 1 = black against white).
const colourTolerance = 0.1

// Reach is how far, in pixels, a pixel may have moved and still count as matching. Two toolkits
// place and anti-alias the same glyph slightly differently; within this distance that is noise.
const Reach = 2

// SpotSize is the side, in pixels, of the square a Spot is measured over.
const SpotSize = 24

// StrongColour is the colour distance (0–1, 1 = black against white) above which a differing
// pixel counts as strongly off in the Spot measure. Anti-aliased edges are mostly under it.
const StrongColour = 0.35

// MaxShift is how far, in pixels, Measure looks for a differing pixel's nearest match. A pixel
// with no match that close counts as unmatched: content that is missing, new or recoloured.
const MaxShift = 8

const maxYIQDelta = 35215.0

// Measures are the four numbers a pair of screenshots is compared by. Each answers a different
// question, so that glyph noise (many thin edges, a shade off, a pixel or two away) can be told
// apart from a real difference (a solid block of the wrong colour, or something in the wrong place).
type Measures struct {
	// Area: the share of the drawn area that differs, as a percentage. A pixel differs when no
	// pixel within Reach of it in the other screenshot has its colour, in either direction.
	Area float64 `json:"area"`
	// Strength: how far off the differing pixels are in colour, as a percentage (100 = black
	// against white): the mean, over differing pixels, of the distance to the closest colour
	// within Reach in the other screenshot.
	Strength float64 `json:"strength"`
	// Shift: how far, in pixels, differing pixels have moved: the median distance to the nearest
	// pixel of the same colour in the other screenshot, over the differing pixels that have one
	// within MaxShift. 0 when none has.
	Shift float64 `json:"shift"`
	// Unmatched: the share of differing pixels with no match within MaxShift, as a percentage.
	Unmatched float64 `json:"unmatched"`
	// Spot: how concentrated the difference is. The share of the worst SpotSize×SpotSize spot's
	// pixels that are strongly off in colour (more than StrongColour), as a percentage. Glyph
	// noise is spread thinly over every edge; a wrong word, colour or opacity is concentrated.
	Spot float64 `json:"spot"`
	// Spots are the worst spots, worst first (at most five, not overlapping).
	Spots []Box `json:"spots,omitempty"`
	// Drawn is the number of pixels drawn in either screenshot.
	Drawn int `json:"drawn"`
}

// Box is a rectangle in screenshot pixels.
type Box struct {
	X int `json:"x"`
	Y int `json:"y"`
	W int `json:"w"`
	H int `json:"h"`
}

func (b Box) String() string { return fmt.Sprintf("%d×%d at %d,%d", b.W, b.H, b.X, b.Y) }

// Colours of the diff image.
var (
	diffStrong = color.RGBA{R: 230, G: 0, B: 40, A: 255}  // strongly off in colour
	diffWeak   = color.RGBA{R: 255, G: 150, B: 0, A: 255} // differs a little (mostly edges)
	diffBox    = color.RGBA{R: 30, G: 90, B: 230, A: 255} // outline of one of the worst spots
)

// Measure compares two PNG screenshots of the same size.
//
//   - The drawn area is every pixel that is not the background colour (the top-left pixel) in
//     either image. Measuring against it, not against the whole screenshot, keeps a small
//     component's real differences from disappearing into a large empty viewport.
//   - A pixel differs when no pixel within Reach of it in the other image has its colour, in
//     either direction.
//
// It writes an image to diffPath (if not empty): strongly differing pixels red, the other
// differing pixels orange, a blue box around each of the worst spots, and the rest faded.
// The note says why there are no measures (different sizes, nothing drawn).
func Measure(aPath, bPath, diffPath string) (m Measures, note string, err error) {
	ai, err := readPNG(aPath)
	if err != nil {
		return m, "", err
	}
	bi, err := readPNG(bPath)
	if err != nil {
		return m, "", err
	}
	if ai.Bounds().Dx() != bi.Bounds().Dx() || ai.Bounds().Dy() != bi.Bounds().Dy() {
		all := Measures{Area: 100, Strength: 100, Unmatched: 100, Spot: 100}
		return all, fmt.Sprintf("sizes differ: %d×%d against %d×%d", ai.Bounds().Dx(), ai.Bounds().Dy(), bi.Bounds().Dx(), bi.Bounds().Dy()), nil
	}
	a, b := toYIQ(ai), toYIQ(bi)
	w, h := a.w, a.h
	limit := maxYIQDelta * colourTolerance * colourTolerance
	bgA, bgB := a.px[0], b.px[0]

	// nearest returns, for the pixel at x,y of from, the distance (Chebyshev, in pixels) to the
	// nearest pixel of in with the same colour, searching up to radius; radius+1 if there is none.
	nearest := func(from, in *yiqImage, x, y, radius int) int {
		c := from.px[y*w+x]
		for r := 0; r <= radius; r++ {
			for dy := -r; dy <= r; dy++ {
				for dx := -r; dx <= r; dx++ {
					if max(abs(dx), abs(dy)) != r {
						continue // the ring at distance r only
					}
					xx, yy := x+dx, y+dy
					if xx >= 0 && yy >= 0 && xx < w && yy < h && delta(c, in.px[yy*w+xx]) <= limit {
						return r
					}
				}
			}
		}
		return radius + 1
	}
	// closest is the smallest colour distance between the pixel at x,y of from and any pixel
	// within Reach in in.
	closest := func(from, in *yiqImage, x, y int) float64 {
		c := from.px[y*w+x]
		best := math.Inf(1)
		for dy := -Reach; dy <= Reach; dy++ {
			for dx := -Reach; dx <= Reach; dx++ {
				xx, yy := x+dx, y+dy
				if xx >= 0 && yy >= 0 && xx < w && yy < h {
					best = math.Min(best, delta(c, in.px[yy*w+xx]))
				}
			}
		}
		return best
	}

	differs := make([]bool, w*h)
	strong := make([]bool, w*h)
	drawn, differ := 0, 0
	var strengthSum float64
	var shifts []int
	unmatched := 0
	for y := 0; y < h; y++ {
		for x := 0; x < w; x++ {
			i := y*w + x
			if delta(a.px[i], bgA) <= limit && delta(b.px[i], bgB) <= limit {
				continue
			}
			drawn++
			da := nearest(a, b, x, y, Reach)
			db := nearest(b, a, x, y, Reach)
			if da <= Reach && db <= Reach {
				continue
			}
			differs[i] = true
			differ++
			// The direction that failed decides how far off the pixel is; with both failing,
			// the worse of the two.
			s, str := 0, 0.0
			if da > Reach {
				s = max(s, nearest(a, b, x, y, MaxShift))
				str = math.Max(str, closest(a, b, x, y))
			}
			if db > Reach {
				s = max(s, nearest(b, a, x, y, MaxShift))
				str = math.Max(str, closest(b, a, x, y))
			}
			if s > MaxShift {
				unmatched++
			} else {
				shifts = append(shifts, s)
			}
			c := math.Sqrt(math.Min(1, str/maxYIQDelta))
			strengthSum += c
			strong[i] = c > StrongColour
		}
	}
	spot, spots := worstSpots(strong, w, h)

	if diffPath != "" {
		diff := image.NewRGBA(image.Rect(0, 0, w, h))
		for y := 0; y < h; y++ {
			for x := 0; x < w; x++ {
				i := y*w + x
				switch {
				case strong[i]:
					diff.Set(x, y, diffStrong)
				case differs[i]:
					diff.Set(x, y, diffWeak)
				default:
					g := uint8(255 + (a.px[i].y-255)*0.15)
					diff.Set(x, y, color.RGBA{R: g, G: g, B: g, A: 255})
				}
			}
		}
		for _, r := range spots {
			outline(diff, r)
		}
		if err := writePNG(diffPath, diff); err != nil {
			return m, "", err
		}
	}
	m.Drawn = drawn
	m.Spots = spots
	if drawn == 0 {
		return m, "nothing is drawn in either image", nil
	}
	m.Area = 100 * float64(differ) / float64(drawn)
	m.Spot = spot
	if differ > 0 {
		m.Strength = 100 * strengthSum / float64(differ)
		m.Unmatched = 100 * float64(unmatched) / float64(differ)
	}
	if len(shifts) > 0 {
		sort.Ints(shifts)
		m.Shift = float64(shifts[len(shifts)/2])
	}
	return m, "", nil
}

// Pixels is Measure's area alone: the share of the drawn area that differs.
func Pixels(aPath, bPath, diffPath string) (percent float64, note string, err error) {
	m, note, err := Measure(aPath, bPath, diffPath)
	return m.Area, note, err
}

func abs(x int) int {
	if x < 0 {
		return -x
	}
	return x
}

// worstSpots slides a SpotSize square over the image in steps of a quarter of its size, and
// returns the largest share (a percentage) of strong pixels in one square, and the boxes of the
// worst non-overlapping squares that have any (at most five, worst first). An image smaller than
// a spot is measured as one spot.
func worstSpots(strong []bool, w, h int) (float64, []Box) {
	// A summed-area table makes each square's count four lookups.
	sum := make([]int, (w+1)*(h+1))
	for y := 0; y < h; y++ {
		row := 0
		for x := 0; x < w; x++ {
			if strong[y*w+x] {
				row++
			}
			sum[(y+1)*(w+1)+x+1] = sum[y*(w+1)+x+1] + row
		}
	}
	sw, sh := min(SpotSize, w), min(SpotSize, h)
	count := func(x, y int) int {
		return sum[(y+sh)*(w+1)+x+sw] - sum[y*(w+1)+x+sw] - sum[(y+sh)*(w+1)+x] + sum[y*(w+1)+x]
	}
	type spot struct {
		box Box
		n   int
	}
	var all []spot
	step := max(1, SpotSize/4)
	for y := 0; y+sh <= h; y += step {
		for x := 0; x+sw <= w; x += step {
			if n := count(x, y); n > 0 {
				all = append(all, spot{Box{X: x, Y: y, W: sw, H: sh}, n})
			}
		}
	}
	if len(all) == 0 {
		return 0, nil
	}
	sort.SliceStable(all, func(i, j int) bool { return all[i].n > all[j].n })
	overlaps := func(a, b Box) bool {
		return a.X < b.X+b.W && b.X < a.X+a.W && a.Y < b.Y+b.H && b.Y < a.Y+a.H
	}
	var out []Box
	for _, s := range all {
		if len(out) == 5 {
			break
		}
		free := true
		for _, o := range out {
			free = free && !overlaps(s.box, o)
		}
		if free {
			out = append(out, s.box)
		}
	}
	return 100 * float64(all[0].n) / float64(sw*sh), out
}

func outline(img *image.RGBA, b Box) {
	for x := b.X; x < b.X+b.W; x++ {
		img.Set(x, b.Y, diffBox)
		img.Set(x, b.Y+b.H-1, diffBox)
	}
	for y := b.Y; y < b.Y+b.H; y++ {
		img.Set(b.X, y, diffBox)
		img.Set(b.X+b.W-1, y, diffBox)
	}
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
