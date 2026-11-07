//go:build windows

package rpc

import (
	"os/exec"
	"strconv"
	"strings"
)

// shellCommand runs the adapter command with cmd. cmd has no `exec`, and needs none: a leading
// `exec ` (written for /bin/sh) is dropped, so one target.yaml works on every OS.
func shellCommand(command string) *exec.Cmd {
	return exec.Command("cmd", "/C", strings.TrimPrefix(strings.TrimSpace(command), "exec "))
}

func setGroup(cmd *exec.Cmd) {}

// killGroup ends the adapter and everything it started (node, java, cargo's child).
func killGroup(cmd *exec.Cmd) {
	if cmd.Process != nil {
		_ = exec.Command("taskkill", "/T", "/F", "/PID", strconv.Itoa(cmd.Process.Pid)).Run()
		_ = cmd.Process.Kill()
	}
}
