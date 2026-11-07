// fakeadapter is a target adapter for omnicui's own tests. It implements the counter and the
// switch in plain Go, speaks the same JSON-RPC as a real adapter, and can be told to misbehave:
//
//	FAKE_CAN          constructs to advertise (default: all of them)
//	FAKE_CANNOT       capability=reason,… to advertise as missing
//	FAKE_COMPONENTS   components it has (default: ui.counter,ui.switch)
//	FAKE_COLOR        RRGGBB of the drawn component in screenshots (default 1d1f1b)
//	FAKE_SHIFT        pixels to move the drawn component right (default 0)
//	FAKE_EASING       linear or ease-out (default ease-out), FAKE_DURATION ms (default 200)
//	FAKE_BUG          no-name (the button has no name), double (each press adds two)
//	FAKE_CRASH_ON     exit with code 3 when request N arrives (after writing to stderr)
//	FAKE_CLOSE_ON     close standard output when request N arrives, and keep running
//	FAKE_HANG_ON      never answer request N
//	FAKE_STRAY_ON     print a stray line before answering request N
//	FAKE_WRONG_ID_ON  answer request N with the wrong id
//	FAKE_ERROR_ON     reply with an error to this method
//	FAKE_SLEEP_MS     sleep before every reply
//	FAKE_PID_FILE     write the process id here at start
//	FAKE_PROBE        how to draw the render check's picture: ok (default), small (half the
//	                  size asked for), blank (white only)
//	FAKE_HIDDEN       parts that report no size on screen (bounds 0×0), e.g. value
//	FAKE_ROLE         part=role,… to report another role than the contract's
//
// The settings come from the environment, or from a file of KEY=VALUE lines given as the first
// argument (so that no `env` command is needed, on any OS). As a stub generator:
// `fakeadapter --stubs <file>` copies standard input to <file>; `fakeadapter --fail-stubs` exits 4.
package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"image"
	"image/color"
	"image/png"
	"io"
	"math"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"time"
)

type req struct {
	ID     int64           `json:"id"`
	Method string          `json:"method"`
	Params json.RawMessage `json:"params"`
}

type params struct {
	Component   string         `json:"component"`
	Props       map[string]any `json:"props"`
	Viewport    struct{ Width, Height int }
	ColorScheme string           `json:"colorScheme"`
	Part        string           `json:"part"`
	Field       string           `json:"field"`
	Key         string           `json:"key"`
	Path        string           `json:"path"`
	Point       string           `json:"point"`
	Steps       []map[string]any `json:"steps"`
}

type state struct {
	component string
	w, h      int
	dark      bool
	count     int
	on        bool
	label     string
	focused   string
	events    []map[string]any
	moved     float64 // the switch thumb's offset, 0..20
}

var st state

func env(k, def string) string {
	if v := os.Getenv(k); v != "" {
		return v
	}
	return def
}

func envInt(k string) int {
	n, _ := strconv.Atoi(os.Getenv(k))
	return n
}

func has(list, item string) bool {
	for _, x := range strings.Split(list, ",") {
		if x == item {
			return true
		}
	}
	return false
}

type rpcErr struct {
	code int
	msg  string
}

func (e *rpcErr) Error() string { return e.msg }

func ease(t float64) float64 {
	if env("FAKE_EASING", "ease-out") == "linear" {
		return t
	}
	// cubic-bezier(0, 0, 0.58, 1)
	lo, hi := 0.0, 1.0
	cub := func(p1, p2, s float64) float64 { u := 1 - s; return 3*u*u*s*p1 + 3*u*s*s*p2 + s*s*s }
	for i := 0; i < 50; i++ {
		m := (lo + hi) / 2
		if cub(0, 0.58, m) < t {
			lo = m
		} else {
			hi = m
		}
	}
	return cub(0, 1, (lo+hi)/2)
}

func act(method string, p params) error {
	if !strings.HasPrefix(p.Part, st.component+".") {
		return &rpcErr{-32002, "no part " + p.Part}
	}
	part := strings.TrimPrefix(p.Part, st.component+".")
	known := map[string]bool{"ui.counter.value": true, "ui.counter.increment": true, "ui.switch.switch": true, "ui.switch.thumb": true}
	if !known[p.Part] {
		return &rpcErr{-32002, "no part " + p.Part}
	}
	press := func() {
		switch part {
		case "increment":
			st.count++
			if has(os.Getenv("FAKE_BUG"), "double") {
				st.count++
			}
			st.events = append(st.events, map[string]any{"name": "changed", "fields": map[string]any{"count": st.count}})
		case "switch":
			st.on = !st.on
			st.events = append(st.events, map[string]any{"name": "toggled", "fields": map[string]any{"on": st.on}})
		}
	}
	switch method {
	case "press":
		press()
		st.focused = part
	case "focus":
		st.focused = part
	case "key":
		if (p.Key == "Enter" || p.Key == "Space") && st.focused == part {
			press()
		}
	}
	if st.on {
		st.moved = 20
	} else {
		st.moved = 0
	}
	return nil
}

func screenshot(path string) error {
	img := image.NewRGBA(image.Rect(0, 0, st.w, st.h))
	bg := color.RGBA{0xfb, 0xfa, 0xf7, 0xff}
	if st.dark {
		bg = color.RGBA{0x1b, 0x1d, 0x19, 0xff}
	}
	v, _ := strconv.ParseUint(env("FAKE_COLOR", "1d1f1b"), 16, 32)
	fg := color.RGBA{uint8(v >> 16), uint8(v >> 8), uint8(v), 0xff}
	shift := envInt("FAKE_SHIFT")
	for y := 0; y < st.h; y++ {
		for x := 0; x < st.w; x++ {
			c := bg
			if x >= 24+shift && x < 84+shift && y >= 24 && y < 54 {
				c = fg
			}
			img.Set(x, y, c)
		}
	}
	os.MkdirAll(filepath.Dir(path), 0o755)
	f, err := os.Create(path)
	if err != nil {
		return err
	}
	defer f.Close()
	return png.Encode(f, img)
}

// probe draws the render check's picture: white, a black square at 8,8 and a red one 8 px from
// the bottom-right corner.
func probe(path string, w, h int) error {
	switch env("FAKE_PROBE", "ok") {
	case "small":
		w, h = w/2, h/2
	}
	img := image.NewRGBA(image.Rect(0, 0, w, h))
	for y := 0; y < h; y++ {
		for x := 0; x < w; x++ {
			c := color.RGBA{255, 255, 255, 255}
			if env("FAKE_PROBE", "ok") != "blank" {
				if x >= 8 && x < 40 && y >= 8 && y < 40 {
					c = color.RGBA{0, 0, 0, 255}
				}
				if x >= w-40 && x < w-8 && y >= h-40 && y < h-8 {
					c = color.RGBA{255, 0, 0, 255}
				}
			}
			img.Set(x, y, c)
		}
	}
	os.MkdirAll(filepath.Dir(path), 0o755)
	f, err := os.Create(path)
	if err != nil {
		return err
	}
	defer f.Close()
	return png.Encode(f, img)
}

func handle(n int, r req) (any, error) {
	var p params
	json.Unmarshal(r.Params, &p)
	if r.Method == os.Getenv("FAKE_ERROR_ON") {
		return nil, &rpcErr{-32003, "deliberate error on " + r.Method}
	}
	switch r.Method {
	case "advertise":
		can := strings.Split(env("FAKE_CAN", "mount,press,focus,key,type,drag,hover,read,events,screenshot,trace,record,probe,bounds,role,focused"), ",")
		cannot := map[string]string{}
		for _, kv := range strings.Split(os.Getenv("FAKE_CANNOT"), ",") {
			if k, v, ok := strings.Cut(kv, "="); ok {
				cannot[k] = v
				var keep []string
				for _, c := range can {
					if c != k {
						keep = append(keep, c)
					}
				}
				can = keep
			}
		}
		return map[string]any{"protocol": "1", "can": can, "cannot": cannot}, nil
	case "mount":
		if !has(env("FAKE_COMPONENTS", "ui.counter,ui.switch"), p.Component) {
			return nil, &rpcErr{-32001, "no component " + p.Component}
		}
		st = state{component: p.Component, w: p.Viewport.Width, h: p.Viewport.Height, dark: p.ColorScheme == "dark", label: "Dark mode"}
		if v, ok := p.Props["start"].(float64); ok {
			st.count = int(v)
		}
		if v, ok := p.Props["on"].(bool); ok {
			st.on = v
		}
		if v, ok := p.Props["label"].(string); ok {
			st.label = v
		}
		if st.on {
			st.moved = 20
		}
		return nil, nil
	case "press", "focus", "key", "type", "drag", "hover":
		return nil, act(r.Method, p)
	case "read":
		if err := act("read", p); err != nil {
			return nil, err
		}
		switch p.Part + "/" + p.Field {
		case "ui.counter.value/text":
			return strconv.Itoa(st.count), nil
		case "ui.counter.increment/name":
			if has(os.Getenv("FAKE_BUG"), "no-name") {
				return nil, nil
			}
			return "Increment", nil
		case "ui.counter.increment/text":
			return "+", nil
		case "ui.switch.switch/checked":
			return st.on, nil
		case "ui.switch.switch/name":
			return st.label, nil
		}
		part := strings.TrimPrefix(p.Part, st.component+".")
		switch p.Field {
		case "focused":
			return st.focused == part, nil
		case "role":
			for _, kv := range strings.Split(os.Getenv("FAKE_ROLE"), ",") {
				if k, v, ok := strings.Cut(kv, "="); ok && k == part {
					return v, nil
				}
			}
			return map[string]string{"value": "status", "increment": "button", "switch": "switch", "thumb": "generic"}[part], nil
		}
		return nil, nil
	case "bounds":
		if err := act("read", p); err != nil {
			return nil, err
		}
		if has(os.Getenv("FAKE_HIDDEN"), strings.TrimPrefix(p.Part, st.component+".")) {
			return map[string]any{"x": 24, "y": 24, "width": 0, "height": 0}, nil
		}
		shift := envInt("FAKE_SHIFT")
		return map[string]any{"x": 24 + shift, "y": 24, "width": 60, "height": 30}, nil
	case "probe":
		return p.Path, probe(p.Path, p.Viewport.Width, p.Viewport.Height)
	case "events":
		out := st.events
		st.events = nil
		if out == nil {
			out = []map[string]any{}
		}
		return out, nil
	case "screenshot":
		return p.Path, screenshot(p.Path)
	case "trace", "record":
		start := st.moved
		for _, s := range p.Steps {
			b, _ := json.Marshal(s)
			var sp params
			json.Unmarshal(b, &sp)
			if err := act(s["method"].(string), sp); err != nil {
				return nil, err
			}
		}
		end := st.moved
		dur, _ := strconv.ParseFloat(env("FAKE_DURATION", "200"), 64)
		var out []map[string]any
		for i := 0; i <= 30; i++ {
			ms := float64(i) * 1000 / 60
			t := math.Max(0, math.Min(1, (ms-50)/dur))
			x := 36 + start + (end-start)*ease(t)
			if r.Method == "trace" {
				out = append(out, map[string]any{"ms": ms, "x": x, "y": 36})
			} else {
				file := filepath.Join(p.Path, fmt.Sprintf("%d.png", i))
				if err := screenshot(file); err != nil {
					return nil, err
				}
				out = append(out, map[string]any{"ms": ms, "file": file})
			}
		}
		return out, nil
	}
	return nil, &rpcErr{-32601, "unknown method " + r.Method}
}

func main() {
	if len(os.Args) > 1 {
		switch os.Args[1] {
		case "--stubs":
			data, _ := io.ReadAll(os.Stdin)
			os.WriteFile(os.Args[2], data, 0o644)
			return
		case "--fail-stubs":
			fmt.Fprintln(os.Stderr, "no")
			os.Exit(4)
		default:
			if data, err := os.ReadFile(os.Args[1]); err == nil {
				for _, line := range strings.Split(string(data), "\n") {
					if k, v, ok := strings.Cut(strings.TrimSpace(line), "="); ok {
						os.Setenv(k, v)
					}
				}
			}
		}
	}
	if f := os.Getenv("FAKE_PID_FILE"); f != "" {
		os.WriteFile(f, []byte(strconv.Itoa(os.Getpid())), 0o644)
	}
	fmt.Fprintln(os.Stderr, "fake adapter starting")
	out := bufio.NewWriter(os.Stdout)
	sc := bufio.NewScanner(os.Stdin)
	n := 0
	for sc.Scan() {
		n++
		var r req
		if err := json.Unmarshal(sc.Bytes(), &r); err != nil {
			continue
		}
		switch n {
		case envInt("FAKE_CRASH_ON"):
			fmt.Fprintln(os.Stderr, "fatal: the fake adapter crashed on purpose during", r.Method)
			os.Exit(3)
		case envInt("FAKE_CLOSE_ON"):
			os.Stdout.Close()
			time.Sleep(time.Hour)
		case envInt("FAKE_HANG_ON"):
			time.Sleep(time.Hour)
		case envInt("FAKE_STRAY_ON"):
			fmt.Fprintln(out, "Loading theme… done")
		}
		if ms := envInt("FAKE_SLEEP_MS"); ms > 0 {
			time.Sleep(time.Duration(ms) * time.Millisecond)
		}
		result, err := handle(n, r)
		id := r.ID
		if n == envInt("FAKE_WRONG_ID_ON") {
			id += 100
		}
		reply := map[string]any{"jsonrpc": "2.0", "id": id}
		if err != nil {
			code := -32003
			if re, ok := err.(*rpcErr); ok {
				code = re.code
			}
			reply["error"] = map[string]any{"code": code, "message": err.Error()}
		} else {
			reply["result"] = result
		}
		b, _ := json.Marshal(reply)
		out.Write(append(b, '\n'))
		out.Flush()
	}
}
