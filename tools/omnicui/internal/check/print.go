package check

import (
	"fmt"
	"io"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"unicode/utf8"
)

func pad(s string, n int) string {
	if w := utf8.RuneCountInString(s); w < n {
		return s + strings.Repeat(" ", n-w)
	}
	return s + " "
}

func pairLabel(c Comparison) string {
	if c.B == "timing" {
		return c.A + " vs timing"
	}
	return c.A + " ↔ " + c.B
}

func unit(kind string) string {
	if kind == "pixels" {
		return "%"
	}
	return " ms"
}

func valueText(c Comparison) string {
	if c.Value >= 1e9 {
		return "∞"
	}
	if c.Kind == "pixels" {
		return fmt.Sprintf("%.1f%%", c.Value)
	}
	if c.Value < 10 {
		return fmt.Sprintf("%.1f ms", c.Value)
	}
	return fmt.Sprintf("%.0f ms", c.Value)
}

// comparisonLine is one line about a test's screenshots or one traced part.
func comparisonLine(cs []Comparison) string {
	var parts, warnings, notes []string
	for _, c := range cs {
		if strings.HasPrefix(c.Note, "not compared: ") {
			notes = append(notes, pairLabel(c)+": "+strings.TrimPrefix(c.Note, "not compared: "))
			continue
		}
		parts = append(parts, pairLabel(c)+" "+valueText(c))
		if c.Warning {
			warnings = append(warnings, pairLabel(c)+" "+valueText(c))
		}
	}
	if len(parts) == 0 {
		return "not compared: " + strings.Join(notes, "; ")
	}
	th := cs[0].Threshold
	thText := fmt.Sprintf("%g%s", th, unit(cs[0].Kind))
	if len(warnings) > 0 {
		return fmt.Sprintf("%s   warning: %s above %s", strings.Join(parts, " · "), strings.Join(warnings, ", "), thText)
	}
	return fmt.Sprintf("%s   pass (all under %s)", strings.Join(parts, " · "), thText)
}

// Print writes the console report.
// Print writes the results table, and where the results and the report are (runDir).
func Print(w io.Writer, rep *Report, runDir string) {
	for _, t := range rep.Targets {
		if t.Broken != "" {
			fmt.Fprintf(w, "%s: broken: %s\n", t.Name, t.Broken)
			for _, l := range tailN(t.Stderr, 8) {
				fmt.Fprintf(w, "  stderr | %s\n", l)
			}
		}
	}
	for _, t := range rep.Targets {
		if why, ok := strings.CutPrefix(t.Probe, "failed: "); ok {
			fmt.Fprintf(w, "%s: render check failed: %s; screenshots from %s are off for this run\n", t.Name, why, t.Name)
		}
	}
	names := []string{}
	for _, t := range rep.Targets {
		names = append(names, t.Name)
	}
	// A message that many tests of one target share (a browser that died, a missing component)
	// is printed once, here, instead of under every test.
	repeated := repeatedMessages(rep)
	for _, n := range names {
		for _, m := range sortedKeys(repeated[n]) {
			fmt.Fprintf(w, "%s: %d tests: %s\n", n, repeated[n][m], m)
		}
	}
	col := 30
	for _, c := range rep.Contracts {
		for _, t := range c.Tests {
			col = max(col, utf8.RuneCountInString(t.Name)+2)
		}
	}
	// A column per target, wide enough for its name and the longest status in it.
	tw := 10
	for _, n := range names {
		tw = max(tw, utf8.RuneCountInString(n)+2)
	}
	for _, r := range rep.Results {
		tw = max(tw, len(r.Status)+2)
	}
	for _, c := range rep.Contracts {
		fmt.Fprintf(w, "\n%s", pad(c.ID, col+2))
		for _, n := range names {
			fmt.Fprint(w, pad(n, tw))
		}
		fmt.Fprintln(w)
		for _, test := range c.Tests {
			fmt.Fprintf(w, "  %s", pad(test.Name, col))
			var notes []string
			missing := map[string][]string{}
			for _, n := range names {
				res := find(rep, c.ID, test.Name, n)
				if res == nil {
					fmt.Fprint(w, pad("-", tw))
					continue
				}
				st := string(res.Status)
				if res.Status == Fail || res.Status == Broken {
					st = strings.ToUpper(st)
				}
				fmt.Fprint(w, pad(st, tw))
				if res.Status != Pass {
					for _, m := range res.Messages {
						if repeated[n][m] == 0 {
							notes = append(notes, n+": "+m)
						}
					}
					for _, l := range tailN(res.Stderr, 5) {
						notes = append(notes, n+": stderr | "+l)
					}
				}
				for _, k := range sortedKeys(res.Missing) {
					if res.Status == Pass {
						missing[k] = append(missing[k], fmt.Sprintf("%s (%s)", n, res.Missing[k]))
					}
				}
			}
			fmt.Fprintln(w)
			for _, n := range notes {
				fmt.Fprintf(w, "      %s\n", n)
			}
			var pix, seen []Comparison
			traces := map[string][]Comparison{}
			var partOrder []string
			for _, cmp := range rep.Comparisons {
				if cmp.Contract != c.ID || cmp.Test != test.Name {
					continue
				}
				if cmp.Kind == "pixels" {
					pix = append(pix, cmp)
				} else if cmp.Kind == "visibility" {
					seen = append(seen, cmp)
				} else {
					// One line per traced part (or clip) and combination.
					key := "trace " + cmp.Part
					if cmp.Kind == "motion" {
						key = "clip, visual progress"
					}
					if cmp.Combo != "" && len(Combos(c)) > 1 && test.AllCombinations {
						key += " " + cmp.Combo
					}
					if _, ok := traces[key]; !ok {
						partOrder = append(partOrder, key)
					}
					traces[key] = append(traces[key], cmp)
				}
			}
			if len(pix) > 0 {
				fmt.Fprintf(w, "    screenshots, worst combination per pair:\n")
				var rows []MeasuresRow
				for _, p := range pix {
					if p.Measures == nil {
						continue
					}
					label := pairLabel(p)
					if len(Combos(c)) > 1 {
						label += " " + p.Combo
					}
					rows = append(rows, MeasuresRow{Label: label, Measures: *p.Measures, Over: p.Over, Note: p.Note, SpotPart: p.SpotPart})
				}
				fmt.Fprint(w, MeasuresTable(rows, *pix[0].Thresholds, "      "))
			}
			for _, p := range partOrder {
				fmt.Fprintf(w, "    %s%s\n", pad(p, col-2), comparisonLine(traces[p]))
			}
			if combos := Combos(c); (len(test.Trace) > 0 || test.Record) && len(combos) > 1 && !test.AllCombinations {
				fmt.Fprintf(w, "    %s%s\n", pad("", col-2), "traces and clip: first combination only ("+combos[0].Label()+")")
			}
			for _, v := range seen {
				fmt.Fprintf(w, "    %s%s: warning: %s\n", pad("visibility", col-2), v.A, v.Note)
			}
			for _, k := range sortedKeys(missing) {
				fmt.Fprintf(w, "    %s%s\n", pad(k, col-2), "not advertised by "+strings.Join(missing[k], ", "))
			}
		}
	}
	for _, r := range rep.Required {
		fmt.Fprintf(w, "\nrequired: %s\n", r)
	}
	if slow := Slowest(rep, 5); len(slow) > 0 {
		fmt.Fprintf(w, "\nslowest:")
		for _, r := range slow {
			fmt.Fprintf(w, " %s %s › %s %s;", r.Target, shortID(r.Contract), r.Test, Seconds(r.Ms))
		}
		fmt.Fprintln(w)
		for _, t := range rep.Targets {
			if t.Ms > 0 {
				fmt.Fprintf(w, "  %s took %s, %s of it starting\n", t.Name, Seconds(t.Ms), Seconds(t.StartMs))
			}
		}
	}
	s := rep.Summary
	measured := ""
	if s.NotMeasured > 0 {
		measured = fmt.Sprintf(" · %d not measured", s.NotMeasured)
	}
	fmt.Fprintf(w, "\n%d pass · %d fail · %d broken · %d not advertised%s · %d warnings\n", s.Pass, s.Fail, s.Broken, s.NotAdvertised, measured, s.Warnings)
	if wd, err := os.Getwd(); err == nil {
		if rel, err := filepath.Rel(wd, runDir); err == nil && !strings.HasPrefix(rel, "..") {
			runDir = rel
		}
	}
	fmt.Fprintf(w, "results: %s   report: %s\n", filepath.Join(runDir, "results.json"), filepath.Join(runDir, "report.html"))
}

func tailN(xs []string, n int) []string {
	if len(xs) > n {
		return xs[len(xs)-n:]
	}
	return xs
}

func find(rep *Report, contractID, test, target string) *TestResult {
	for i := range rep.Results {
		r := &rep.Results[i]
		if r.Contract == contractID && r.Test == test && r.Target == target {
			return r
		}
	}
	return nil
}

// repeatedMessages finds, per target, the failure messages that three or more of its tests share.
func repeatedMessages(rep *Report) map[string]map[string]int {
	count := map[string]map[string]int{}
	for _, res := range rep.Results {
		if res.Status == Pass {
			continue
		}
		if count[res.Target] == nil {
			count[res.Target] = map[string]int{}
		}
		for _, m := range res.Messages {
			count[res.Target][m]++
		}
	}
	for t, ms := range count {
		for m, n := range ms {
			if n < 3 {
				delete(ms, m)
			}
		}
		if len(ms) == 0 {
			delete(count, t)
		}
	}
	return count
}

// Slowest returns the n slowest tests of the run, over all targets, slowest first. Only tests
// that took a second or more count: below that, the list would be noise.
func Slowest(rep *Report, n int) []TestResult {
	var out []TestResult
	for _, r := range rep.Results {
		if r.Ms >= 1000 {
			out = append(out, r)
		}
	}
	sort.SliceStable(out, func(i, j int) bool { return out[i].Ms > out[j].Ms })
	if len(out) > n {
		out = out[:n]
	}
	return out
}

// Seconds writes a duration in ms for people: 0.4s, 12s, 2m05s.
func Seconds(ms int64) string {
	switch {
	case ms < 10000:
		return fmt.Sprintf("%.1fs", float64(ms)/1000)
	case ms < 60000:
		return fmt.Sprintf("%ds", (ms+500)/1000)
	default:
		s := (ms + 500) / 1000
		return fmt.Sprintf("%dm%02ds", s/60, s%60)
	}
}

func shortID(id string) string {
	if i := strings.LastIndex(id, "."); i >= 0 {
		return id[i+1:]
	}
	return id
}
