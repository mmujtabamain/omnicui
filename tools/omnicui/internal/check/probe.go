package check

import (
	"errors"
	"fmt"
	"image"
	_ "image/png"
	"os"
	"path/filepath"

	"omnicui/internal/contract"
	"omnicui/internal/rpc"
	"time"
)

// The render check. Advertising `screenshot` is a claim; a target can make it and still not be
// able to draw (a browser whose GPU process crashes while simple pages load, a headless screen
// smaller than the viewport, a missing graphics driver). So before any test, omnicui asks each
// target that advertises both `screenshot` and `probe` to draw a known picture at the largest
// viewport of the run, and checks it:
//
//   - a white background,
//   - a black 32×32 square at 8,8,
//   - a red (#ff0000) 32×32 square whose bottom-right corner is 8 px from the viewport's.
//
// The red square catches a picture cut off or scaled down; the black one a blank picture.

// probeViewport is the largest width and the largest height of the contracts' viewports.
func probeViewport(cs []*contract.Contract) contract.Viewport {
	v := contract.DefaultViewport
	first := true
	for _, c := range cs {
		for _, vp := range c.Viewports {
			if first {
				v, first = vp, false
			}
			v.Width, v.Height = max(v.Width, vp.Width), max(v.Height, vp.Height)
		}
	}
	return v
}

// checkProbe says what is wrong with a probe picture, or nil.
func checkProbe(path string, vp contract.Viewport) error {
	f, err := os.Open(path)
	if err != nil {
		return fmt.Errorf("the target replied, but wrote no picture (%v)", err)
	}
	defer f.Close()
	img, _, err := image.Decode(f)
	if err != nil {
		return fmt.Errorf("the picture is not a PNG: %v", err)
	}
	b := img.Bounds()
	if b.Dx() != vp.Width || b.Dy() != vp.Height {
		return fmt.Errorf("the picture is %d×%d, not the %d×%d asked for: the target's screen or window is not the viewport's size", b.Dx(), b.Dy(), vp.Width, vp.Height)
	}
	at := func(x, y int) (int, int, int) {
		r, g, bl, _ := img.At(b.Min.X+x, b.Min.Y+y).RGBA()
		return int(r >> 8), int(g >> 8), int(bl >> 8)
	}
	if r, g, bl := at(vp.Width/2, vp.Height/2); r < 192 || g < 192 || bl < 192 {
		return fmt.Errorf("the background is rgb(%d,%d,%d), not white: the target draws something else, or nothing", r, g, bl)
	}
	if r, g, bl := at(24, 24); r > 64 || g > 64 || bl > 64 {
		return fmt.Errorf("the black square at the top left is missing (rgb(%d,%d,%d) there): the target is not drawing", r, g, bl)
	}
	if r, g, bl := at(vp.Width-24, vp.Height-24); r < 192 || g > 64 || bl > 64 {
		return fmt.Errorf("the red square at the bottom right is missing (rgb(%d,%d,%d) there): the picture is cut off or scaled", r, g, bl)
	}
	return nil
}

// RenderCheck asks a started adapter for the probe picture at vp, writes it to path, and says
// what is wrong with it, or nil. `omnicui targets` uses it; a check uses targetRun.probe.
func RenderCheck(client *rpc.Client, vp contract.Viewport, path string, timeout time.Duration) error {
	os.MkdirAll(filepath.Dir(path), 0o755)
	err := client.Call("probe", map[string]any{"path": path, "viewport": map[string]int{"width": vp.Width, "height": vp.Height}}, timeout, nil)
	var re *rpc.RemoteError
	if errors.As(err, &re) {
		return errors.New(re.Message)
	}
	if err != nil {
		return err
	}
	return checkProbe(path, vp)
}

// ProbeViewport is the viewport the render check uses for these contracts.
func ProbeViewport(cs []*contract.Contract) contract.Viewport { return probeViewport(cs) }

// probe runs the render check on one target. It returns "" when it passed or was not run, and
// otherwise why it failed.
func (tr *targetRun) probe(info *TargetInfo) string {
	if !tr.can["screenshot"] {
		return ""
	}
	if !tr.can["probe"] {
		info.Probe = "not advertised"
		return ""
	}
	vp := probeViewport(tr.r.opts.Contracts)
	path := filepath.Join(tr.r.runDir, tr.t.Name, "probe.png")
	os.MkdirAll(filepath.Dir(path), 0o755)
	err := tr.call("probe", map[string]any{"path": path, "viewport": map[string]int{"width": vp.Width, "height": vp.Height}}, nil)
	if err == nil {
		err = checkProbe(path, vp)
	}
	if err != nil {
		info.Probe = "failed: " + err.Error()
		tr.log.Warn("render check failed", "error", err.Error())
		return err.Error()
	}
	info.Probe = "passed"
	tr.log.Info("render check passed", "width", vp.Width, "height", vp.Height)
	return ""
}
