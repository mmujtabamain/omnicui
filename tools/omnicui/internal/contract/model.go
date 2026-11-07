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

// Timing is a named timing function a trace is checked against.
type Timing struct {
	DurationMS float64 `json:"durationMs"`
	Easing     string  `json:"easing"`
}

// TraceSpec follows one point on a part while the steps run.
type TraceSpec struct {
	Part   string  `json:"part"`
	Point  string  `json:"point"`
	Timing *Timing `json:"timing,omitempty"`
	Line   int     `json:"-"`
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
	Line       int            `json:"-"`
}

// Viewport is a window size.
type Viewport struct {
	Width  int `json:"width"`
	Height int `json:"height"`
}

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
	// PixelThreshold is a percentage of pixels; TimingThreshold is in milliseconds.
	PixelThreshold  float64 `json:"pixelThreshold"`
	TimingThreshold float64 `json:"timingThreshold"`
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
	DefaultPixelThreshold  = 30.0
	DefaultTimingThreshold = 20.0
)
