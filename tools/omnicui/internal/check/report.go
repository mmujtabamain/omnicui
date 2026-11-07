package check

import (
	"encoding/json"
	"fmt"
	"html/template"
	"math"
	"os"
	"path/filepath"
	"strings"

	"omnicui/internal/compare"
	"omnicui/internal/contract"
)

type reportTest struct {
	Contract    *contract.Contract
	Test        contract.Test
	Anchor      string
	Results     []TestResult
	Combos      []Combo
	Pixels      []Comparison
	Timing      []Comparison
	Visibility  []Comparison
	ClipSets    []clipSet
	Chart       template.HTML
	HasShots    bool
	HasClips    bool
	TimingLines []string
}

// clipSet is the clips of one test at one combination: their chart, the player's data, and
// each target's animated PNG.
type clipSet struct {
	Label   string
	Chart   template.HTML
	Player  string // JSON for the player script
	Targets []clipTarget
}

type clipTarget struct {
	Name, File, First string
}

type playerTarget struct {
	Name   string   `json:"name"`
	Offset float64  `json:"offset"`
	Frames [][2]any `json:"frames"`
	Align  string   `json:"align"`
}

// clipSets builds, for each combination where any target recorded, the player's data. Each
// target's timeline is lined up at the moment its first input landed, when the target says when
// that was, and otherwise where its picture starts to change.
func clipSets(c *contract.Contract, results []TestResult) []clipSet {
	var out []clipSet
	for _, cb := range Combos(c) {
		key := cb.Key()
		set := clipSet{Label: cb.Label(), Chart: clipChart(results, key)}
		var targets []playerTarget
		lo, hi := 0.0, 0.0
		for _, res := range results {
			clip, ok := res.Clips[key]
			if !ok || len(clip.Frames) == 0 {
				continue
			}
			pt := playerTarget{Name: res.Target, Offset: clip.Frames[0].MS, Align: "its first frame"}
			if len(clip.Inputs) > 0 {
				pt.Offset, pt.Align = clip.Inputs[0], "its first input"
			} else if len(clip.Values) == len(clip.Frames) {
				for i, v := range clip.Values {
					if v >= 0.02 {
						pt.Offset, pt.Align = clip.Frames[i].MS, "where it starts to change"
						break
					}
				}
			}
			for _, f := range clip.Frames {
				pt.Frames = append(pt.Frames, [2]any{f.MS, filepath.ToSlash(f.File)})
				lo, hi = math.Min(lo, f.MS-pt.Offset), math.Max(hi, f.MS-pt.Offset)
			}
			targets = append(targets, pt)
			set.Targets = append(set.Targets, clipTarget{Name: res.Target, File: filepath.ToSlash(clip.File), First: filepath.ToSlash(clip.Frames[0].File)})
		}
		if len(targets) == 0 {
			continue
		}
		data, _ := json.Marshal(map[string]any{"targets": targets, "min": math.Floor(lo), "max": math.Ceil(hi)})
		set.Player = string(data)
		out = append(out, set)
	}
	return out
}

type reportData struct {
	Rep     *Report
	Targets []string
	Tests   []reportTest
}

var dashes = []string{"", "6 4", "2 3", "10 3 2 3"}
var strokes = []string{"#1f5fbf", "#b3261e", "#1d7a3a", "#6b3fa0"}

type chartLine struct {
	name string
	pts  [][2]float64
}

// progressOf is how far each sample has gone from the first to the last: along the movement for
// a position, along the change for a value.
func progressOf(s []compare.Sample) []float64 {
	out := make([]float64, len(s))
	p0, pn := s[0], s[len(s)-1]
	if p0.Value != nil && pn.Value != nil {
		change := *pn.Value - *p0.Value
		for i, smp := range s {
			if smp.Value != nil && change != 0 {
				out[i] = (*smp.Value - *p0.Value) / change
			}
		}
		return out
	}
	dx, dy := pn.X-p0.X, pn.Y-p0.Y
	d2 := dx*dx + dy*dy
	for i, smp := range s {
		if d2 > 0 {
			out[i] = ((smp.X-p0.X)*dx + (smp.Y-p0.Y)*dy) / d2
		}
	}
	return out
}

// startAligned shifts a progress series so that 0 ms is where it passes 2%: each target reacts
// after its own delay, and the chart compares the shape of the change.
func startAligned(ms, progress []float64) ([][2]float64, float64) {
	t0 := -1.0
	for i, p := range progress {
		if p >= 0.02 {
			t0 = ms[i]
			break
		}
	}
	if t0 < 0 {
		return nil, 0
	}
	var pts [][2]float64
	maxT := 0.0
	for i := range ms {
		t := ms[i] - t0
		if t < -20 {
			continue
		}
		pts = append(pts, [2]float64{t, progress[i]})
		maxT = math.Max(maxT, t)
	}
	return pts, maxT
}

// chart draws each target's traced progress of one part over time as an SVG line chart.
func chart(test contract.Test, results []TestResult, spec contract.TraceSpec, first string) template.HTML {
	var lines []chartLine
	maxT := 1.0
	for _, res := range results {
		for _, tr := range res.Traces {
			if tr.Part != spec.Part || tr.Point != spec.Point || (tr.Combo != "" && tr.Combo != first) || len(tr.Samples) < 2 || !tr.Curve.Moved {
				continue
			}
			ms := make([]float64, len(tr.Samples))
			for i, smp := range tr.Samples {
				ms[i] = smp.MS
			}
			pts, m := startAligned(ms, progressOf(tr.Samples))
			if pts == nil {
				continue
			}
			maxT = math.Max(maxT, m)
			lines = append(lines, chartLine{name: res.Target, pts: pts})
		}
	}
	if spec.Timing != nil {
		want, err := compare.Expected(spec.Timing.DurationMS, spec.Timing.Easing)
		if err == nil {
			pts := [][2]float64{{0, 0.02}}
			for i, l := range compare.Levels {
				pts = append(pts, [2]float64{want.Times[i], l})
			}
			pts = append(pts, [2]float64{want.Duration, 0.98})
			lines = append(lines, chartLine{name: fmt.Sprintf("%.0fms %s", spec.Timing.DurationMS, spec.Timing.Easing), pts: pts})
			maxT = math.Max(maxT, want.Duration)
		}
	}
	return drawChart(lines, maxT, "Progress of "+traceLabel(spec)+" over time, aligned where each target's change starts")
}

// clipChart draws each target's clip's visual progress at one combination.
func clipChart(results []TestResult, combo string) template.HTML {
	var lines []chartLine
	maxT := 1.0
	for _, res := range results {
		clip, ok := res.Clips[combo]
		if !ok || clip.Progress == nil || len(clip.Values) != len(clip.Frames) {
			continue
		}
		ms := make([]float64, len(clip.Frames))
		for i, f := range clip.Frames {
			ms[i] = f.MS
		}
		pts, m := startAligned(ms, clip.Values)
		if pts == nil {
			continue
		}
		maxT = math.Max(maxT, m)
		lines = append(lines, chartLine{name: res.Target, pts: pts})
	}
	return drawChart(lines, maxT, "Visual progress of each target's clip at "+combo+" (how far the picture has gone from its first frame to its last), aligned where each starts to change")
}

func drawChart(lines []chartLine, maxT float64, what string) template.HTML {
	if len(lines) == 0 {
		return ""
	}
	maxT = math.Min(maxT, 2000)
	const W, H, L, B = 560.0, 240.0, 48.0, 32.0
	x := func(t float64) float64 { return L + (t+20)/(maxT+20)*(W-L-16) }
	y := func(p float64) float64 { return H - B - math.Max(-0.1, math.Min(1.1, p))*(H-B-16) }
	var sb strings.Builder
	var names []string
	for _, l := range lines {
		names = append(names, l.name)
	}
	label := what + ": " + strings.Join(names, ", ") + "."
	fmt.Fprintf(&sb, `<svg class="chart" viewBox="0 0 %.0f %.0f" role="img" aria-label="%s">`, W, H, template.HTMLEscapeString(label))
	fmt.Fprintf(&sb, `<line class="axis" x1="%.1f" y1="%.1f" x2="%.1f" y2="%.1f"/>`, L, H-B, W-16, H-B)
	fmt.Fprintf(&sb, `<line class="axis" x1="%.1f" y1="16" x2="%.1f" y2="%.1f"/>`, L, L, H-B)
	for _, p := range []float64{0, 0.5, 1} {
		fmt.Fprintf(&sb, `<text x="%.1f" y="%.1f" text-anchor="end">%d%%</text>`, L-6, y(p)+4, int(p*100))
	}
	step := 50.0
	if maxT > 400 {
		step = 100
	}
	if maxT > 1000 {
		step = 250
	}
	for t := 0.0; t <= maxT; t += step {
		fmt.Fprintf(&sb, `<text x="%.1f" y="%.1f" text-anchor="middle">%.0f</text>`, x(t), H-B+16, t)
	}
	fmt.Fprintf(&sb, `<text x="%.1f" y="%.1f" text-anchor="end">ms</text>`, W-16, H-4)
	for i, l := range lines {
		var pts []string
		for _, pt := range l.pts {
			if pt[0] > maxT {
				continue
			}
			pts = append(pts, fmt.Sprintf("%.1f,%.1f", x(pt[0]), y(pt[1])))
		}
		dash := ""
		if d := dashes[i%len(dashes)]; d != "" {
			dash = fmt.Sprintf(` stroke-dasharray="%s"`, d)
		}
		fmt.Fprintf(&sb, `<polyline fill="none" stroke="%s" stroke-width="2"%s points="%s"/>`, strokes[i%len(strokes)], dash, strings.Join(pts, " "))
		ly := 20.0 + float64(i)*16
		fmt.Fprintf(&sb, `<line x1="%.1f" y1="%.1f" x2="%.1f" y2="%.1f" stroke="%s" stroke-width="2"%s/>`, W-200, ly, W-176, ly, strokes[i%len(strokes)], dash)
		fmt.Fprintf(&sb, `<text x="%.1f" y="%.1f">%s</text>`, W-170, ly+4, template.HTMLEscapeString(l.name))
	}
	sb.WriteString(`</svg>`)
	return template.HTML(sb.String())
}

func writeReport(path string, rep *Report) error {
	data := reportData{Rep: rep}
	for _, t := range rep.Targets {
		data.Targets = append(data.Targets, t.Name)
	}
	for _, c := range rep.Contracts {
		for _, test := range c.Tests {
			rt := reportTest{Contract: c, Test: test, Anchor: c.Short() + "-" + slug(test.Name), Combos: Combos(c)}
			for _, res := range rep.Results {
				if res.Contract == c.ID && res.Test == test.Name {
					rt.Results = append(rt.Results, res)
					rt.HasShots = rt.HasShots || len(res.Screenshots) > 0
					rt.HasClips = rt.HasClips || len(res.Clips) > 0
				}
			}
			for _, cmp := range rep.Comparisons {
				if cmp.Contract != c.ID || cmp.Test != test.Name {
					continue
				}
				switch cmp.Kind {
				case "pixels":
					rt.Pixels = append(rt.Pixels, cmp)
				case "visibility":
					rt.Visibility = append(rt.Visibility, cmp)
				default:
					rt.Timing = append(rt.Timing, cmp)
				}
			}
			first := ""
			if cbs := Combos(c); len(cbs) > 0 {
				first = cbs[0].Key()
			}
			for _, spec := range test.Trace {
				rt.Chart += chart(test, rt.Results, spec, first)
			}
			if rt.HasClips {
				rt.ClipSets = clipSets(c, rt.Results)
			}
			data.Tests = append(data.Tests, rt)
		}
	}
	funcs := template.FuncMap{
		"shot":     func(res TestResult, key string) string { return res.Screenshots[key] },
		"pair":     pairLabel,
		"measures": func() []string { return contract.PixelMeasureNames },
		"measure": func(c Comparison, name string) string {
			if c.Measures == nil {
				return "–"
			}
			return MeasureText(name, MeasureValue(*c.Measures, name))
		},
		"over": func(c Comparison, name string) bool {
			for _, o := range c.Over {
				if o == name {
					return true
				}
			}
			return false
		},
		"threshold": func(c Comparison, name string) string {
			if c.Thresholds == nil {
				return ""
			}
			return ThresholdText(name, c.Thresholds.Get(name))
		},
		"spot": func(c Comparison) string {
			if c.Measures == nil || len(c.Measures.Spots) == 0 {
				return ""
			}
			return c.Measures.Spots[0].String()
		},
		"value":  valueText,
		"lower":  strings.ToLower,
		"cls":    func(s Status) string { return strings.ReplaceAll(string(s), " ", "-") },
		"join":   strings.Join,
		"unitOf": unit,
		"missing": func(res TestResult) []string {
			var out []string
			for _, k := range sortedKeys(res.Missing) {
				out = append(out, fmt.Sprintf("%s not advertised (%s)", k, res.Missing[k]))
			}
			return out
		},
	}
	tpl, err := template.New("report").Funcs(funcs).Parse(reportHTML)
	if err != nil {
		return err
	}
	f, err := os.Create(path)
	if err != nil {
		return err
	}
	defer f.Close()
	return tpl.Execute(f, data)
}

const reportHTML = `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>omnicui check · {{.Rep.Summary.Pass}} pass, {{.Rep.Summary.Fail}} fail, {{.Rep.Summary.Warnings}} warnings</title>
<style>
:root { color-scheme: light dark; --bg: #fbfaf7; --fg: #1d1f1b; --muted: #4f5449; --line: #c9c6bd; --card: #ffffff;
  --pass: #1d6b34; --fail: #a3201a; --warn: #7a4b00; --na: #4f5449; }
@media (prefers-color-scheme: dark) { :root { --bg: #161814; --fg: #ecebe6; --muted: #b9b8b0; --line: #4a4c45; --card: #20231e;
  --pass: #7fd197; --fail: #ff9d93; --warn: #f2c36b; --na: #b9b8b0; } }
* { box-sizing: border-box; }
body { margin: 0; background: var(--bg); color: var(--fg); font: 16px/1.5 system-ui, -apple-system, "Segoe UI", sans-serif; }
a { color: inherit; }
header, main { max-width: 72rem; margin: 0 auto; padding: 1rem 1.25rem; }
h1 { font-size: 1.6rem; margin: .5rem 0; }
h2 { font-size: 1.3rem; margin: 2rem 0 .5rem; border-top: 1px solid var(--line); padding-top: 1rem; }
h3 { font-size: 1.1rem; margin: 1.5rem 0 .5rem; }
.summary { display: flex; flex-wrap: wrap; gap: .5rem 1.25rem; padding: 0; list-style: none; }
.summary li { border: 1px solid var(--line); border-radius: .4rem; padding: .25rem .6rem; background: var(--card); }
.muted { color: var(--muted); }
.table { overflow-x: auto; margin: .5rem 0; } .table:focus-visible { outline: 2px solid var(--fg); outline-offset: 2px; }
table { border-collapse: collapse; width: 100%; background: var(--card); }
caption { text-align: left; font-weight: 600; padding: .25rem 0; }
th, td { border: 1px solid var(--line); padding: .35rem .5rem; text-align: left; vertical-align: top; }
.st { font-weight: 700; white-space: nowrap; }
.th { font-size: 0.85em; }
.st-pass { color: var(--pass); } .st-fail, .st-broken { color: var(--fail); } .st-warning { color: var(--warn); } .st-not-advertised { color: var(--na); }
.player .controls { display: flex; flex-wrap: wrap; gap: .75rem; align-items: center; }
.player .time { width: min(28rem, 70vw); }
.shots { display: flex; flex-wrap: wrap; gap: .75rem; margin: .5rem 0; padding: 0; list-style: none; }
figure { margin: 0; border: 1px solid var(--line); border-radius: .4rem; padding: .4rem; background: var(--card); }
figure img { display: block; max-width: 100%; height: auto; border: 1px solid var(--line); }
figcaption { font-size: .9rem; color: var(--muted); margin-top: .25rem; overflow-wrap: anywhere; }
td, th, p, div { overflow-wrap: anywhere; }
.chart { width: 100%; max-width: 36rem; height: auto; background: var(--card); border: 1px solid var(--line); border-radius: .4rem; }
.chart text { fill: var(--fg); font-size: 11px; }
.chart .axis { stroke: var(--muted); stroke-width: 1; }
@media (forced-colors: active) { .chart polyline, .chart line { stroke: CanvasText; } .chart text { fill: CanvasText; } }
pre { white-space: pre-wrap; background: var(--card); border: 1px solid var(--line); padding: .5rem; border-radius: .4rem; font-size: .85rem; }
.skip { position: absolute; left: -10000px; } .skip:focus { left: 1rem; top: 1rem; background: var(--card); padding: .5rem; }
@media (max-width: 40rem) { th, td { padding: .3rem; font-size: .9rem; } }
</style>
</head>
<body>
<a class="skip" href="#main">Skip to the results</a>
<header>
<h1>omnicui check</h1>
<p class="muted">Run {{.Rep.RunID}} · {{len .Rep.Contracts}} contracts · targets: {{join .Targets ", "}}</p>
<ul class="summary" aria-label="Summary">
<li><strong>{{.Rep.Summary.Pass}}</strong> pass</li>
<li><strong>{{.Rep.Summary.Fail}}</strong> fail</li>
<li><strong>{{.Rep.Summary.Broken}}</strong> broken</li>
<li><strong>{{.Rep.Summary.NotAdvertised}}</strong> not advertised</li>
<li><strong>{{.Rep.Summary.Warnings}}</strong> warnings</li>
</ul>
{{range .Rep.Required}}<p><strong>Required:</strong> {{.}}</p>{{end}}
{{range .Rep.Targets}}{{if .Broken}}<p><strong>{{.Name}} is broken:</strong> {{.Broken}}</p>{{if .Stderr}}<pre aria-label="Last lines of {{.Name}}'s standard error">{{join .Stderr "\n"}}</pre>{{end}}{{end}}{{end}}
</header>
<main id="main">
<section aria-labelledby="targets-h">
<h2 id="targets-h">Targets</h2>
<div class="table" role="region" tabindex="0" aria-label="What each target advertised">
<table>
<caption>What each target advertised</caption>
<thead><tr><th scope="col">Target</th><th scope="col">Can</th><th scope="col">Cannot</th></tr></thead>
<tbody>
{{range .Rep.Targets}}<tr><th scope="row">{{.Name}}</th><td>{{join .Can ", "}}</td><td>{{range $k, $v := .Cannot}}{{$k}} ({{$v}}) {{end}}</td></tr>
{{end}}</tbody>
</table>
</div>
</section>
{{$targets := .Targets}}
{{range .Tests}}
<section aria-labelledby="{{.Anchor}}">
<h2 id="{{.Anchor}}">{{.Contract.ID}}: {{.Test.Name}}</h2>
<div class="table" role="region" tabindex="0" aria-label="Result on each target">
<table>
<caption>Result on each target</caption>
<thead><tr><th scope="col">Target</th><th scope="col">Status</th><th scope="col">Details</th></tr></thead>
<tbody>
{{range .Results}}<tr><th scope="row">{{.Target}}</th><td class="st st-{{cls .Status}}">{{.Status}}</td><td>{{range .Messages}}<div>{{.}}</div>{{end}}{{range missing .}}<div class="muted">{{.}}</div>{{end}}</td></tr>
{{end}}</tbody>
</table>
</div>
{{if .HasShots}}
<h3>Screenshots</h3>
{{$t := .}}
{{range $cb := .Combos}}
<ul class="shots" aria-label="Screenshots at {{$cb.Label}}">
{{range $t.Results}}{{$res := .}}{{with shot $res $cb.Key}}<li><figure><img src="{{.}}" alt="The component on {{$res.Target}} at {{$cb.Label}}" width="{{$cb.Viewport.Width}}" height="{{$cb.Viewport.Height}}"><figcaption>{{$res.Target}} · {{$cb.Label}}</figcaption></figure></li>{{end}}{{end}}
</ul>
{{end}}
{{if .Pixels}}
<div class="table" role="region" tabindex="0" aria-label="Differences between targets (worst over all viewports and colour schemes)">
<table>
<caption>Differences between targets (worst viewport and colour scheme per pair). Area: how much of the drawn area differs. Strength: how far off in colour. Shift: how far pixels moved. Spot: how concentrated the difference is in its worst 24×24 px spot. ▲ marks a measure above its threshold.</caption>
<thead><tr><th scope="col">Pair</th>{{range measures}}<th scope="col">{{.}}</th>{{end}}<th scope="col">Result</th><th scope="col">Where</th></tr></thead>
<tbody>
{{range $c := .Pixels}}<tr><th scope="row">{{pair $c}}</th>{{range $n := measures}}<td{{if over $c $n}} class="st-warning"{{end}}>{{measure $c $n}}{{if over $c $n}} ▲ <span class="th">above {{threshold $c $n}}</span>{{end}}</td>{{end}}<td class="st {{if $c.Warning}}st-warning{{else}}st-pass{{end}}">{{if $c.Warning}}warning{{else}}pass{{end}}</td><td><figure><img src="{{$c.Diff}}" alt="Differences between {{$c.A}} and {{$c.B}} at {{$c.Combo}}: strongly differing pixels red, slightly differing orange, the worst spots boxed in blue" loading="lazy"><figcaption>{{$c.Combo}}{{with spot $c}}; worst spot {{.}}{{end}}{{with $c.SpotPart}}, in {{.}}{{end}}{{if $c.Note}}: {{$c.Note}}{{end}}</figcaption></figure></td></tr>
{{end}}</tbody>
{{with $first := index .Pixels 0}}<tfoot><tr><th scope="row">thresholds</th>{{range $n := measures}}<td>{{threshold $first $n}}</td>{{end}}<td></td><td></td></tr></tfoot>{{end}}
</table>
</div>
{{end}}
{{end}}
{{if .Timing}}
<h3>Motion</h3>
{{.Chart}}
<div class="table" role="region" tabindex="0" aria-label="Traced timing: the largest difference at any level of the movement">
<table>
<caption>Traced timing: the largest difference at any level of the movement</caption>
<thead><tr><th scope="col">Part</th><th scope="col">Compared</th><th scope="col">Difference</th><th scope="col">Result</th><th scope="col">Note</th></tr></thead>
<tbody>
{{range .Timing}}<tr><td>{{.Part}}</td><th scope="row">{{pair .}}</th><td>{{value .}} (threshold {{.Threshold}} ms)</td><td class="st {{if .Warning}}st-warning{{else}}st-pass{{end}}">{{if .Warning}}warning{{else}}pass{{end}}</td><td>{{.Note}}</td></tr>
{{end}}</tbody>
</table>
</div>
{{end}}
{{if .Visibility}}
<h3>Visibility</h3>
<ul>{{range .Visibility}}<li class="st-warning">{{.A}}: {{.Note}}</li>{{end}}</ul>
{{end}}
{{if .HasClips}}
<h3>Recorded clips</h3>
{{range .ClipSets}}
<h4>{{.Label}}</h4>
{{.Chart}}
<div class="player" data-player="{{.Player}}">
<ul class="shots">{{range .Targets}}<li><figure><img src="{{.First}}" alt="{{.Name}}, at the moment the slider shows"><figcaption>{{.Name}}</figcaption></figure></li>{{end}}</ul>
<p class="controls"><button type="button" class="play">Play</button>
<label>Speed <select class="speed"><option value="1">1×</option><option value="0.5">½×</option><option value="0.25" selected>¼×</option></select></label>
<label>Time <input type="range" class="time" step="1"></label> <output class="at">0 ms</output></p>
<p class="note">Every clip at the same moment, lined up at its first input (or, if the target doesn't say when that was, where its picture starts to change).</p>
</div>
<details><summary>Each clip on its own (animated PNG)</summary>
<ul class="shots">{{range .Targets}}{{if .File}}<li><figure><picture><source media="(prefers-reduced-motion: reduce)" srcset="{{.First}}"><img src="{{.File}}" alt="Clip of the test running on {{.Name}}"></picture><figcaption>{{.Name}} · {{.File}}</figcaption></figure></li>{{end}}{{end}}</ul>
</details>
{{end}}
{{end}}
</section>
{{end}}
</main>
<script>
// The clip player: shows every target's frame for the same moment, lined up per target.
for (const p of document.querySelectorAll(".player")) {
  const d = JSON.parse(p.dataset.player);
  const imgs = p.querySelectorAll("img"), time = p.querySelector(".time"), at = p.querySelector(".at");
  const play = p.querySelector(".play"), speed = p.querySelector(".speed");
  time.min = d.min; time.max = d.max; time.value = d.min;
  const show = (t) => {
    d.targets.forEach((tg, i) => {
      let src = tg.frames[0][1];
      for (const [ms, f] of tg.frames) { if (ms - tg.offset <= t) src = f; else break; }
      if (imgs[i].getAttribute("src") !== src) imgs[i].src = src;
    });
    time.value = t; at.value = Math.round(t) + " ms";
  };
  time.addEventListener("input", () => show(+time.value));
  let raf = 0;
  play.addEventListener("click", () => {
    if (raf) { cancelAnimationFrame(raf); raf = 0; play.textContent = "Play"; return; }
    let t = +time.value >= d.max ? d.min : +time.value, last = performance.now();
    play.textContent = "Pause";
    const tick = (now) => {
      t += (now - last) * +speed.value; last = now;
      if (t >= d.max) { show(d.max); raf = 0; play.textContent = "Play"; return; }
      show(t); raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
  });
  show(d.min);
}
</script>
</body>
</html>
`
