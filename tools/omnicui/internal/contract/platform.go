package contract

import (
	"fmt"
	"regexp"
	"sort"
	"strings"
)

// Platforms: what a target runs on, and the selectors a contract uses to say something for some
// platforms only. A target is described by its name, its design (target.yaml `design:`), its
// forms (target.yaml `platform:`) and its OS (what the adapter reports in advertise). A selector
// names one of those, or is `default`; when several match a target, the most specific wins:
// target > OS > form > design > default.

// Forms and OSes are the closed vocabulary of platform tags.
var (
	Forms = []string{"desktop", "web", "mobile", "tablet"}
	OSes  = []string{"linux", "windows", "macos", "ios", "android"}
)

// Platform is one target, as selectors see it.
type Platform struct {
	Target string   `json:"target"`
	Design string   `json:"design,omitempty"`
	OS     string   `json:"os,omitempty"`
	Forms  []string `json:"forms,omitempty"`
}

// Rank says how specifically a selector matches the platform: 5 its target, 4 its OS, 3 one of
// its forms, 2 its design, 1 default, 0 not at all.
func (p Platform) Rank(sel string) int {
	switch {
	case sel == p.Target:
		return 5
	case p.OS != "" && sel == p.OS:
		return 4
	case contains(p.Forms, sel):
		return 3
	case p.Design != "" && sel == p.Design:
		return 2
	case sel == "default":
		return 1
	}
	return 0
}

// Matches says whether any of the selectors matches the platform (default matches all).
func (p Platform) Matches(sels []string) bool {
	for _, s := range sels {
		if p.Rank(s) > 0 {
			return true
		}
	}
	return false
}

// Pick returns the value of the most specific selector that matches the platform, and that
// selector; ok is false when none matches.
func Pick[T any](byPlatform map[string]T, p Platform) (v T, sel string, ok bool) {
	best := 0
	for _, k := range sortedSelectors(byPlatform) {
		if r := p.Rank(k); r > best {
			best, v, sel, ok = r, byPlatform[k], k, true
		}
	}
	return
}

func sortedSelectors[T any](m map[string]T) []string {
	out := make([]string, 0, len(m))
	for k := range m {
		out = append(out, k)
	}
	sort.Strings(out)
	return out
}

func contains(xs []string, x string) bool {
	for _, y := range xs {
		if y == x {
			return true
		}
	}
	return false
}

// String says what the platform is, for messages: "android (mobile, android)".
func (p Platform) String() string {
	var tags []string
	tags = append(tags, p.Forms...)
	if p.OS != "" {
		tags = append(tags, p.OS)
	}
	if p.Design != "" {
		tags = append(tags, "design "+p.Design)
	}
	if len(tags) == 0 {
		return p.Target
	}
	return fmt.Sprintf("%s (%s)", p.Target, strings.Join(tags, ", "))
}

// Aspects are what a part holds the same between two targets.
type Aspects uint8

const (
	Layout Aspects = 1 << iota
	Colours
	Motion
	pixelsBit
	// Pixels implies layout, colours and motion.
	Pixels = pixelsBit | Layout | Colours | Motion
	// NoAspects: nothing visual is compared (behaviour still is).
	NoAspects Aspects = 0
)

// AspectNames are the words a contract uses, in the order they are shown.
var AspectNames = []string{"pixels", "layout", "colours", "motion", "none"}

// ParseAspects reads `pixels`, `none`, or a list of layout, colours and motion.
func ParseAspects(v any) (Aspects, error) {
	switch x := v.(type) {
	case string:
		return parseAspect(x)
	case []any:
		var a Aspects
		for _, e := range x {
			s, _ := e.(string)
			b, err := parseAspect(s)
			if err != nil {
				return 0, err
			}
			a |= b
		}
		return a, nil
	}
	return 0, fmt.Errorf("expected pixels, none, or a list of layout, colours and motion")
}

func parseAspect(s string) (Aspects, error) {
	switch s {
	case "pixels":
		return Pixels, nil
	case "layout":
		return Layout, nil
	case "colours", "colors":
		return Colours, nil
	case "motion":
		return Motion, nil
	case "none":
		return NoAspects, nil
	}
	return 0, fmt.Errorf("`%s` is not an aspect: write pixels, layout, colours, motion or none", s)
}

// Has says whether every aspect of b is in a.
func (a Aspects) Has(b Aspects) bool { return a&b == b }

// String is how a set of aspects is written: "pixels", "none", "layout + colours".
func (a Aspects) String() string {
	if a.Has(Pixels) {
		return "pixels"
	}
	var out []string
	for _, x := range []struct {
		b Aspects
		n string
	}{{Layout, "layout"}, {Colours, "colours"}, {Motion, "motion"}} {
		if a.Has(x.b) {
			out = append(out, x.n)
		}
	}
	if len(out) == 0 {
		return "none"
	}
	return strings.Join(out, " + ")
}

// MarshalText writes aspects for results.json.
func (a Aspects) MarshalText() ([]byte, error) { return []byte(a.String()), nil }

// Input is one platform's form of a named input (platforms.yaml): a key, one of omnicui's own
// actions (press, focus, hover), or an action the targets define (do), or nothing at all.
type Input struct {
	None   bool           `json:"none,omitempty"`
	Key    string         `json:"key,omitempty"`
	Action string         `json:"action,omitempty"`
	Do     string         `json:"do,omitempty"`
	With   map[string]any `json:"with,omitempty"`
}

// String is what a target was sent, for the report: "key Alt+Left", "do back", "none".
func (in Input) String() string {
	switch {
	case in.None:
		return "none"
	case in.Key != "":
		return "key " + in.Key
	case in.Do != "":
		return "do " + in.Do
	}
	return in.Action
}

// Inputs are the project's named inputs (platforms.yaml): for each name, its form per selector.
type Inputs map[string]map[string]Input

var inputNameRE = regexp.MustCompile(`^[a-z][a-zA-Z0-9]*$`)

// UnknownSelectors are the selectors a contract (and platforms.yaml) uses that match no form,
// OS, target or design of the project: a typo, or a target that was removed. names are the
// project's target and design names.
func UnknownSelectors(cs []*Contract, names []string) []Problem {
	known := append(append(append([]string{"default"}, Forms...), OSes...), names...)
	var out []Problem
	seen := map[string]bool{}
	check := func(file string, line int, sel, where string) {
		if contains(known, sel) || seen[file+sel] {
			return
		}
		seen[file+sel] = true
		m := fmt.Sprintf("`%s` (%s) is not a platform, a target or a design", sel, where)
		if s := closest(sel, known); s != "" {
			m += fmt.Sprintf("; did you mean `%s`?", s)
		}
		out = append(out, Problem{File: rel(file), Line: line, Message: m, Warning: true})
	}
	for _, c := range cs {
		for sel := range c.ViewportsBy {
			check(c.File, 1, sel, "viewports")
		}
		for _, p := range c.Parts {
			for sel := range p.Same {
				check(c.File, 1, sel, "same of "+p.Name)
			}
		}
		for _, t := range c.Tests {
			for _, sel := range append(append([]string{}, t.Only...), t.Except...) {
				check(c.File, t.Line, sel, "only/except")
			}
			for _, e := range t.Expect {
				for _, ch := range e.Checks {
					for sel := range ch.ByPlatform {
						check(c.File, e.Line, sel, e.Part+"."+ch.Field)
					}
				}
			}
		}
		for name, forms := range c.Inputs {
			for sel := range forms {
				check("platforms.yaml", 1, sel, "input "+name)
			}
		}
	}
	sort.SliceStable(out, func(i, j int) bool { return out[i].File+fmt.Sprint(out[i].Line) < out[j].File+fmt.Sprint(out[j].Line) })
	return out
}
