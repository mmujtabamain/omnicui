package main

import (
	"fmt"
	"sort"
	"strings"

	"omnicui/internal/contract"
)

// contractHelp is `omnicui help contract`: everything a contract can say, its defaults, and how
// targets are compared, on one screen. The full specification is docs/SPEC.md.
func contractHelp() string {
	d := contract.DefaultPixelThresholds
	var roles []string
	for _, name := range contract.RoleNames() {
		r := contract.Roles[name]
		var fields, actions []string
		for f := range r.Fields {
			fields = append(fields, f)
		}
		for a := range r.Actions {
			actions = append(actions, a)
		}
		sort.Strings(fields)
		sort.Strings(actions)
		roles = append(roles, fmt.Sprintf("  %-9s fields: %s\n            actions: %s", name, strings.Join(fields, ", "), strings.Join(actions, ", ")))
	}
	sort.Strings(roles)
	return fmt.Sprintf(`A contract: contracts/<name>.contract.yaml

  id: ui.counter                       lower-case words joined by dots; a part's id is <id>.<part>
  props:  { start: { type: integer, default: 0 } }    types: string, integer, number, boolean
  events: { changed: { count: { type: integer } } }
  parts:  { value: { role: status }, increment: { role: button } }
  tests:
    - name: each press adds one
      props: { start: 0 }
      steps:  [ { action: press, part: increment, times: 3 } ]       times: 1–100
              actions: press, focus, key (key:), type (text:), drag (dx:, dy:), hover; or
              { do: swipe, part: row, with: { direction: left } }: an action the targets define
              and advertise as do.<name> (scroll, swipe, refresh, back, pick…)
      expect: [ { part: value, text: "3" }, { event: changed, times: 3, last: { count: 3 } } ]
      screenshot: true                 at every viewport × colour scheme; or
                  { mask: [part…] }    leave those parts out, wherever either target shows them
                  { part: toolbar }    compare only that part's box on each target
      trace: [ { part: value, point: center, timing: { duration: 200ms, easing: ease-out } },
               { part: value, property: opacity } ]          property: position, opacity, scale
      record: true                     a clip, compared by its visual progress; { part: x }: of
                                       that part's box only
      allCombinations: true            traces and the clip at every viewport × colour scheme
  viewports:    [ { width: 320, height: 240, props: { … } } ]         default 320×240; props: every test's at that size
  colorSchemes: [ light, dark ]                                       default light
  keyboard:     true                   Enter/Space must do what a press does (see below)
  mask:         [ part… ]              parts left out of every screenshot comparison
  parts: { controls: { role: generic, optional: true } }   shown by some targets only
  thresholds:   { area: %g%%, strength: %g%%, shift: %gpx, spot: %g%%, timing: %gms, motion: %gms }   (the defaults)

Names (props, events, parts, event fields) are camelCase: a lower-case letter, then letters and
digits. card-number is refused; write cardNumber.

Roles, what a test can check on a part of each, and the actions it accepts:
%s
  role     the role assistive technology reports (WAI-ARIA), compared with the value given
  visible  the part has a size on screen and is inside the viewport (needs the target's bounds)
  focused  the part has keyboard focus
  selected a button that stays chosen: a chip, a tab, a segment (targets that advertise selected)

How targets are compared (warnings, never failures, unless check --fail-on says so):
  screenshots  every pair of targets, at every combination; the worst combination is reported.
               area      how much of the drawn area differs (a pixel differs when no pixel within
                         2 px of it in the other screenshot has its colour)
               strength  how far off in colour the recoloured pixels are, on average (100%% = black
                         against white); pixels only displaced are left out
               shift     how far the differing pixels moved: the median distance to the nearest
                         pixel of the same colour, up to 8 px. Shown, but by default it never
                         warns: anti-aliasing alone reaches 3–7 px
               spot      how concentrated the difference is: the share of the worst 24×24 px spot
                         that is strongly off in colour, with no pixel of its colour within 4 px.
                         Glyph noise spreads thinly and displaced edges have their colour nearby;
                         a wrong word, colour or opacity is concentrated, so this finds them
  traces       a point's position (or, with property: opacity or scale, a part's), lined up, then
               compared at 10%%, 20%%, … 90%% of the change, against each other and against the
               named timing (threshold: timing)
  clips        record: true. Each clip becomes its visual progress: how far the picture has gone
               from its first frame to its last, over time. Compared like traces (threshold:
               motion), so a fade, a blur or a colour change is compared too, on every target,
               whatever its frame rate. Traces and clips are taken at the first viewport × colour
               scheme only, unless the test says allCombinations: true. A change that goes out and
               comes back (a spring, a hover in and out) is measured out and back
  keyboard     keyboard: true adds, for every button (Enter, Space), switch and checkbox (Space),
               a test that the key does what a pointer press does: same events, same state.
               "not measured" when the part isn't on screen when the component mounts (give a
               sheet or a dialog a component of its own) or the press changes nothing. A target
               may advertise the keys its platform uses: keys: { button: [Enter] }
  visibility   a part whose text, name, value or checked state was read while it had no size on
               screen (targets that report bounds)

Thresholds far below the defaults make every comparison warn: correct renderings by different
toolkits differ by up to 26%% of area and by up to about 1.6%% in spot; the same easing measured on
two toolkits by 5–15 ms, and the same clip by up to 30 ms. Try them on saved screenshots with: omnicui compare a.png b.png --diff d.png

Every reply waits until the UI has settled: nothing animating, events processed, and the app not
busy (for example, waiting for a network reply), when the app tells its target so.

The full specification: docs/SPEC.md
`, d.Area, d.Strength, d.Shift, d.Spot, contract.DefaultTimingThreshold, contract.DefaultMotionThreshold, strings.Join(roles, "\n"))
}
