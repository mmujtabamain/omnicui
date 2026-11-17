package check

import (
	"fmt"
	"math"
	"strconv"
	"strings"
	"unicode/utf8"

	"omnicui/internal/compare"
	"omnicui/internal/contract"
)

// MeasureValue returns one of a screenshot comparison's measures by name.
func MeasureValue(m compare.Measures, name string) float64 {
	switch name {
	case "area":
		return m.Area
	case "strength":
		return m.Strength
	case "shift":
		return m.Shift
	case "spot":
		return m.Spot
	}
	return 0
}

// Over lists the measures above their thresholds, in display order.
func Over(th contract.PixelThresholds, m compare.Measures) []string {
	var out []string
	for _, n := range contract.PixelMeasureNames {
		if MeasureValue(m, n) > th.Get(n) {
			out = append(out, n)
		}
	}
	return out
}

// MeasureText is one measure as it is shown: "8.6%", "3 px", "8+ px".
func MeasureText(name string, v float64) string {
	if name == "shift" {
		return fmt.Sprintf("%.0f px", v)
	}
	if v >= 10 {
		return fmt.Sprintf("%.0f%%", v)
	}
	return fmt.Sprintf("%.1f%%", v)
}

// ThresholdText is a threshold as it is written in a contract.
func ThresholdText(name string, v float64) string {
	if name == "shift" {
		return fmt.Sprintf("%g px", v)
	}
	return fmt.Sprintf("%g%%", v)
}

// MeasuresRow is one pair of screenshots in a measures table.
type MeasuresRow struct {
	Label    string
	Measures compare.Measures
	Over     []string
	Note     string
	SpotPart string
}

// MeasuresTable prints one row per pair with all four measures, a mark on each measure above its
// threshold, and a last row with the thresholds:
//
//	                 area    strength  shift   spot
//	egui ↔ web       5.7%    14%       3 px    0.0%    pass
//	javafx ↔ web     75% ▲   52% ▲     4 px    41% ▲   warning: area, strength, spot above the threshold
//	thresholds       30%     28%       8 px    2%
func MeasuresTable(rows []MeasuresRow, th contract.PixelThresholds, indent string) string {
	labelW := utf8.RuneCountInString("thresholds")
	for _, r := range rows {
		labelW = max(labelW, utf8.RuneCountInString(r.Label))
	}
	const colW = 10
	var b strings.Builder
	b.WriteString(indent + pad("", labelW+1))
	for _, n := range contract.PixelMeasureNames {
		b.WriteString(pad(n, colW))
	}
	b.WriteString("\n")
	for _, r := range rows {
		b.WriteString(indent + pad(r.Label, labelW+1))
		over := map[string]bool{}
		for _, o := range r.Over {
			over[o] = true
		}
		for _, n := range contract.PixelMeasureNames {
			cell := MeasureText(n, MeasureValue(r.Measures, n))
			if over[n] {
				cell += " ▲"
			}
			b.WriteString(pad(cell, colW))
		}
		switch {
		case len(r.Over) > 0:
			b.WriteString("warning: " + strings.Join(r.Over, ", ") + " above the threshold")
			if len(r.Measures.Spots) > 0 && over["spot"] {
				b.WriteString(" (worst spot " + r.Measures.Spots[0].String())
				if r.SpotPart != "" {
					b.WriteString(", in `" + r.SpotPart + "`")
				}
				b.WriteString(")")
			}
		default:
			b.WriteString("pass")
		}
		if r.Note != "" {
			b.WriteString("; " + r.Note)
		}
		b.WriteString("\n")
	}
	b.WriteString(indent + pad("thresholds", labelW+1))
	for _, n := range contract.PixelMeasureNames {
		b.WriteString(pad(ThresholdText(n, th.Get(n)), colW))
	}
	b.WriteString("\n")
	return b.String()
}

// relationWords say a relation in a failure: "expected left of `b`".
var relationWords = map[string]string{
	"leftOf":     "left of",
	"rightOf":    "right of",
	"above":      "above",
	"below":      "below",
	"middleWith": "its middle level with",
}

// relationSlack is how far a relation may be off: a pixel, for rounding.
const relationSlack = 1.0

// relation checks where a part's box a is relative to another part's box b; it returns why not,
// or "" when it holds.
func relation(field string, a, b *Rect, self, other string) string {
	if a == nil || a.Width <= 0 || a.Height <= 0 {
		return fmt.Sprintf("but `%s` has no size on screen", self)
	}
	if b == nil || b.Width <= 0 || b.Height <= 0 {
		return fmt.Sprintf("but `%s` has no size on screen", other)
	}
	switch field {
	case "leftOf":
		if a.X+a.Width <= b.X+relationSlack {
			return ""
		}
		return fmt.Sprintf("but its right edge is at %s and `%s` starts at %s", px(a.X+a.Width), other, px(b.X))
	case "rightOf":
		if a.X >= b.X+b.Width-relationSlack {
			return ""
		}
		return fmt.Sprintf("but it starts at %s and `%s` ends at %s", px(a.X), other, px(b.X+b.Width))
	case "above":
		if a.Y+a.Height <= b.Y+relationSlack {
			return ""
		}
		return fmt.Sprintf("but its bottom is at %s and `%s` starts at %s", px(a.Y+a.Height), other, px(b.Y))
	case "below":
		if a.Y >= b.Y+b.Height-relationSlack {
			return ""
		}
		return fmt.Sprintf("but it starts at %s and `%s` ends at %s", px(a.Y), other, px(b.Y+b.Height))
	case "middleWith":
		d := (a.Y + a.Height/2) - (b.Y + b.Height/2)
		if math.Abs(d) <= relationSlack {
			return ""
		}
		dir := "lower"
		if d < 0 {
			dir = "higher"
		}
		return fmt.Sprintf("but it sits %s px %s", px(math.Abs(d)), dir)
	}
	return "but omnicui doesn't know this relation"
}

// px writes a coordinate without trailing zeros.
func px(v float64) string {
	return strconv.FormatFloat(math.Round(v*10)/10, 'f', -1, 64)
}
