//go:build !windows

package rpc

import (
	"os/exec"
	"syscall"
)

func shellCommand(command string) *exec.Cmd { return exec.Command("/bin/sh", "-c", command) }

// setGroup puts the adapter in its own process group, so that a wrapper such as `npm` or
// `cargo run` is stopped together with the program it started.
func setGroup(cmd *exec.Cmd) { cmd.SysProcAttr = &syscall.SysProcAttr{Setpgid: true} }

func killGroup(cmd *exec.Cmd) {
	if cmd.Process != nil {
		_ = syscall.Kill(-cmd.Process.Pid, syscall.SIGKILL)
	}
}
