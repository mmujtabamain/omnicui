package project

import (
	"errors"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"

	"gopkg.in/yaml.v3"
)

// Reference is a recorded screenshot of the app the targets reproduce, listed in references.yaml at
// the project's root. Each target's screenshot of the test, at the image's size (or its crop's) and
// colour scheme, is compared with it pixel by pixel, the same way two targets of one design are.
type Reference struct {
	// The contract's id and the test's name.
	Contract string `yaml:"contract"`
	Test     string `yaml:"test"`
	// Image is the screenshot, relative to the project's folder.
	Image string `yaml:"image"`
	// ColorScheme it was recorded in; light when empty.
	ColorScheme string `yaml:"colorScheme"`
	// Crop: the part of the image the test's viewport shows (a component of a larger window).
	Crop *Box `yaml:"crop"`
	// Mask: areas of the image (in its own coordinates) that may differ, such as the OS's window
	// controls. MaskParts: parts of the contract left out on each target (their boxes).
	Mask      []Box    `yaml:"mask"`
	MaskParts []string `yaml:"maskParts"`
	// Why the reference holds for this test: the state it was recorded in.
	Why string `yaml:"why"`
}

// Box is an area of an image, in pixels.
type Box struct {
	X      float64 `yaml:"x"`
	Y      float64 `yaml:"y"`
	Width  float64 `yaml:"width"`
	Height float64 `yaml:"height"`
}

// ReferencesFile is the list's path.
func (p Project) ReferencesFile() string { return filepath.Join(p.Root, "references.yaml") }

// References loads references.yaml; none when there is no such file. An entry without a contract,
// a test or an image, or whose image is missing, is an error that names it.
func (p Project) References() ([]Reference, error) {
	data, err := os.ReadFile(p.ReferencesFile())
	if errors.Is(err, fs.ErrNotExist) {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}
	var refs []Reference
	if err := yaml.Unmarshal(data, &refs); err != nil {
		return nil, fmt.Errorf("references.yaml: %w", err)
	}
	for i, r := range refs {
		if r.Contract == "" || r.Test == "" || r.Image == "" {
			return nil, fmt.Errorf("references.yaml: entry %d needs contract, test and image", i+1)
		}
		if _, err := os.Stat(filepath.Join(p.Root, r.Image)); err != nil {
			return nil, fmt.Errorf("references.yaml: entry %d (%s › %s): no image %s", i+1, r.Contract, r.Test, r.Image)
		}
		if refs[i].ColorScheme == "" {
			refs[i].ColorScheme = "light"
		}
	}
	return refs, nil
}
