// Package contract loads and checks contract files.
package contract

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
}

// Step is one user action. Times is already 1 or more.
type Step struct {
	Action string  `json:"action"`
	Part   string  `json:"part"`
	Key    string  `json:"key,omitempty"`
	Text   string  `json:"text,omitempty"`
	DX     float64 `json:"dx,omitempty"`
	DY     float64 `json:"dy,omitempty"`
	Times  int     `json:"times"`
	Line   int     `json:"-"`
}

// Check is one field of one part, as read after the steps.
type Check struct {
	Field string `json:"field"`
	Want  any    `json:"want"`
}

// Expectation is either about a part (Part and Checks) or about an event (Event, Times, Last).
type Expectation struct {
	Part   string         `json:"part,omitempty"`
	Checks []Check        `json:"checks,omitempty"`
	Event  string         `json:"event,omitempty"`
	Times  *int           `json:"times,omitempty"`
	Last   map[string]any `json:"last,omitempty"`
	Line   int            `json:"-"`
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
	Trace      []TraceSpec    `json:"trace,omitempty"`
	Record     bool           `json:"record,omitempty"`
	// AllCombinations takes the traces and the clip at every viewport × colour scheme, not only
	// the first.
	AllCombinations bool `json:"allCombinations,omitempty"`
	// Keyboard is set on the tests omnicui adds for `keyboard: true`: press Part with Key, and
	// expect what pressing it with the pointer does.
	Keyboard *KeyboardCheck `json:"keyboard,omitempty"`
	Line     int            `json:"-"`
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
	ID           string     `json:"id"`
	File         string     `json:"file"`
	Props        []Field    `json:"props"`
	Events       []Event    `json:"events"`
	Parts        []Part     `json:"parts"`
	Tests        []Test     `json:"tests"`
	Viewports    []Viewport `json:"viewports"`
	ColorSchemes []string   `json:"colorSchemes"`
	// Keyboard adds, for every button, switch and checkbox, a test per key that activates its
	// role: the key must do what a pointer press does, on every target.
	Keyboard bool `json:"keyboard,omitempty"`
	// Pixels are the screenshot thresholds; TimingThreshold is in milliseconds.
	Pixels          PixelThresholds `json:"pixelThresholds"`
	TimingThreshold float64         `json:"timingThreshold"`
	// MotionThreshold is for clips compared by visual progress, in ms. It is wider than timing's:
	// a clip shows everything that changes (a colour, a highlight), each target's frames come on
	// its own clock, and correct renderings measured up to 20 ms apart.
	MotionThreshold float64 `json:"motionThreshold"`
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
