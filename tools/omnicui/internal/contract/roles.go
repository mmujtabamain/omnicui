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
