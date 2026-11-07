package contract

// Role says which fields of a part can be checked and which actions it accepts.
// Role names and field names follow WAI-ARIA.
type Role struct {
	Fields  map[string]string // field → its type: string, boolean or number
	Actions map[string]bool
}

func set(xs ...string) map[string]bool {
	m := map[string]bool{}
	for _, x := range xs {
		m[x] = true
	}
	return m
}

// Every role also has `role` (the role assistive technology reports, compared with the
// contract's) and `visible` (the part has a size on screen and is inside the viewport); the
// focusable roles have `focused`. They are added to each role below.

// Roles is the full table. A role that is not here is refused by the schema.
var Roles = map[string]Role{
	"button":   {Fields: map[string]string{"text": "string", "name": "string", "disabled": "boolean"}, Actions: set("press", "focus", "key", "hover")},
	"switch":   {Fields: map[string]string{"text": "string", "name": "string", "disabled": "boolean", "checked": "boolean"}, Actions: set("press", "focus", "key", "hover")},
	"checkbox": {Fields: map[string]string{"text": "string", "name": "string", "disabled": "boolean", "checked": "boolean"}, Actions: set("press", "focus", "key", "hover")},
	"slider":   {Fields: map[string]string{"name": "string", "disabled": "boolean", "value": "number"}, Actions: set("press", "focus", "key", "drag", "hover")},
	"textbox":  {Fields: map[string]string{"text": "string", "name": "string", "disabled": "boolean"}, Actions: set("press", "focus", "key", "type", "hover")},
	"status":   {Fields: map[string]string{"text": "string", "name": "string"}, Actions: set("hover")},
	"heading":  {Fields: map[string]string{"text": "string", "name": "string"}, Actions: set("hover")},
	"generic":  {Fields: map[string]string{"text": "string"}, Actions: set("hover")},
}

// focusable roles can hold keyboard focus.
var focusable = []string{"button", "switch", "checkbox", "slider", "textbox"}

func init() {
	for name, r := range Roles {
		r.Fields["role"] = "string"
		r.Fields["visible"] = "boolean"
		Roles[name] = r
	}
	// selected: a button that stays chosen (a chip, a tab, a segment of a segmented control).
	Roles["button"].Fields["selected"] = "boolean"
	for _, name := range focusable {
		Roles[name].Fields["focused"] = "boolean"
	}
}

// RoleNames lists the roles, for `role:` expectations.
func RoleNames() []string {
	out := make([]string, 0, len(Roles))
	for n := range Roles {
		out = append(out, n)
	}
	return out
}
