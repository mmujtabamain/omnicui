package compare

import (
	"fmt"
	"math"
	"regexp"
	"slices"
	"strconv"
)

// Sample is one point of a trace: a time in milliseconds and a position in pixels, or, for a
// traced property such as opacity, its value.
type Sample struct {
	MS    float64  `json:"ms"`
	X     float64  `json:"x"`
	Y     float64  `json:"y"`
	Value *float64 `json:"value,omitempty"`
}

// Levels are the fractions of the movement at which traces are compared.
var Levels = []float64{0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9}

// startLevel and endLevel bound the reported duration.
const startLevel, endLevel = 0.02, 0.98

// Curve is a trace reduced to the times at which the point reached each level.
type Curve struct {
	Moved    bool    `json:"moved"`
	Distance float64 `json:"distance"`
	// Times[i] is when the point reached Levels[i], in ms from the first sample.
	Times []float64 `json:"times"`
	// Duration is from 2% to 98% of the movement, in ms.
	Duration float64 `json:"duration"`
	// RoundTrip: it went out and came back (a press that scales a star up and back, a hover in
	// and out). The levels are then of the whole trip: up to 50% on the way out, the rest back.
	RoundTrip bool `json:"roundTrip,omitempty"`
}

// roundTrip is the progress of a change that ends where it started: half of it on the way out
// to the farthest sample (fromFirst, the distance from the first sample), half on the way back
// (toLast, the distance to the last sample). Nil when it never went anywhere.
func roundTrip(fromFirst, toLast []float64, minChange float64) []float64 {
	k := 0
	for i, d := range fromFirst {
		if d > fromFirst[k] {
			k = i
		}
	}
	out, back := fromFirst[k], toLast[k]
	if out < minChange || back <= 0 {
		return nil
	}
	p := make([]float64, len(fromFirst))
	for i := range fromFirst {
		if i <= k {
			p[i] = 0.5 * fromFirst[i] / out
		} else {
			p[i] = 0.5 + 0.5*(1-math.Min(1, toLast[i]/back))
		}
	}
	return p
}

// levels turns a progress from 0 to 1 over time into a Curve. A level is reached between two
// samples less than maxGap ms apart by interpolation, and otherwise at the later sample.
func levels(ms, progress []float64, maxGap, distance float64) Curve {
	n := len(ms)
	at := func(level float64) float64 {
		for i := 1; i < n; i++ {
			if progress[i] >= level {
				if ms[i]-ms[i-1] > maxGap {
					return ms[i]
				}
				a, b := progress[i-1], progress[i]
				f := 1.0
				if b > a {
					f = (level - a) / (b - a)
				}
				return ms[i-1] + f*(ms[i]-ms[i-1])
			}
		}
		return ms[n-1]
	}
	c := Curve{Moved: true, Distance: distance}
	for _, l := range Levels {
		c.Times = append(c.Times, at(l))
	}
	c.Duration = at(endLevel) - at(startLevel)
	return c
}

// Analyze turns samples into a Curve. A point that moves less than one pixel didn't move.
// Samples that carry a value (a traced property) are analysed by AnalyzeValues instead.
func Analyze(samples []Sample) Curve {
	if len(samples) > 0 && samples[0].Value != nil {
		ms := make([]float64, len(samples))
		vs := make([]float64, len(samples))
		for i, s := range samples {
			ms[i] = s.MS
			if s.Value != nil {
				vs[i] = *s.Value
			}
		}
		return AnalyzeValues(ms, vs, 0.02)
	}
	if len(samples) < 2 {
		return Curve{}
	}
	p0, pn := samples[0], samples[len(samples)-1]
	dx, dy := pn.X-p0.X, pn.Y-p0.Y
	dist := math.Hypot(dx, dy)
	if dist < 1 {
		from, to := make([]float64, len(samples)), make([]float64, len(samples))
		ms := make([]float64, len(samples))
		for i, s := range samples {
			from[i], to[i], ms[i] = math.Hypot(s.X-p0.X, s.Y-p0.Y), math.Hypot(s.X-pn.X, s.Y-pn.Y), s.MS
		}
		if p := roundTrip(from, to, 1); p != nil {
			c := levels(ms, p, math.Inf(1), slices.Max(from))
			c.RoundTrip = true
			return c
		}
		return Curve{Distance: dist}
	}
	progress := make([]float64, len(samples))
	for i, s := range samples {
		progress[i] = ((s.X-p0.X)*dx + (s.Y-p0.Y)*dy) / (dist * dist)
	}
	at := func(level float64) float64 {
		for i := 1; i < len(samples); i++ {
			if progress[i] >= level {
				a, b := progress[i-1], progress[i]
				f := 1.0
				if b > a {
					f = (level - a) / (b - a)
				}
				return samples[i-1].MS + f*(samples[i].MS-samples[i-1].MS)
			}
		}
		return pn.MS
	}
	c := Curve{Moved: true, Distance: dist}
	for _, l := range Levels {
		c.Times = append(c.Times, at(l))
	}
	c.Duration = at(endLevel) - at(startLevel)
	return c
}

// AnalyzeValues turns a value over time (an opacity, the picture's progress towards its final
// state) into a Curve: the times at which it covered each level of its change from the first
// value to the last. A change smaller than minChange is no change.
func AnalyzeValues(ms, values []float64, minChange float64) Curve {
	return analyzeValues(ms, values, minChange, math.Inf(1))
}

// AnalyzeHeld is AnalyzeValues for frames that are held until the next one: a screencast sends a
// frame only when the picture changes, so a long gap means "unchanged until then", not a slow
// ramp. A level is reached between two frames less than maxGap ms apart by interpolation, and
// otherwise at the later frame.
func AnalyzeHeld(ms, values []float64, minChange, maxGap float64) Curve {
	return analyzeValues(ms, values, minChange, maxGap)
}

func analyzeValues(ms, values []float64, minChange, maxGap float64) Curve {
	n := len(values)
	if n < 2 || len(ms) != n {
		return Curve{}
	}
	v0, vn := values[0], values[n-1]
	change := vn - v0
	if math.Abs(change) < minChange {
		from, to := make([]float64, n), make([]float64, n)
		for i, v := range values {
			from[i], to[i] = math.Abs(v-v0), math.Abs(v-vn)
		}
		if p := roundTrip(from, to, minChange); p != nil {
			c := levels(ms, p, maxGap, slices.Max(from))
			c.RoundTrip = true
			return c
		}
		return Curve{Distance: math.Abs(change)}
	}
	progress := make([]float64, n)
	for i, v := range values {
		progress[i] = (v - v0) / change
	}
	return levels(ms, progress, maxGap, math.Abs(change))
}

// Differ returns the largest difference in ms between two curves at the same level, after
// shifting one so that they line up as well as possible. Targets react to input after different
// delays, and sample at different moments, so only the shape and length of the movement are
// compared: its timing function and its duration.
func Differ(a, b Curve) (float64, string) {
	switch {
	case !a.Moved && !b.Moved:
		return 0, "neither moved"
	case a.Moved != b.Moved:
		return math.Inf(1), "one changed and the other didn't"
	case a.RoundTrip != b.RoundTrip:
		return math.Inf(1), "one went out and came back, the other ended somewhere else"
	}
	shift := 0.0
	for i := range Levels {
		shift += a.Times[i] - b.Times[i]
	}
	shift /= float64(len(Levels))
	worst, at := 0.0, 0.0
	for i := range Levels {
		if d := math.Abs(a.Times[i] - b.Times[i] - shift); d > worst {
			worst, at = d, Levels[i]
		}
	}
	return worst, fmt.Sprintf("largest at %d%% of the movement", int(at*100+0.5))
}

// Expected is the curve a CSS timing function gives over a duration.
func Expected(durationMS float64, easing string) (Curve, error) {
	x1, y1, x2, y2, err := bezier(easing)
	if err != nil {
		return Curve{}, err
	}
	timeAt := func(level float64) float64 { return durationMS * solveX(x1, y1, x2, y2, level) }
	c := Curve{Moved: true, Distance: 1}
	for _, l := range Levels {
		c.Times = append(c.Times, timeAt(l))
	}
	c.Duration = timeAt(endLevel) - timeAt(startLevel)
	return c, nil
}

var cubicRE = regexp.MustCompile(`^cubic-bezier\(\s*([0-9.]+)\s*,\s*(-?[0-9.]+)\s*,\s*([0-9.]+)\s*,\s*(-?[0-9.]+)\s*\)$`)

// bezier returns the control points of a CSS easing.
func bezier(easing string) (x1, y1, x2, y2 float64, err error) {
	switch easing {
	case "linear":
		return 0, 0, 1, 1, nil
	case "ease":
		return 0.25, 0.1, 0.25, 1, nil
	case "ease-in":
		return 0.42, 0, 1, 1, nil
	case "ease-out":
		return 0, 0, 0.58, 1, nil
	case "ease-in-out":
		return 0.42, 0, 0.58, 1, nil
	}
	m := cubicRE.FindStringSubmatch(easing)
	if m == nil {
		return 0, 0, 0, 0, fmt.Errorf("unknown easing %q", easing)
	}
	v := make([]float64, 4)
	for i := range v {
		v[i], _ = strconv.ParseFloat(m[i+1], 64)
	}
	return v[0], v[1], v[2], v[3], nil
}

func cubic(p1, p2, t float64) float64 {
	u := 1 - t
	return 3*u*u*t*p1 + 3*u*t*t*p2 + t*t*t
}

// solveX finds the time fraction at which the easing's output reaches level.
func solveX(x1, y1, x2, y2, level float64) float64 {
	lo, hi := 0.0, 1.0
	for i := 0; i < 60; i++ {
		mid := (lo + hi) / 2
		if cubic(y1, y2, mid) < level {
			lo = mid
		} else {
			hi = mid
		}
	}
	return cubic(x1, x2, (lo+hi)/2)
}
