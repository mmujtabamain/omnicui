package compare

import (
	"image"
	"image/color/palette"
	"image/draw"
	"image/gif"
	"math"
	"os"
)

// Frame is one frame a target recorded.
type Frame struct {
	MS   float64 `json:"ms"`
	File string  `json:"file"`
}

// Clip turns recorded frames into an animated GIF, keeping their timing.
func Clip(frames []Frame, out string) error {
	anim := &gif.GIF{}
	for i, f := range frames {
		img, err := readPNG(f.File)
		if err != nil {
			return err
		}
		p := image.NewPaletted(img.Bounds(), palette.Plan9)
		draw.FloydSteinberg.Draw(p, img.Bounds(), img, img.Bounds().Min)
		delay := 100 // hold the last frame for a second
		if i+1 < len(frames) {
			delay = int(math.Max(2, math.Round((frames[i+1].MS-f.MS)/10)))
		}
		anim.Image = append(anim.Image, p)
		anim.Delay = append(anim.Delay, delay)
	}
	w, err := os.Create(out)
	if err != nil {
		return err
	}
	defer w.Close()
	return gif.EncodeAll(w, anim)
}
