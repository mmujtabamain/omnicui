package check

import (
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
	Chart       template.HTML
	HasShots    bool
	HasClips    bool
	PixelLine   string
	TimingLines []string
}

type reportData struct {
	Rep     *Report
	Targets []string
	Tests   []reportTest
}

var dashes = []string{"", "6 4", "2 3", "10 3 2 3"}
var strokes = []string{"#1f5fbf", "#b3261e", "#1d7a3a", "#6b3fa0"}

// chart draws each target's traced progress over time as an SVG line chart.
func chart(test contract.Test, results []TestResult, part string) template.HTML {
	type line struct {
		name string
		pts  [][2]float64
	}
	var lines []line
	maxT := 1.0
	for _, res := range results {
		for _, tr := range res.Traces {
			if tr.Part != part || len(tr.Samples) < 2 || !tr.Curve.Moved {
				continue
			}
			s := tr.Samples
			p0, pn := s[0], s[len(s)-1]
			dx, dy := pn.X-p0.X, pn.Y-p0.Y
			d2 := dx*dx + dy*dy
			var t0 float64 = -1
			var pts [][2]float64
			for _, smp := range s {
				p := ((smp.X-p0.X)*dx + (smp.Y-p0.Y)*dy) / d2
				if t0 < 0 && p >= 0.02 {
					t0 = smp.MS
				}
				pts = append(pts, [2]float64{smp.MS, p})
			}
			if t0 < 0 {
				continue
			}
			var shifted [][2]float64
			for _, pt := range pts {
				t := pt[0] - t0
				if t < -20 {
					continue
				}
				shifted = append(shifted, [2]float64{t, pt[1]})
				maxT = math.Max(maxT, t)
			}
			lines = append(lines, line{name: res.Target, pts: shifted})
		}
	}
	for _, spec := range test.Trace {
		if spec.Part == part && spec.Timing != nil {
			want, err := compare.Expected(spec.Timing.DurationMS, spec.Timing.Easing)
			if err == nil {
				pts := [][2]float64{{0, 0.02}}
				for i, l := range compare.Levels {
					pts = append(pts, [2]float64{want.Times[i], l})
				}
				pts = append(pts, [2]float64{want.Duration, 0.98})
				lines = append(lines, line{name: fmt.Sprintf("%.0fms %s", spec.Timing.DurationMS, spec.Timing.Easing), pts: pts})
				maxT = math.Max(maxT, want.Duration)
			}
		}
	}
	if len(lines) == 0 {
		return ""
	}
	maxT = math.Min(maxT, 2000)
	const W, H, L, B = 560.0, 240.0, 48.0, 32.0
	x := func(t float64) float64 { return L + (t+20)/(maxT+20)*(W-L-16) }
	y := func(p float64) float64 { return H - B - math.Max(-0.1, math.Min(1.1, p))*(H-B-16) }
	var sb strings.Builder
	label := fmt.Sprintf("Progress of %s over time, aligned where each target's movement starts: %s.", part, func() string {
		var n []string
		for _, l := range lines {
			n = append(n, l.name)
		}
		return strings.Join(n, ", ")
	}())
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
					rt.HasClips = rt.HasClips || res.Clip != ""
				}
			}
			var parts []string
			for _, cmp := range rep.Comparisons {
				if cmp.Contract != c.ID || cmp.Test != test.Name {
					continue
				}
				if cmp.Kind == "pixels" {
					rt.Pixels = append(rt.Pixels, cmp)
				} else {
					rt.Timing = append(rt.Timing, cmp)
				}
			}
			if len(rt.Pixels) > 0 {
				rt.PixelLine = comparisonLine(rt.Pixels)
			}
			seen := map[string]bool{}
			for _, spec := range test.Trace {
				if !seen[spec.Part] {
					seen[spec.Part] = true
					parts = append(parts, spec.Part)
				}
			}
			for _, p := range parts {
				rt.Chart += chart(test, rt.Results, p)
			}
			data.Tests = append(data.Tests, rt)
		}
	}
	funcs := template.FuncMap{
		"shot": func(res TestResult, key string) string { return res.Screenshots[key] },
		"firstFrame": func(clip string) string {
			return filepath.Join(filepath.Dir(clip), "frames", "0.png")
		},
		"pair":   pairLabel,
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
.st-pass { color: var(--pass); } .st-fail, .st-broken { color: var(--fail); } .st-warning { color: var(--warn); } .st-not-advertised { color: var(--na); }
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
{{if .PixelLine}}<p>{{.PixelLine}}</p>{{end}}
{{$t := .}}
{{range $cb := .Combos}}
<ul class="shots" aria-label="Screenshots at {{$cb.Label}}">
{{range $t.Results}}{{$res := .}}{{with shot $res $cb.Key}}<li><figure><img src="{{.}}" alt="The component on {{$res.Target}} at {{$cb.Label}}" width="{{$cb.Viewport.Width}}" height="{{$cb.Viewport.Height}}"><figcaption>{{$res.Target}} · {{$cb.Label}}</figcaption></figure></li>{{end}}{{end}}
</ul>
{{end}}
{{if .Pixels}}
<div class="table" role="region" tabindex="0" aria-label="Differences between targets (worst over all viewports and colour schemes)">
<table>
<caption>Differences between targets (worst over all viewports and colour schemes)</caption>
<thead><tr><th scope="col">Pair</th><th scope="col">Pixels that differ</th><th scope="col">Result</th><th scope="col">Where</th></tr></thead>
<tbody>
{{range .Pixels}}<tr><th scope="row">{{pair .}}</th><td>{{value .}} (threshold {{.Threshold}}%)</td><td class="st {{if .Warning}}st-warning{{else}}st-pass{{end}}">{{if .Warning}}warning{{else}}pass{{end}}</td><td><figure><img src="{{.Diff}}" alt="Differences between {{.A}} and {{.B}} at {{.Combo}}: pixels that differ are red" loading="lazy"><figcaption>{{.Combo}}{{if .Note}}: {{.Note}}{{end}}</figcaption></figure></td></tr>
{{end}}</tbody>
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
{{if .HasClips}}
<h3>Recorded clips</h3>
<ul class="shots">
{{range .Results}}{{if .Clip}}<li><figure><picture><source media="(prefers-reduced-motion: reduce)" srcset="{{firstFrame .Clip}}"><img src="{{.Clip}}" alt="Clip of the test running on {{.Target}}"></picture><figcaption>{{.Target}} · {{.Clip}}</figcaption></figure></li>{{end}}{{end}}
</ul>
{{end}}
</section>
{{end}}
</main>
</body>
</html>
`
