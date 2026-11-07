package check

import (
	"fmt"
	"io"
	"os"
	"path/filepath"
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
	var parts, warnings []string
	for _, c := range cs {
		parts = append(parts, pairLabel(c)+" "+valueText(c))
		if c.Warning {
			warnings = append(warnings, pairLabel(c)+" "+valueText(c))
		}
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
	names := []string{}
	for _, t := range rep.Targets {
		names = append(names, t.Name)
	}
	col := 30
	for _, c := range rep.Contracts {
		for _, t := range c.Tests {
			col = max(col, utf8.RuneCountInString(t.Name)+2)
		}
	}
	for _, c := range rep.Contracts {
		fmt.Fprintf(w, "\n%s", pad(c.ID, col+2))
		for _, n := range names {
			fmt.Fprint(w, pad(n, 10))
		}
		fmt.Fprintln(w)
		for _, test := range c.Tests {
			fmt.Fprintf(w, "  %s", pad(test.Name, col))
			var notes []string
			missing := map[string][]string{}
			for _, n := range names {
				res := find(rep, c.ID, test.Name, n)
				if res == nil {
					fmt.Fprint(w, pad("-", 10))
					continue
				}
				st := string(res.Status)
				if res.Status == Fail || res.Status == Broken {
					st = strings.ToUpper(st)
				}
				fmt.Fprint(w, pad(st, 10))
				if res.Status != Pass {
					for _, m := range res.Messages {
						notes = append(notes, n+": "+m)
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
			var pix []Comparison
			traces := map[string][]Comparison{}
			var partOrder []string
			for _, cmp := range rep.Comparisons {
				if cmp.Contract != c.ID || cmp.Test != test.Name {
					continue
				}
				if cmp.Kind == "pixels" {
					pix = append(pix, cmp)
				} else {
					if _, ok := traces[cmp.Part]; !ok {
						partOrder = append(partOrder, cmp.Part)
					}
					traces[cmp.Part] = append(traces[cmp.Part], cmp)
				}
			}
			if len(pix) > 0 {
				fmt.Fprintf(w, "    %s%s\n", pad("screenshot", col-2), comparisonLine(pix))
			}
			for _, p := range partOrder {
				fmt.Fprintf(w, "    %s%s\n", pad("trace "+p, col-2), comparisonLine(traces[p]))
			}
			for _, k := range sortedKeys(missing) {
				fmt.Fprintf(w, "    %s%s\n", pad(k, col-2), "not advertised by "+strings.Join(missing[k], ", "))
			}
		}
	}
	for _, r := range rep.Required {
		fmt.Fprintf(w, "\nrequired: %s\n", r)
	}
	s := rep.Summary
	fmt.Fprintf(w, "\n%d pass · %d fail · %d broken · %d not advertised · %d warnings\n", s.Pass, s.Fail, s.Broken, s.NotAdvertised, s.Warnings)
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
