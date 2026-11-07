package compare

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"hash/crc32"
	"image"
	"image/png"
	"io"
	"math"
	"os"
)

// Motion from a recording. Traces follow one point's position, which says nothing about a fade,
// a blur or a colour change. A recording shows all of them, but each target records on its own
// clock (a screencast that sends a frame only when the page changes, snapshots in real time,
// simulated frames) and from its own moment, so frame N of one clip is not frame N of another.
//
// VisualProgress sidesteps both: it reduces a clip to how far the picture has gone from its first
// frame towards its last, over time, a 0→1 curve like a trace's. Curves are then lined up and
// compared the same way as traces, which removes the difference in start time, and they are
// compared at levels of progress, not at frame numbers, which removes the difference in frame
// rate.

// VisualProgress is a clip's progress towards its last frame. For each frame, the distance to the
// last frame is the mean colour difference over the pixels that change at all in the clip, and
// progress is 1 − distance/distance(first, last). Note says why there is no curve: the clip
// ends where it started (a round trip, which traces measure better), or nothing changes.
//
// held says the frames come only when the picture changes (a screencast): a level reached after a
// gap of more than 40 ms is reached at the later frame, not somewhere along the gap. Otherwise
// (a frame every pulse, which a slow pulse delays) levels are interpolated across gaps.
func VisualProgress(frames []Frame, held bool) (Curve, []float64, string, error) {
	if len(frames) < 2 {
		return Curve{}, nil, "fewer than two frames", nil
	}
	imgs := make([]*yiqImage, len(frames))
	for i, f := range frames {
		img, err := readPNG(f.File)
		if err != nil {
			return Curve{}, nil, "", err
		}
		imgs[i] = toYIQ(img)
		if imgs[i].w != imgs[0].w || imgs[i].h != imgs[0].h {
			return Curve{}, nil, fmt.Sprintf("frame %d is %d×%d, frame 0 %d×%d", i, imgs[i].w, imgs[i].h, imgs[0].w, imgs[0].h), nil
		}
	}
	first, last := imgs[0], imgs[len(imgs)-1]
	limit := maxYIQDelta * colourTolerance * colourTolerance
	// The pixels that change at some point: comparing only them keeps a still background from
	// diluting the motion, and makes the measure the same for a small part and a whole page.
	changing := make([]int, 0)
	for p := range first.px {
		for _, im := range imgs[1:] {
			if delta(first.px[p], im.px[p]) > limit {
				changing = append(changing, p)
				break
			}
		}
	}
	if len(changing) == 0 {
		return Curve{}, nil, "nothing changes in the clip", nil
	}
	dist := func(a, b *yiqImage) float64 {
		sum := 0.0
		for _, p := range changing {
			sum += math.Sqrt(delta(a.px[p], b.px[p]) / maxYIQDelta)
		}
		return sum / float64(len(changing))
	}
	total := dist(first, last)
	peak := 0.0
	for _, im := range imgs {
		peak = math.Max(peak, dist(first, im))
	}
	if total < 0.25*peak {
		return Curve{}, nil, "the clip ends close to where it started (a round trip); trace a part to compare it", nil
	}
	ms := make([]float64, len(frames))
	progress := make([]float64, len(frames))
	for i, im := range imgs {
		ms[i] = frames[i].MS
		progress[i] = 1 - dist(im, last)/total
	}
	if held {
		return AnalyzeHeld(ms, progress, 0.01, 40), progress, "", nil
	}
	return AnalyzeValues(ms, progress, 0.01), progress, "", nil
}

// APNG writes frames as an animated PNG: lossless, so gradients and textures keep their colours
// (a GIF's 256-colour palette dithers them), and played by every current browser. Each frame is
// shown until the next one's time; the last for a second.
func APNG(frames []Frame, out string) error {
	if len(frames) == 0 {
		return fmt.Errorf("no frames")
	}
	var buf bytes.Buffer
	buf.WriteString("\x89PNG\r\n\x1a\n")
	seq := uint32(0)
	var w, h int
	for i, f := range frames {
		img, err := readPNG(f.File)
		if err != nil {
			return err
		}
		if i == 0 {
			w, h = img.Bounds().Dx(), img.Bounds().Dy()
		} else if img.Bounds().Dx() != w || img.Bounds().Dy() != h {
			// Frames must all be the size of the animation: draw a different-sized frame at the
			// top left of one of the right size.
			canvas := image.NewNRGBA(image.Rect(0, 0, w, h))
			for y := 0; y < min(h, img.Bounds().Dy()); y++ {
				for x := 0; x < min(w, img.Bounds().Dx()); x++ {
					canvas.Set(x, y, img.At(img.Bounds().Min.X+x, img.Bounds().Min.Y+y))
				}
			}
			img = canvas
		}
		var enc bytes.Buffer
		if err := (&png.Encoder{CompressionLevel: png.BestSpeed}).Encode(&enc, img); err != nil {
			return err
		}
		chunks, err := pngChunks(enc.Bytes())
		if err != nil {
			return err
		}
		if i == 0 {
			writeChunk(&buf, "IHDR", chunks["IHDR"][0])
			actl := make([]byte, 8)
			binary.BigEndian.PutUint32(actl[0:], uint32(len(frames)))
			binary.BigEndian.PutUint32(actl[4:], 0) // loop forever
			writeChunk(&buf, "acTL", actl)
		}
		delay := 1000.0
		if i+1 < len(frames) {
			delay = math.Max(1, math.Round(frames[i+1].MS-f.MS))
		}
		fctl := make([]byte, 26)
		binary.BigEndian.PutUint32(fctl[0:], seq)
		seq++
		binary.BigEndian.PutUint32(fctl[4:], uint32(w))
		binary.BigEndian.PutUint32(fctl[8:], uint32(h))
		// x and y offsets 0
		binary.BigEndian.PutUint16(fctl[20:], uint16(math.Min(delay, 65535)))
		binary.BigEndian.PutUint16(fctl[22:], 1000) // delay is in ms
		fctl[24], fctl[25] = 0, 0                   // dispose none, blend source
		writeChunk(&buf, "fcTL", fctl)
		for _, data := range chunks["IDAT"] {
			if i == 0 {
				writeChunk(&buf, "IDAT", data)
				continue
			}
			fdat := make([]byte, 4+len(data))
			binary.BigEndian.PutUint32(fdat, seq)
			seq++
			copy(fdat[4:], data)
			writeChunk(&buf, "fdAT", fdat)
		}
	}
	writeChunk(&buf, "IEND", nil)
	return os.WriteFile(out, buf.Bytes(), 0o644)
}

// pngChunks splits an encoded PNG into its chunks' data, by type, in order.
func pngChunks(b []byte) (map[string][][]byte, error) {
	out := map[string][][]byte{}
	r := bytes.NewReader(b[8:])
	for {
		var n uint32
		if err := binary.Read(r, binary.BigEndian, &n); err == io.EOF {
			return out, nil
		} else if err != nil {
			return nil, err
		}
		head := make([]byte, 4+int(n)+4)
		if _, err := io.ReadFull(r, head); err != nil {
			return nil, err
		}
		typ := string(head[:4])
		out[typ] = append(out[typ], head[4:4+n])
	}
}

func writeChunk(w *bytes.Buffer, typ string, data []byte) {
	binary.Write(w, binary.BigEndian, uint32(len(data)))
	crc := crc32.NewIEEE()
	crc.Write([]byte(typ))
	crc.Write(data)
	w.WriteString(typ)
	w.Write(data)
	binary.Write(w, binary.BigEndian, crc.Sum32())
}
