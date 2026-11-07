package check

import (
	"fmt"
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
