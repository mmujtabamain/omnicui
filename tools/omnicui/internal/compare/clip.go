package compare

// Frame is one frame a target recorded.
type Frame struct {
	MS   float64 `json:"ms"`
	File string  `json:"file"`
}
