package contract

import (
	"bytes"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strconv"
	"strings"
	"unicode"

	"github.com/santhosh-tekuri/jsonschema/v6"
	"github.com/santhosh-tekuri/jsonschema/v6/kind"
	"golang.org/x/text/language"
	"golang.org/x/text/message"
	"gopkg.in/yaml.v3"

	"omnicui/schema"
)

// Problem is one error or warning about a contract file, with the line it is about.
type Problem struct {
	File    string `json:"file"`
	Line    int    `json:"line"`
	Message string `json:"message"`
	Warning bool   `json:"warning"`
}

func (p Problem) String() string {
	kind := "error"
	if p.Warning {
		kind = "warning"
	}
	return fmt.Sprintf("%s:%d  %s: %s", p.File, p.Line, kind, p.Message)
}

var compiled *jsonschema.Schema

func compiledSchema() *jsonschema.Schema {
	if compiled != nil {
		return compiled
	}
	doc, err := jsonschema.UnmarshalJSON(bytes.NewReader(schema.JSON))
	if err != nil {
		panic(err)
	}
	c := jsonschema.NewCompiler()
	if err := c.AddResource("contract.schema.json", doc); err != nil {
		panic(err)
	}
	compiled, err = c.Compile("contract.schema.json")
	if err != nil {
		panic(err)
	}
	return compiled
}

// LoadDir loads every *.contract.yaml in dir, in file-name order.
func LoadDir(dir string) ([]*Contract, []Problem, error) {
	files, err := filepath.Glob(filepath.Join(dir, "*.contract.yaml"))
	if err != nil {
		return nil, nil, err
	}
	sort.Strings(files)
	var all []*Contract
	var problems []Problem
	ids := map[string]string{}
	for _, f := range files {
		c, ps := LoadFile(f)
		problems = append(problems, ps...)
		if c == nil {
			continue
		}
		if prev, ok := ids[c.ID]; ok {
			problems = append(problems, Problem{File: rel(f), Line: 1, Message: fmt.Sprintf("id `%s` is already used by %s", c.ID, prev)})
			continue
		}
		ids[c.ID] = rel(f)
		all = append(all, c)
	}
	return all, problems, nil
}

func rel(path string) string {
	if wd, err := os.Getwd(); err == nil {
		if r, err := filepath.Rel(wd, path); err == nil && !strings.HasPrefix(r, "..") {
			return r
		}
	}
	return path
}

// HasErrors reports whether any problem is an error rather than a warning.
func HasErrors(ps []Problem) bool {
	for _, p := range ps {
		if !p.Warning {
			return true
		}
	}
	return false
}

// LoadFile reads, validates and checks one contract. It returns nil if there are errors.
func LoadFile(path string) (*Contract, []Problem) {
	file := rel(path)
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, []Problem{{File: file, Line: 1, Message: err.Error()}}
	}
	return Parse(file, data)
}

// Parse validates and checks one contract's YAML text.
func Parse(file string, data []byte) (*Contract, []Problem) {
	var doc yaml.Node
	if err := yaml.Unmarshal(data, &doc); err != nil {
		line := 1
		if te, ok := err.(*yaml.TypeError); ok && len(te.Errors) > 0 {
			fmt.Sscanf(te.Errors[0], "line %d", &line)
		}
		msg := err.Error()
		fmt.Sscanf(msg, "yaml: line %d", &line)
		return nil, []Problem{{File: file, Line: line, Message: "not valid YAML: " + strings.TrimPrefix(msg, "yaml: ")}}
	}
	if len(doc.Content) == 0 {
		return nil, []Problem{{File: file, Line: 1, Message: "the file is empty"}}
	}
	lines := map[string]int{}
	value, err := toValue(doc.Content[0], "", lines)
	if err != nil {
		return nil, []Problem{{File: file, Line: 1, Message: err.Error()}}
	}
	l := loader{file: file, lines: lines, doc: value}
	if verr := compiledSchema().Validate(value); verr != nil {
		ve, ok := verr.(*jsonschema.ValidationError)
		if !ok {
			return nil, []Problem{{File: file, Line: 1, Message: verr.Error()}}
		}
		l.schemaProblems(ve)
		l.sortProblems()
		return nil, l.problems
	}
	c := l.build(value.(map[string]any), doc.Content[0])
	l.crossCheck(c)
	l.sortProblems()
	if HasErrors(l.problems) {
		return nil, l.problems
	}
	return c, l.problems
}

// toValue turns a YAML node into JSON-like values (map[string]any, []any, json.Number, string,
// bool, nil), recording the line of every JSON pointer it passes.
func toValue(n *yaml.Node, ptr string, lines map[string]int) (any, error) {
	lines[ptr] = n.Line
	switch n.Kind {
	case yaml.MappingNode:
		m := map[string]any{}
		for i := 0; i+1 < len(n.Content); i += 2 {
			k := n.Content[i]
			if k.Kind != yaml.ScalarNode {
				return nil, fmt.Errorf("line %d: keys must be plain words", k.Line)
			}
			child := ptr + "/" + escape(k.Value)
			lines[child+"#key"] = k.Line
			v, err := toValue(n.Content[i+1], child, lines)
			if err != nil {
				return nil, err
			}
			m[k.Value] = v
		}
		return m, nil
	case yaml.SequenceNode:
		arr := make([]any, 0, len(n.Content))
		for i, c := range n.Content {
			v, err := toValue(c, ptr+"/"+strconv.Itoa(i), lines)
			if err != nil {
				return nil, err
			}
			arr = append(arr, v)
		}
		return arr, nil
	case yaml.AliasNode:
		return toValue(n.Alias, ptr, lines)
	case yaml.ScalarNode:
		switch n.Tag {
		case "!!int", "!!float":
			return json.Number(strings.ReplaceAll(n.Value, "_", "")), nil
		case "!!bool":
			var b bool
			if err := n.Decode(&b); err != nil {
				return nil, err
			}
			return b, nil
		case "!!null":
			return nil, nil
		default:
			return n.Value, nil
		}
	}
	return nil, fmt.Errorf("line %d: unsupported YAML", n.Line)
}

func escape(s string) string {
	return strings.ReplaceAll(strings.ReplaceAll(s, "~", "~0"), "/", "~1")
}

type loader struct {
	doc      any // the whole file, as plain values
	file     string
	lines    map[string]int
	problems []Problem
}

func (l *loader) sortProblems() {
	sort.SliceStable(l.problems, func(i, j int) bool { return l.problems[i].Line < l.problems[j].Line })
}

// value is the value at a JSON pointer whose segments need no unescaping, or nil.
func (l *loader) value(ptr string) any {
	v := l.doc
	for _, seg := range strings.Split(strings.TrimPrefix(ptr, "/"), "/") {
		m, ok := v.(map[string]any)
		if !ok {
			return nil
		}
		v = m[seg]
	}
	if f, ok := v.(float64); ok && f == float64(int64(f)) {
		return int64(f)
	}
	return v
}

// line finds the line of a JSON pointer, falling back to its nearest parent.
func (l *loader) line(ptr string) int {
	for {
		if n, ok := l.lines[ptr]; ok {
			return n
		}
		i := strings.LastIndex(ptr, "/")
		if i < 0 {
			return 1
		}
		ptr = ptr[:i]
	}
}

func (l *loader) errorf(ptr, format string, args ...any) {
	l.problems = append(l.problems, Problem{File: l.file, Line: l.line(ptr), Message: fmt.Sprintf(format, args...)})
}

func (l *loader) warnf(ptr, format string, args ...any) {
	l.problems = append(l.problems, Problem{File: l.file, Line: l.line(ptr), Message: fmt.Sprintf(format, args...), Warning: true})
}

var printer = message.NewPrinter(language.English)

// Words the schema knows, for "did you mean" on unknown fields.
var knownFields = []string{"id", "props", "events", "parts", "tests", "viewports", "colorSchemes", "thresholds",
	"type", "enum", "default", "role", "name", "steps", "expect", "screenshot", "trace", "record",
	"action", "part", "times", "key", "text", "dx", "dy", "event", "last", "point", "timing", "duration", "easing",
	"width", "height", "pixels", "area", "strength", "shift", "spot", "disabled", "checked", "value", "focused", "visible", "property", "opacity", "allCombinations", "keyboard", "motion", "mask", "optional"}

func (l *loader) schemaProblems(ve *jsonschema.ValidationError) {
	seen := map[string]bool{}
	var walk func(e *jsonschema.ValidationError)
	walk = func(e *jsonschema.ValidationError) {
		if len(e.Causes) > 0 {
			for _, c := range e.Causes {
				walk(c)
			}
			return
		}
		ptr := ""
		for _, p := range e.InstanceLocation {
			ptr += "/" + escape(p)
		}
		msg := e.ErrorKind.LocalizedString(printer)
		switch k := e.ErrorKind.(type) {
		case *kind.AdditionalProperties:
			for _, p := range k.Properties {
				m := fmt.Sprintf("unknown field `%s`", p)
				if s := closest(p, knownFields); s != "" {
					m += fmt.Sprintf("; did you mean `%s`?", s)
				}
				key := ptr + "/" + escape(p) + m
				if !seen[key] {
					seen[key] = true
					l.problems = append(l.problems, Problem{File: l.file, Line: l.line(ptr + "/" + escape(p) + "#key"), Message: m})
				}
			}
			return
		case *kind.Required:
			msg = fmt.Sprintf("missing field %s", quoteAll(k.Missing))
		case *kind.Type:
			msg = fmt.Sprintf("expected %s, got %s", strings.Join(k.Want, " or "), k.Got)
			// A threshold written as a bare number: say which unit it takes.
			if n := len(e.InstanceLocation); n == 2 && e.InstanceLocation[0] == "thresholds" && k.Got == "number" {
				got := fmt.Sprint(l.value(ptr))
				switch e.InstanceLocation[1] {
				case "area", "strength", "spot", "pixels":
					msg = patternMessage(got, "%$")
				case "shift":
					msg = patternMessage(got, "px$")
				case "timing", "motion":
					msg = patternMessage(got, "ms$")
				}
			}
		case *kind.Enum:
			msg = fmt.Sprintf("`%v` is not one of %s", k.Got, quoteAll(anyStrings(k.Want)))
		case *kind.Pattern:
			msg = patternMessage(k.Got, k.Want)
			// A key in the wrong form (propertyNames) is reported against no location: find the
			// key itself, for its line and where it is.
			if ptr == "" {
				for p := range l.lines {
					if strings.HasSuffix(p, "/"+escape(k.Got)+"#key") && (ptr == "" || l.lines[p] < l.lines[ptr]) {
						ptr = p
					}
				}
				if ptr != "" {
					e.InstanceLocation = strings.Split(strings.TrimPrefix(strings.TrimSuffix(ptr, "/"+escape(k.Got)+"#key"), "/"), "/")
				}
			}
		case *kind.FalseSchema:
			if len(e.InstanceLocation) > 0 {
				msg = fmt.Sprintf("unknown field `%s`", e.InstanceLocation[len(e.InstanceLocation)-1])
			}
		}
		where := strings.Join(e.InstanceLocation, ".")
		if where != "" {
			msg = where + ": " + msg
		}
		if !seen[ptr+msg] {
			seen[ptr+msg] = true
			l.problems = append(l.problems, Problem{File: l.file, Line: l.line(ptr), Message: msg})
		}
	}
	walk(ve)
}

func quoteAll(xs []string) string {
	q := make([]string, len(xs))
	for i, x := range xs {
		q[i] = "`" + x + "`"
	}
	return strings.Join(q, ", ")
}

func anyStrings(xs []any) []string {
	out := make([]string, len(xs))
	for i, x := range xs {
		out[i] = fmt.Sprint(x)
	}
	return out
}

// orderedKeys returns a mapping node's keys in file order.
func orderedKeys(n *yaml.Node) []string {
	var keys []string
	if n == nil || n.Kind != yaml.MappingNode {
		return nil
	}
	for i := 0; i+1 < len(n.Content); i += 2 {
		keys = append(keys, n.Content[i].Value)
	}
	return keys
}

func child(n *yaml.Node, key string) *yaml.Node {
	if n == nil || n.Kind != yaml.MappingNode {
		return nil
	}
	for i := 0; i+1 < len(n.Content); i += 2 {
		if n.Content[i].Value == key {
			return n.Content[i+1]
		}
	}
	return nil
}

func num(v any) float64 {
	switch x := v.(type) {
	case json.Number:
		f, _ := x.Float64()
		return f
	case float64:
		return x
	case int:
		return float64(x)
	}
	return 0
}

// plain turns json.Number into int64 or float64, recursively, so values compare and print simply.
func plain(v any) any {
	switch x := v.(type) {
	case json.Number:
		if i, err := x.Int64(); err == nil {
			return i
		}
		f, _ := x.Float64()
		return f
	case map[string]any:
		m := map[string]any{}
		for k, vv := range x {
			m[k] = plain(vv)
		}
		return m
	case []any:
		out := make([]any, len(x))
		for i, vv := range x {
			out[i] = plain(vv)
		}
		return out
	}
	return v
}

func parseMS(s string) float64 {
	f, _ := strconv.ParseFloat(strings.TrimSuffix(s, "ms"), 64)
	return f
}

func parsePX(s string) float64 {
	f, _ := strconv.ParseFloat(strings.TrimSuffix(s, "px"), 64)
	return f
}

// lintThresholds warns about a threshold far stricter than its default. Correct renderings by
// different toolkits always differ a little, so such a threshold makes every comparison warn,
// and real differences drown among the warnings.
func (l *loader) lintThresholds(c *Contract, given map[string]any) {
	d := DefaultPixelThresholds
	type th struct {
		name, unit, typical string
		got, def            float64
	}
	for _, t := range []th{
		{"area", "%", "correct renderings by different toolkits usually differ by 3–26% of the drawn area", c.Pixels.Area, d.Area},
		{"strength", "%", "anti-aliased edges alone usually reach 12–24%", c.Pixels.Strength, d.Strength},
		{"spot", "%", "correct renderings reach up to about 1.6%", c.Pixels.Spot, d.Spot},
		{"shift", "px", "glyph placement alone usually measures 3–7 px", c.Pixels.Shift, d.Shift},
		{"timing", "ms", "the same easing measured on two toolkits usually differs by 5–15 ms", c.TimingThreshold, DefaultTimingThreshold},
		{"motion", "ms", "the same clip recorded on two toolkits differed by up to 30 ms", c.MotionThreshold, DefaultMotionThreshold},
	} {
		ptr := "/thresholds/" + t.name
		if _, ok := given[t.name]; ok && t.got < t.def/3 {
			l.warnf(ptr, "thresholds.%s %g%s is far below the default %g%s; %s, so expect most comparisons to warn",
				t.name, t.got, t.unit, t.def, t.unit, t.typical)
		}
	}
}

func parsePercent(s string) float64 {
	f, _ := strconv.ParseFloat(strings.TrimSuffix(s, "%"), 64)
	return f
}

func (l *loader) build(v map[string]any, root *yaml.Node) *Contract {
	c := &Contract{ID: v["id"].(string), File: l.file}
	props, _ := v["props"].(map[string]any)
	for _, name := range orderedKeys(child(root, "props")) {
		c.Props = append(c.Props, field(name, props[name].(map[string]any)))
	}
	events, _ := v["events"].(map[string]any)
	for _, name := range orderedKeys(child(root, "events")) {
		ev := Event{Name: name, Fields: []Field{}}
		fields, _ := events[name].(map[string]any)
		for _, fname := range orderedKeys(child(child(root, "events"), name)) {
			ev.Fields = append(ev.Fields, field(fname, fields[fname].(map[string]any)))
		}
		c.Events = append(c.Events, ev)
	}
	parts := v["parts"].(map[string]any)
	for _, name := range orderedKeys(child(root, "parts")) {
		p := parts[name].(map[string]any)
		opt, _ := p["optional"].(bool)
		c.Parts = append(c.Parts, Part{Name: name, Role: p["role"].(string), ID: c.ID + "." + name, Optional: opt})
	}
	for i, tv := range v["tests"].([]any) {
		t := tv.(map[string]any)
		base := fmt.Sprintf("/tests/%d", i)
		test := Test{Name: t["name"].(string), Line: l.line(base)}
		if p, ok := t["props"].(map[string]any); ok {
			test.Props = plain(p).(map[string]any)
		}
		if steps, ok := t["steps"].([]any); ok {
			for j, sv := range steps {
				s := sv.(map[string]any)
				st := Step{Action: s["action"].(string), Part: s["part"].(string), Times: 1, Line: l.line(fmt.Sprintf("%s/steps/%d", base, j))}
				if x, ok := s["times"]; ok {
					st.Times = int(num(x))
				}
				if x, ok := s["key"].(string); ok {
					st.Key = x
				}
				if x, ok := s["text"].(string); ok {
					st.Text = x
				}
				if x, ok := s["dx"]; ok {
					st.DX = num(x)
				}
				if x, ok := s["dy"]; ok {
					st.DY = num(x)
				}
				test.Steps = append(test.Steps, st)
			}
		}
		if exps, ok := t["expect"].([]any); ok {
			expNodes := child(child(root, "tests").Content[i], "expect")
			for j, ev := range exps {
				e := ev.(map[string]any)
				ex := Expectation{Line: l.line(fmt.Sprintf("%s/expect/%d", base, j))}
				if name, ok := e["event"].(string); ok {
					ex.Event = name
					if x, ok := e["times"]; ok {
						n := int(num(x))
						ex.Times = &n
					}
					if last, ok := e["last"].(map[string]any); ok {
						ex.Last = plain(last).(map[string]any)
					}
				} else {
					ex.Part = e["part"].(string)
					for _, k := range orderedKeys(expNodes.Content[j]) {
						if k != "part" {
							ex.Checks = append(ex.Checks, Check{Field: k, Want: plain(e[k])})
						}
					}
				}
				test.Expect = append(test.Expect, ex)
			}
		}
		switch sv := t["screenshot"].(type) {
		case bool:
			test.Screenshot = sv
		case map[string]any:
			test.Screenshot = true
			test.ScreenshotPart, _ = sv["part"].(string)
			if ms, ok := sv["mask"].([]any); ok {
				for _, x := range ms {
					test.Mask = append(test.Mask, x.(string))
				}
			}
		}
		test.Record, _ = t["record"].(bool)
		test.AllCombinations, _ = t["allCombinations"].(bool)
		if traces, ok := t["trace"].([]any); ok {
			for j, trv := range traces {
				tr := trv.(map[string]any)
				spec := TraceSpec{Part: tr["part"].(string), Property: "position", Line: l.line(fmt.Sprintf("%s/trace/%d", base, j))}
				spec.Point, _ = tr["point"].(string)
				if prop, ok := tr["property"].(string); ok {
					spec.Property = prop
				}
				if spec.Property == "position" && spec.Point == "" {
					l.errorf(fmt.Sprintf("%s/trace/%d", base, j), "a position trace needs `point: center` or `point: topLeft`")
				}
				if tm, ok := tr["timing"].(map[string]any); ok {
					spec.Timing = &Timing{DurationMS: parseMS(tm["duration"].(string)), Easing: tm["easing"].(string)}
				}
				test.Trace = append(test.Trace, spec)
			}
		}
		c.Tests = append(c.Tests, test)
	}
	if vps, ok := v["viewports"].([]any); ok {
		for _, vp := range vps {
			m := vp.(map[string]any)
			vp := Viewport{Width: int(num(m["width"])), Height: int(num(m["height"]))}
			if props, ok := m["props"].(map[string]any); ok {
				vp.Props = plain(props).(map[string]any)
			}
			c.Viewports = append(c.Viewports, vp)
		}
	} else {
		c.Viewports = []Viewport{DefaultViewport}
	}
	c.Keyboard, _ = v["keyboard"].(bool)
	if ms, ok := v["mask"].([]any); ok {
		for _, x := range ms {
			c.Mask = append(c.Mask, x.(string))
		}
	}
	if cs, ok := v["colorSchemes"].([]any); ok {
		for _, s := range cs {
			c.ColorSchemes = append(c.ColorSchemes, s.(string))
		}
	} else {
		c.ColorSchemes = DefaultColorSchemes
	}
	c.Pixels, c.TimingThreshold, c.MotionThreshold = DefaultPixelThresholds, DefaultTimingThreshold, DefaultMotionThreshold
	if th, ok := v["thresholds"].(map[string]any); ok {
		if p, ok := th["pixels"].(string); ok {
			l.errorf("/thresholds/pixels#key", "thresholds.pixels was renamed to area (the share of the drawn area that differs): write `area: %s`", p)
		}
		percent := map[string]*float64{"area": &c.Pixels.Area, "strength": &c.Pixels.Strength, "spot": &c.Pixels.Spot}
		for name, dst := range percent {
			if p, ok := th[name].(string); ok {
				*dst = parsePercent(p)
			}
		}
		if s, ok := th["shift"].(string); ok {
			c.Pixels.Shift = parsePX(s)
		}
		if t, ok := th["timing"].(string); ok {
			c.TimingThreshold = parseMS(t)
		}
		if t, ok := th["motion"].(string); ok {
			c.MotionThreshold = parseMS(t)
		}
		l.lintThresholds(c, th)
	}
	return c
}

func field(name string, m map[string]any) Field {
	f := Field{Name: name, Type: m["type"].(string)}
	if e, ok := m["enum"].([]any); ok {
		f.Enum = plain(e).([]any)
	}
	if d, ok := m["default"]; ok {
		f.Default, f.HasDefault = plain(d), true
	}
	return f
}

// typeOK reports whether a value fits a contract type.
func typeOK(t string, v any) bool {
	switch t {
	case "string":
		_, ok := v.(string)
		return ok
	case "boolean":
		_, ok := v.(bool)
		return ok
	case "integer":
		_, ok := v.(int64)
		return ok
	case "number":
		switch v.(type) {
		case int64, float64:
			return true
		}
	}
	return false
}

func describe(v any) string {
	switch v.(type) {
	case string:
		return "a string"
	case bool:
		return "a boolean"
	case int64:
		return "an integer"
	case float64:
		return "a number"
	case nil:
		return "nothing"
	}
	return "an object"
}

func (l *loader) crossCheck(c *Contract) {
	partNames := []string{}
	for _, p := range c.Parts {
		partNames = append(partNames, p.Name)
	}
	eventNames := []string{}
	for _, e := range c.Events {
		eventNames = append(eventNames, e.Name)
	}
	propNames := []string{}
	for _, p := range c.Props {
		propNames = append(propNames, p.Name)
		if p.HasDefault && !typeOK(p.Type, p.Default) {
			l.errorf("/props/"+p.Name+"/default", "the default of `%s` is %s, but its type is %s", p.Name, describe(p.Default), p.Type)
		}
	}
	used := map[string]bool{}
	unknownPart := func(ptr, name string) bool {
		if c.PartByName(name) != nil {
			used[name] = true
			return false
		}
		m := fmt.Sprintf("part `%s` does not exist", name)
		if s := closest(name, partNames); s != "" {
			m += fmt.Sprintf("; did you mean `%s`?", s)
		}
		l.errorf(ptr, "%s", m)
		return true
	}
	names := map[string]bool{}
	for i, t := range c.Tests {
		base := fmt.Sprintf("/tests/%d", i)
		if names[t.Name] {
			l.errorf(base+"/name", "another test is already called `%s`", t.Name)
		}
		names[t.Name] = true
		for _, k := range sortedKeys(t.Props) {
			var decl *Field
			for j := range c.Props {
				if c.Props[j].Name == k {
					decl = &c.Props[j]
				}
			}
			if decl == nil {
				m := fmt.Sprintf("prop `%s` does not exist", k)
				if s := closest(k, propNames); s != "" {
					m += fmt.Sprintf("; did you mean `%s`?", s)
				}
				l.errorf(base+"/props/"+escape(k)+"#key", "%s", m)
			} else if !typeOK(decl.Type, t.Props[k]) {
				l.errorf(base+"/props/"+escape(k), "prop `%s` is %s, but its type is %s", k, describe(t.Props[k]), decl.Type)
			}
		}
		for j, s := range t.Steps {
			ptr := fmt.Sprintf("%s/steps/%d", base, j)
			if unknownPart(ptr+"/part", s.Part) {
				continue
			}
			role := c.PartByName(s.Part).Role
			if !Roles[role].Actions[s.Action] {
				l.errorf(ptr+"/action", "`%s` is a %s, which does not accept `%s`", s.Part, role, s.Action)
			}
		}
		for j, e := range t.Expect {
			ptr := fmt.Sprintf("%s/expect/%d", base, j)
			if e.Event != "" {
				ev := c.EventByName(e.Event)
				if ev == nil {
					m := fmt.Sprintf("event `%s` does not exist", e.Event)
					if s := closest(e.Event, eventNames); s != "" {
						m += fmt.Sprintf("; did you mean `%s`?", s)
					}
					l.errorf(ptr+"/event", "%s", m)
					continue
				}
				for _, k := range sortedKeys(e.Last) {
					var decl *Field
					fieldNames := []string{}
					for x := range ev.Fields {
						fieldNames = append(fieldNames, ev.Fields[x].Name)
						if ev.Fields[x].Name == k {
							decl = &ev.Fields[x]
						}
					}
					if decl == nil {
						m := fmt.Sprintf("event `%s` has no field `%s`", e.Event, k)
						if s := closest(k, fieldNames); s != "" {
							m += fmt.Sprintf("; did you mean `%s`?", s)
						}
						l.errorf(ptr+"/last/"+escape(k)+"#key", "%s", m)
					} else if !typeOK(decl.Type, e.Last[k]) {
						l.errorf(ptr+"/last/"+escape(k), "field `%s` of `%s` is %s, but its type is %s", k, e.Event, describe(e.Last[k]), decl.Type)
					}
				}
				continue
			}
			if unknownPart(ptr+"/part", e.Part) {
				continue
			}
			role := c.PartByName(e.Part).Role
			for _, ch := range e.Checks {
				t, ok := Roles[role].Fields[ch.Field]
				if !ok {
					l.errorf(ptr+"/"+ch.Field+"#key", "`%s` is a %s, which has no field `%s`", e.Part, role, ch.Field)
				} else if !typeOK(t, ch.Want) && !(t == "number" && typeOK("number", ch.Want)) {
					l.errorf(ptr+"/"+ch.Field, "`%s` is %s, but it is %s", ch.Field, t, describe(ch.Want))
				}
			}
		}
		for j, tr := range t.Trace {
			unknownPart(fmt.Sprintf("%s/trace/%d/part", base, j), tr.Part)
		}
		for j, m := range t.Mask {
			unknownPart(fmt.Sprintf("%s/screenshot/mask/%d", base, j), m)
		}
		if t.ScreenshotPart != "" {
			unknownPart(base+"/screenshot/part", t.ScreenshotPart)
			for _, m := range t.Mask {
				if m == t.ScreenshotPart {
					l.errorf(base+"/screenshot/mask", "test `%s` compares only `%s` and masks it too, which leaves nothing to compare", t.Name, m)
				}
			}
		}
		if t.Record && len(t.Steps) == 0 {
			l.warnf(base+"/record", "test `%s` records a clip but has no steps", t.Name)
		}
		if len(t.Trace) > 0 && len(t.Steps) == 0 {
			l.errorf(base+"/trace", "test `%s` traces a part but has no steps to trace", t.Name)
		}
	}
	for j, m := range c.Mask {
		unknownPart(fmt.Sprintf("/mask/%d", j), m)
	}
	if len(c.Mask) > 0 {
		shots := false
		for _, t := range c.Tests {
			shots = shots || t.Screenshot
		}
		if !shots {
			l.warnf("/mask", "`mask` leaves parts out of screenshot comparisons, and no test takes a screenshot")
		}
	}
	for i, vp := range c.Viewports {
		for _, k := range sortedKeys(vp.Props) {
			ptr := fmt.Sprintf("/viewports/%d/props/%s", i, escape(k))
			var decl *Field
			for j := range c.Props {
				if c.Props[j].Name == k {
					decl = &c.Props[j]
				}
			}
			if decl == nil {
				m := fmt.Sprintf("prop `%s` does not exist", k)
				if s := closest(k, propNames); s != "" {
					m += fmt.Sprintf("; did you mean `%s`?", s)
				}
				l.errorf(ptr+"#key", "%s", m)
			} else if !typeOK(decl.Type, vp.Props[k]) {
				l.errorf(ptr, "prop `%s` is %s, but its type is %s", k, describe(vp.Props[k]), decl.Type)
			}
		}
	}
	if c.Keyboard {
		found := false
		for _, p := range c.Parts {
			for _, key := range KeyboardKeys[p.Role] {
				found = true
				used[p.Name] = true
				c.Tests = append(c.Tests, Test{
					Name:     fmt.Sprintf("keyboard: %s on `%s` does what a press does", key, p.Name),
					Keyboard: &KeyboardCheck{Part: p.Name, Key: key},
					Line:     l.line("/keyboard"),
				})
			}
		}
		if !found {
			l.warnf("/keyboard", "`keyboard: true` adds tests for buttons, switches and checkboxes, and this contract has none")
		}
	}
	for _, p := range c.Parts {
		if !used[p.Name] {
			l.warnf("/parts/"+p.Name+"#key", "part `%s` is never used by any test", p.Name)
		}
	}
}

func sortedKeys(m map[string]any) []string {
	keys := make([]string, 0, len(m))
	for k := range m {
		keys = append(keys, k)
	}
	sort.Strings(keys)
	return keys
}

// closest returns the candidate nearest to word, if it is near enough to be a likely typo.
func closest(word string, candidates []string) string {
	best, bestD := "", 1<<30
	for _, c := range candidates {
		if c == word {
			continue
		}
		if d := distance(strings.ToLower(word), strings.ToLower(c)); d < bestD {
			best, bestD = c, d
		}
	}
	limit := 2
	if len(word) > 6 {
		limit = 3
	}
	if bestD <= limit {
		return best
	}
	return ""
}

// distance is the Damerau–Levenshtein (optimal string alignment) edit distance.
func distance(a, b string) int {
	ra, rb := []rune(a), []rune(b)
	d := make([][]int, len(ra)+1)
	for i := range d {
		d[i] = make([]int, len(rb)+1)
		d[i][0] = i
	}
	for j := range d[0] {
		d[0][j] = j
	}
	for i := 1; i <= len(ra); i++ {
		for j := 1; j <= len(rb); j++ {
			cost := 1
			if ra[i-1] == rb[j-1] {
				cost = 0
			}
			d[i][j] = min(d[i-1][j]+1, d[i][j-1]+1, d[i-1][j-1]+cost)
			if i > 1 && j > 1 && ra[i-1] == rb[j-2] && ra[i-2] == rb[j-1] {
				d[i][j] = min(d[i][j], d[i-2][j-2]+1)
			}
		}
	}
	return d[len(ra)][len(rb)]
}

// patternMessage says what a value in the wrong form should look like, and, where it can, how
// this one would be written.
func patternMessage(got, pattern string) string {
	switch {
	case pattern == "^[a-z][a-zA-Z0-9]*$":
		m := fmt.Sprintf("`%s` is not a valid name: names are camelCase, a lower-case letter and then letters and digits", got)
		if c := camelCase(got); c != "" && c != got {
			m += fmt.Sprintf("; write `%s`", c)
		}
		return m
	case strings.HasPrefix(pattern, "^[a-z][a-z0-9]*(\\.") || strings.HasPrefix(pattern, "^[a-z][a-z0-9]*(\\\\."):
		return fmt.Sprintf("`%s` is not a valid id: an id is lower-case words joined by dots, like `ui.counter`", got)
	case strings.HasSuffix(pattern, "%$"):
		return fmt.Sprintf("`%s` is not a percentage: write it like `30%%`", got)
	case strings.HasSuffix(pattern, "px$"):
		return fmt.Sprintf("`%s` is not a length in pixels: write it like `5px`", got)
	case strings.HasSuffix(pattern, "ms$"):
		return fmt.Sprintf("`%s` is not a duration: write it in milliseconds, like `200ms`", got)
	case strings.Contains(pattern, "cubic-bezier"):
		return fmt.Sprintf("`%s` is not a timing function: linear, ease, ease-in, ease-out, ease-in-out or cubic-bezier(x1, y1, x2, y2)", got)
	}
	return fmt.Sprintf("`%s` is not in the expected form (%s)", got, pattern)
}

// camelCase turns card-number, card_number or Card Number into cardNumber; "" if it can't.
func camelCase(s string) string {
	var b strings.Builder
	upper := false
	for _, r := range s {
		switch {
		case r == '-' || r == '_' || r == ' ' || r == '.':
			upper = b.Len() > 0
		case (r >= 'a' && r <= 'z') || (r >= 'A' && r <= 'Z') || (r >= '0' && r <= '9'):
			if b.Len() == 0 {
				if r >= '0' && r <= '9' {
					return ""
				}
				b.WriteRune(unicode.ToLower(r))
			} else if upper {
				b.WriteRune(unicode.ToUpper(r))
			} else {
				b.WriteRune(r)
			}
			upper = false
		default:
			return ""
		}
	}
	return b.String()
}
