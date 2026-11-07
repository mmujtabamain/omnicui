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
      expect: [ { part: value, text: "3" }, { event: changed, times: 3, last: { count: 3 } } ]
      screenshot: true                 at every viewport × colour scheme
      trace: [ { part: value, point: center, timing: { duration: 200ms, easing: ease-out } } ]
      record: true                     a clip; never compared
  viewports:    [ { width: 320, height: 240 } ]                       default 320×240
  colorSchemes: [ light, dark ]                                       default light
  thresholds:   { area: %g%%, strength: %g%%, shift: %gpx, spot: %g%%, timing: %gms }   (the defaults)

Names (props, events, parts, event fields) are camelCase: a lower-case letter, then letters and
digits. card-number is refused; write cardNumber.

Roles, what a test can check on a part of each, and the actions it accepts:
%s
  role     the role assistive technology reports (WAI-ARIA), compared with the value given
  visible  the part has a size on screen and is inside the viewport (needs the target's bounds)
  focused  the part has keyboard focus

How targets are compared (warnings, never failures, unless check --fail-on says so):
  screenshots  every pair of targets, at every combination; the worst combination is reported.
               area      how much of the drawn area differs (a pixel differs when no pixel within
                         2 px of it in the other screenshot has its colour)
               strength  how far off in colour the differing pixels are, on average (100%% = black
                         against white)
               shift     how far the differing pixels moved: the median distance to the nearest
                         pixel of the same colour, up to 8 px
               spot      how concentrated the difference is: the share of the worst 24×24 px spot
                         that is strongly off in colour. Glyph noise spreads thinly; a wrong word,
                         colour or opacity is concentrated, so this is the one that finds them
  traces       lined up, then compared at 10%%, 20%%, … 90%% of the movement, against each other
               and against the named timing. Taken at the first viewport × colour scheme only;
               the recording too.
  visibility   a part whose text, name, value or checked state was read while it had no size on
               screen (targets that report bounds)

Thresholds far below the defaults make every comparison warn: correct renderings by different
toolkits differ by a few percent of area and by up to 3%% in spot, and the same easing measured on
two toolkits by 5–15 ms. Try them on saved screenshots with: omnicui compare a.png b.png --diff d.png

Every reply waits until the UI has settled: nothing animating, events processed, and the app not
busy (for example, waiting for a network reply), when the app tells its target so.

The full specification: docs/SPEC.md
`, d.Area, d.Strength, d.Shift, d.Spot, contract.DefaultTimingThreshold, strings.Join(roles, "\n"))
}
