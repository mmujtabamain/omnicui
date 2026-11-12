// Package contract loads and checks contract files.
package contract

import (
	"fmt"
	"regexp"
	"strings"
)

// Field is a typed prop or event field.
type Field struct {
	Name       string `json:"name"`
	Type       string `json:"type"`
	Enum       []any  `json:"enum,omitempty"`
	Default    any    `json:"default,omitempty"`
	HasDefault bool   `json:"hasDefault"`
}

// Event is a reported event and its fields.
type Event struct {
	Name   string  `json:"name"`
	Fields []Field `json:"fields"`
}

// Part is a piece of the component that tests can act on and check.
type Part struct {
	Name string `json:"name"`
	Role string `json:"role"`
	// ID is the part's id in every target: <contract id>.<part name>.
	ID string `json:"id"`
	// Optional: shown by some targets or designs only (window controls in a desktop toolbar, the
	// thumb of a switch that another design draws as a checkbox). Masks and crops skip it where it
	// isn't; so do traces, clips and expectations on it, and a test whose steps act on it is not
	// measured there.
	Optional bool `json:"optional,omitempty"`
	// Same: what the part holds the same between targets, per platform selector (`default` for
	// every platform). Empty: the contract's (or, across designs, colours and motion).
	Same map[string]Aspects `json:"same,omitempty"`
}

// Step is one user action. Times is already 1 or more.
type Step struct {
	// Action is press, focus, key, type, drag or hover; or "do", for an action a target defines:
	// Do is its name and With its parameters, passed on unchanged. Part may be empty for "do".
	// Or "input": Input names one of the project's inputs (platforms.yaml), sent to each target
	// in its platform's form.
	Action string         `json:"action"`
	Input  string         `json:"input,omitempty"`
	Do     string         `json:"do,omitempty"`
	With   map[string]any `json:"with,omitempty"`
	Part   string         `json:"part"`
	Key    string         `json:"key,omitempty"`
	Text   string         `json:"text,omitempty"`
	DX     float64        `json:"dx,omitempty"`
	DY     float64        `json:"dy,omitempty"`
	Times  int            `json:"times"`
	Line   int            `json:"-"`
}

// TextMatch is a looser `text` or `name`: the value contains a string, or matches a regular
// expression (Go's syntax, RE2), instead of being exactly a string.
type TextMatch struct {
	Contains string `json:"contains,omitempty"`
	Matches  string `json:"matches,omitempty"`
	re       *regexp.Regexp
}

// NewTextMatch makes a TextMatch; an error when the regular expression doesn't compile.
func NewTextMatch(contains, matches string) (TextMatch, error) {
	m := TextMatch{Contains: contains, Matches: matches}
	if matches != "" {
		re, err := regexp.Compile(matches)
		if err != nil {
			return m, err
		}
		m.re = re
	}
	return m, nil
}

// Match says whether a value read from a target matches.
func (m TextMatch) Match(s string) bool {
	if m.re != nil {
		return m.re.MatchString(s)
	}
	if m.Matches != "" {
		re, err := regexp.Compile(m.Matches)
		return err == nil && re.MatchString(s)
	}
	return strings.Contains(s, m.Contains)
}

// String is how a message shows what was expected.
func (m TextMatch) String() string {
	if m.Matches != "" {
		return fmt.Sprintf("text matching /%s/", m.Matches)
	}
	return fmt.Sprintf("text containing %q", m.Contains)
}

// Check is one field of one part, as read after the steps. Want is a string, a boolean, a
// number, or a TextMatch for text and name.
type Check struct {
	Field string `json:"field"`
	Want  any    `json:"want"`
	// ByPlatform: the value per platform selector, when it differs by platform (Want is then
	// the default's). Each one is a platform exception, and the expectation says why.
	ByPlatform map[string]any `json:"byPlatform,omitempty"`
}

// Expectation is either about a part (Part and Checks) or about an event (Event, Times, Last).
type Expectation struct {
	Part   string         `json:"part,omitempty"`
	Checks []Check        `json:"checks,omitempty"`
	Event  string         `json:"event,omitempty"`
	Times  *int           `json:"times,omitempty"`
	Last   map[string]any `json:"last,omitempty"`
	// Why: the reason a value differs by platform.
	Why  string `json:"why,omitempty"`
	Line int    `json:"-"`
}

// Has reports whether the expectation checks a field.
func (e Expectation) Has(field string) bool {
	for _, c := range e.Checks {
		if c.Field == field {
			return true
		}
	}
	return false
}

// Timing is a named timing function a trace is checked against.
type Timing struct {
	DurationMS float64 `json:"durationMs"`
	Easing     string  `json:"easing"`
}

// TraceSpec follows one point on a part while the steps run, or one of its properties.
type TraceSpec struct {
	Part string `json:"part"`
	// Property is "position" (the default: Point is followed) or "opacity" (the part's opacity as
	// drawn, its ancestors' included).
	Property string  `json:"property"`
	Point    string  `json:"point,omitempty"`
	Timing   *Timing `json:"timing,omitempty"`
	Line     int     `json:"-"`
}

// Test is one entry of the contract's tests list.
type Test struct {
	Name       string         `json:"name"`
	Props      map[string]any `json:"props,omitempty"`
	Steps      []Step         `json:"steps,omitempty"`
	Expect     []Expectation  `json:"expect,omitempty"`
	Screenshot bool           `json:"screenshot,omitempty"`
	// Mask: parts left out of the screenshot comparison (with the contract's mask); ScreenshotPart:
	// compare only this part's box on each target.
	Mask           []string    `json:"mask,omitempty"`
	ScreenshotPart string      `json:"screenshotPart,omitempty"`
	Trace          []TraceSpec `json:"trace,omitempty"`
	Record         bool        `json:"record,omitempty"`
	// RecordPart: record: { part: x } measures the clip over that part\'s box only.
	RecordPart string `json:"recordPart,omitempty"`
	// AllCombinations takes the traces and the clip at every viewport × colour scheme, not only
	// the first.
	AllCombinations bool `json:"allCombinations,omitempty"`
	// Keyboard is set on the tests omnicui adds for `keyboard: true`: press Part with Key, and
	// expect what pressing it with the pointer does.
	Keyboard *KeyboardCheck `json:"keyboard,omitempty"`
	// Only and Except: platform selectors; the test is not applicable on a target that matches
	// none of Only, or one of Except.
	Only   []string `json:"only,omitempty"`
	Except []string `json:"except,omitempty"`
	Line   int      `json:"-"`
}

// Applies says whether a test runs on a platform (only / except).
func (t Test) Applies(p Platform) bool {
	if len(t.Only) > 0 && !p.Matches(t.Only) {
		return false
	}
	return !(len(t.Except) > 0 && p.Matches(t.Except))
}

// Viewport is a window size, with the props a test gets at that size (a test's own props win).
type Viewport struct {
	Width  int            `json:"width"`
	Height int            `json:"height"`
	Props  map[string]any `json:"props,omitempty"`
}

// KeyboardCheck is a generated test: a key on a part must do what a pointer press does.
type KeyboardCheck struct {
	Part string `json:"part"`
	Key  string `json:"key"`
}

// KeyboardKeys are the keys that must activate each role, as WAI-ARIA's authoring practices
// have it: a button on Enter and Space, a switch and a checkbox on Space.
var KeyboardKeys = map[string][]string{"button": {"Enter", "Space"}, "switch": {"Space"}, "checkbox": {"Space"}}

// Contract is one loaded and checked contract file.
type Contract struct {
	ID        string     `json:"id"`
	File      string     `json:"file"`
	Props     []Field    `json:"props"`
	Events    []Event    `json:"events"`
	Parts     []Part     `json:"parts"`
	Tests     []Test     `json:"tests"`
	Viewports []Viewport `json:"viewports"`
	// ViewportsBy: viewports per platform selector, when they differ by platform (Viewports is
	// then the default's, or the first given).
	ViewportsBy  map[string][]Viewport `json:"viewportsBy,omitempty"`
	ColorSchemes []string              `json:"colorSchemes"`
	// Keyboard adds, for every button, switch and checkbox, a test per key that activates its
	// role: the key must do what a pointer press does, on every target.
	Keyboard bool `json:"keyboard,omitempty"`
	// Mask: parts left out of every screenshot comparison of this contract.
	Mask []string `json:"mask,omitempty"`
	// Pixels are the screenshot thresholds; TimingThreshold is in milliseconds.
	Pixels          PixelThresholds `json:"pixelThresholds"`
	TimingThreshold float64         `json:"timingThreshold"`
	// MotionThreshold is for clips compared by visual progress, in ms. It is wider than timing's:
	// a clip shows everything that changes (a colour, a highlight), each target's frames come on
	// its own clock, and correct renderings measured up to 30 ms apart.
	MotionThreshold float64 `json:"motionThreshold"`
	// Same: what parts hold the same between targets of one design (default pixels); SameSet says
	// the contract set it.
	Same    Aspects `json:"same"`
	SameSet bool    `json:"-"`
	// LayoutThreshold: how far a part's box may move between targets compared by layout, in
	// percent of the component's width.
	LayoutThreshold float64 `json:"layoutThreshold"`
	// Inputs: the project's named inputs (platforms.yaml), for steps that name one.
	Inputs Inputs `json:"-"`
	// PaletteThreshold is for screenshots of targets of different designs, compared by the
	// colours they draw with: the share, in percent, of one's drawing in colours the other doesn't
	// use. DirectionThreshold is the angle, in degrees, two targets may move a traced point apart.
	PaletteThreshold   float64 `json:"paletteThreshold"`
	DirectionThreshold float64 `json:"directionThreshold"`
}

// Short is the last segment of the id ("counter" for "ui.counter").
func (c *Contract) Short() string {
	for i := len(c.ID) - 1; i >= 0; i-- {
		if c.ID[i] == '.' {
			return c.ID[i+1:]
		}
	}
	return c.ID
}

// PartByName returns the named part, or nil.
func (c *Contract) PartByName(name string) *Part {
	for i := range c.Parts {
		if c.Parts[i].Name == name {
			return &c.Parts[i]
		}
	}
	return nil
}

// EventByName returns the named event, or nil.
func (c *Contract) EventByName(name string) *Event {
	for i := range c.Events {
		if c.Events[i].Name == name {
			return &c.Events[i]
		}
	}
	return nil
}

// Defaults applied when a contract leaves them out.
var (
	DefaultViewport        = Viewport{Width: 320, Height: 240}
	DefaultColorSchemes    = []string{"light"}
	DefaultTimingThreshold = 20.0
	DefaultMotionThreshold = 40.0
	// DefaultPaletteThreshold: correct renderings of one design by three toolkits measured 0%,
	// and a redesign with a palette of its own 74–100% (experiments/redesign).
	DefaultPaletteThreshold = 15.0
	// DefaultDirectionThreshold: a thumb sliding across against one sliding up is 90°.
	DefaultDirectionThreshold = 30.0
	// DefaultLayoutThreshold: a part's box may be 10% of the component's width off.
	DefaultLayoutThreshold = 10.0
	// DefaultPixelThresholds were set from screenshots of correct and defective renderings
	// (docs/verification/pixel-measures.md): correct renderings stay under all four, and each
	// defect there goes over at least one.
	DefaultPixelThresholds = PixelThresholds{Area: 30, Strength: 28, Shift: 8, Spot: 2}
)

// PixelThresholds are how far two screenshots may differ, measure by measure, before the check
// prints a warning. Area, Strength and Spot are percentages; Shift is in pixels.
type PixelThresholds struct {
	Area     float64 `json:"area"`
	Strength float64 `json:"strength"`
	Shift    float64 `json:"shift"`
	Spot     float64 `json:"spot"`
}

// PixelMeasureNames lists the measures in the order they are shown.
var PixelMeasureNames = []string{"area", "strength", "shift", "spot"}

// Get returns one threshold by its name.
func (t PixelThresholds) Get(name string) float64 {
	switch name {
	case "area":
		return t.Area
	case "strength":
		return t.Strength
	case "shift":
		return t.Shift
	case "spot":
		return t.Spot
	}
	return 0
}

// ViewportsFor are the viewports a platform runs a contract at.
func (c *Contract) ViewportsFor(p Platform) []Viewport {
	if vs, _, ok := Pick(c.ViewportsBy, p); ok {
		return vs
	}
	return c.Viewports
}

// PartSame is what a part promises on a platform, and whether the contract said anything.
func (c *Contract) PartSame(part string, p Platform) (Aspects, bool) {
	if pt := c.PartByName(part); pt != nil {
		if a, _, ok := Pick(pt.Same, p); ok {
			return a, true
		}
	}
	return 0, false
}
