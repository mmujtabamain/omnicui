package check

import (
	"encoding/json"
	"errors"
	"fmt"
	"io/fs"
	"os"
	"regexp"
	"sort"
	"strings"

	"omnicui/internal/contract"
)

// FilterTests keeps the tests keep accepts, and the contracts that still have one. The
// contracts are copies; the originals are unchanged.
func FilterTests(cs []*contract.Contract, keep func(*contract.Contract, contract.Test) bool) []*contract.Contract {
	var out []*contract.Contract
	for _, c := range cs {
		var tests []contract.Test
		for _, t := range c.Tests {
			if keep(c, t) {
				tests = append(tests, t)
			}
		}
		if len(tests) == 0 {
			continue
		}
		cc := *c
		cc.Tests = tests
		out = append(out, &cc)
	}
	return out
}

// TestMatches says whether a test's name matches one of the patterns of `check --test`. A
// pattern with * or ? is a glob over the whole name; any other is a part of the name. Case is
// ignored either way.
func TestMatches(name string, patterns []string) bool {
	n := strings.ToLower(name)
	for _, p := range patterns {
		p = strings.ToLower(p)
		if strings.ContainsAny(p, "*?") {
			if globRegexp(p).MatchString(n) {
				return true
			}
		} else if strings.Contains(n, p) {
			return true
		}
	}
	return false
}

func globRegexp(p string) *regexp.Regexp {
	var b strings.Builder
	b.WriteString("^")
	for _, r := range p {
		switch r {
		case '*':
			b.WriteString(".*")
		case '?':
			b.WriteString(".")
		default:
			b.WriteString(regexp.QuoteMeta(string(r)))
		}
	}
	b.WriteString("$")
	return regexp.MustCompile(b.String())
}

// NearestTests suggests, for a --test that matched nothing, the test names sharing a word with
// it (at most five), as ` (tests with a word in common: …)`, or names a few tests.
func NearestTests(cs []*contract.Contract, patterns []string) string {
	words := map[string]bool{}
	for _, p := range patterns {
		for _, w := range strings.FieldsFunc(strings.ToLower(p), func(r rune) bool { return r == ' ' || r == '*' || r == '?' || r == '-' }) {
			if len(w) > 2 {
				words[w] = true
			}
		}
	}
	var near, all []string
	for _, c := range cs {
		for _, t := range c.Tests {
			if t.Keyboard != nil {
				continue
			}
			all = append(all, fmt.Sprintf("`%s`", t.Name))
			for _, w := range strings.Fields(strings.ToLower(t.Name)) {
				if words[w] {
					near = append(near, fmt.Sprintf("`%s`", t.Name))
					break
				}
			}
		}
	}
	switch {
	case len(near) > 0:
		return " (tests with a word in common: " + strings.Join(first(near, 5), ", ") + ")"
	case len(all) > 0:
		return " (for example " + strings.Join(first(all, 3), ", ") + "; a pattern is part of a name, or a glob with *)"
	}
	return ""
}

func first(xs []string, n int) []string {
	if len(xs) > n {
		return append(xs[:n:n], "…")
	}
	return xs
}

// LastProblems reads a previous results.json and returns the tests (contract id, NUL, test
// name) that failed or broke on some target, or had a warning between targets: what
// `check --failed` runs again.
func LastProblems(path string) (map[string]bool, error) {
	data, err := os.ReadFile(path)
	if errors.Is(err, fs.ErrNotExist) {
		return nil, fmt.Errorf("there is no earlier run to take the failures from (%s doesn't exist); run `omnicui check` first", path)
	}
	if err != nil {
		return nil, err
	}
	var last struct {
		Results []struct {
			Contract, Test string
			Status         Status
		} `json:"results"`
		Comparisons []struct {
			Contract, Test string
			Warning        bool
		} `json:"comparisons"`
	}
	if err := json.Unmarshal(data, &last); err != nil {
		return nil, fmt.Errorf("%s: %v", path, err)
	}
	out := map[string]bool{}
	for _, r := range last.Results {
		if r.Status == Fail || r.Status == Broken {
			out[r.Contract+"\x00"+r.Test] = true
		}
	}
	for _, c := range last.Comparisons {
		if c.Warning {
			out[c.Contract+"\x00"+c.Test] = true
		}
	}
	return out, nil
}

// sortedTests is used by tests of this file.
func sortedTests(m map[string]bool) []string {
	var out []string
	for k := range m {
		out = append(out, strings.ReplaceAll(k, "\x00", " › "))
	}
	sort.Strings(out)
	return out
}
