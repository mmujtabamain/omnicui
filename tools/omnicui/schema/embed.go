// Package schema holds the contract JSON Schema. The same file is what editors use through the
// `# yaml-language-server: $schema=…` first line of every contract.
package schema

import _ "embed"

//go:embed contract.schema.json
var JSON []byte
