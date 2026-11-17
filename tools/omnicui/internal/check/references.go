package check

import (
	"fmt"
	"image"
	_ "image/png"
	"os"
	"path/filepath"
	"strings"

	"omnicui/internal/compare"
	"omnicui/internal/contract"
	"omnicui/internal/project"
)

// compareReferences compares each target's screenshot of a test with the recorded references listed
// for it (references.yaml): at the size of the image, or of its crop, and in its colour scheme. A
// reference is the app the targets reproduce, so it is compared pixel by pixel whatever the
// target's design; what may differ is masked in the list.
func (r *runner) compareReferences(c *contract.Contract, test contract.Test, results []TestResult) []Comparison {
	var out []Comparison
	for _, ref := range r.refs {
		if ref.Contract != c.ID || ref.Test != test.Name {
			continue
		}
		img := filepath.Join(r.opts.Project.Root, ref.Image)
		w, h, err := imageSize(img)
		if err != nil {
			out = append(out, Comparison{Contract: c.ID, Test: test.Name, Kind: "reference", A: "reference", Warning: true, Note: err.Error()})
			continue
		}
		if ref.Crop != nil {
			w, h = int(ref.Crop.Width), int(ref.Crop.Height)
		}
		key := Combo{Viewport: contract.Viewport{Width: w, Height: h}, Scheme: ref.ColorScheme}.Key()
		compared := false
		for _, res := range results {
			shot, ok := res.Screenshots[key]
			if !ok {
				continue
			}
			compared = true
			out = append(out, r.referenceComparison(c, test, ref, img, res, shot, key))
		}
		if !compared {
			out = append(out, Comparison{Contract: c.ID, Test: test.Name, Kind: "reference", A: "reference", Warning: true,
				Note: fmt.Sprintf("no target took a %d×%d %s screenshot of this test to compare with %s", w, h, ref.ColorScheme, ref.Image)})
		}
	}
	return out
}

func (r *runner) referenceComparison(c *contract.Contract, test contract.Test, ref project.Reference, img string, res TestResult, shot, key string) Comparison {
	box := func(b project.Box) compare.Region {
		return compare.Region{X: b.X, Y: b.Y, Width: b.Width, Height: b.Height}
	}
	var v compare.View
	if ref.Crop != nil {
		cr := box(*ref.Crop)
		v.CropA = &cr
	}
	for _, m := range ref.Mask {
		v.MasksA = append(v.MasksA, box(m))
	}
	var notes, missing []string
	for _, p := range ref.MaskParts {
		if b, ok := res.Bounds[key][p]; ok && b.Width > 0 && b.Height > 0 {
			v.MasksB = append(v.MasksB, compare.Region{X: b.X, Y: b.Y, Width: b.Width, Height: b.Height})
		} else {
			missing = append(missing, p)
		}
	}
	if len(ref.Mask)+len(ref.MaskParts) > 0 {
		notes = append(notes, fmt.Sprintf("masked: %d area(s) of the reference, parts %s", len(ref.Mask), strings.Join(ref.MaskParts, ", ")))
	}
	if len(missing) > 0 {
		notes = append(notes, "not on screen to mask: "+strings.Join(missing, ", "))
	}
	diffDir := filepath.Join(r.runDir, "diff", c.Short(), slug(test.Name))
	os.MkdirAll(diffDir, 0o755)
	diff := filepath.Join(diffDir, fmt.Sprintf("reference-%s-%s.png", res.Target, key))
	m, note, err := compare.MeasureView(img, filepath.Join(r.runDir, shot), diff, v)
	if err != nil {
		note = err.Error()
		m = compare.Measures{Area: 100, Strength: 100, Unmatched: 100, Spot: 100}
	}
	m.Area, m.Strength, m.Unmatched, m.Spot = round(m.Area, 2), round(m.Strength, 1), round(m.Unmatched, 1), round(m.Spot, 2)
	th := c.Pixels
	over := Over(th, m)
	relDiff, _ := filepath.Rel(r.runDir, diff)
	cmp := Comparison{Contract: c.ID, Test: test.Name, Kind: "reference", A: "reference", B: res.Target, Combo: strings.ReplaceAll(key, "-", " "),
		Value: m.Area, Threshold: th.Area, Warning: len(over) > 0, Diff: relDiff,
		Note: joinNotes(joinNotes("against "+ref.Image, note), strings.Join(notes, "; ")), Measures: &m, Thresholds: &th, Over: over}
	if len(m.Spots) > 0 {
		cmp.SpotPart = partAt(m.Spots[0], nil, res.Bounds[key])
	}
	r.log.Info("reference", "contract", c.ID, "test", test.Name, "target", res.Target, "image", ref.Image,
		"area", m.Area, "strength", m.Strength, "shift", m.Shift, "spot", m.Spot, "over", strings.Join(over, ","))
	return cmp
}

func imageSize(path string) (int, int, error) {
	f, err := os.Open(path)
	if err != nil {
		return 0, 0, err
	}
	defer f.Close()
	cfg, _, err := image.DecodeConfig(f)
	if err != nil {
		return 0, 0, fmt.Errorf("%s: %w", path, err)
	}
	return cfg.Width, cfg.Height, nil
}
