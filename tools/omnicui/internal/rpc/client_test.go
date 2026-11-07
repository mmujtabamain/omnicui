package rpc

import (
	"errors"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"sync"
	"testing"
	"time"

	"omnicui/internal/testutil"
)

func start(t *testing.T, env ...string) (*Client, *strings.Builder) {
	t.Helper()
	var wire strings.Builder
	settings := map[string]string{}
	for _, kv := range env {
		k, v, _ := strings.Cut(kv, "=")
		settings[k] = v
	}
	dir := t.TempDir()
	c, err := Start(Options{Target: "fake", Dir: dir, Command: testutil.Command(t, dir, settings), Wire: &wire, WireMu: &sync.Mutex{}})
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(c.Stop)
	return c, &wire
}

func mount(t *testing.T, c *Client) {
	t.Helper()
	err := c.Call("mount", map[string]any{"component": "ui.counter", "props": map[string]any{}, "viewport": map[string]int{"width": 64, "height": 64}, "colorScheme": "light"}, time.Second, nil)
	if err != nil {
		t.Fatal(err)
	}
}

// R-PROTO-1: requests and replies are JSON-RPC 2.0, one per line, matched by id.
func TestConversation(t *testing.T) {
	c, wire := start(t)
	var ad struct{ Can []string }
	if err := c.Call("advertise", map[string]any{"protocol": Protocol}, time.Second, &ad); err != nil || len(ad.Can) == 0 {
		t.Fatalf("%v %v", ad, err)
	}
	mount(t, c)
	if err := c.Call("press", map[string]any{"part": "ui.counter.increment"}, time.Second, nil); err != nil {
		t.Fatal(err)
	}
	var text string
	if err := c.Call("read", map[string]any{"part": "ui.counter.value", "field": "text"}, time.Second, &text); err != nil || text != "1" {
		t.Fatalf("%q %v", text, err)
	}
	lines := strings.Split(strings.TrimSpace(wire.String()), "\n")
	if len(lines) != 8 || !strings.HasPrefix(lines[0], `fake > {"jsonrpc":"2.0","id":1,"method":"advertise"`) || !strings.HasPrefix(lines[7], `fake < {"id":4,"jsonrpc":"2.0","result":"1"}`) {
		t.Errorf("wire log:\n%s", wire.String())
	}
}

// R-PROTO-2: an error reply is a RemoteError: it fails a test, and the adapter stays usable.
func TestErrorReply(t *testing.T) {
	c, _ := start(t)
	mount(t, c)
	err := c.Call("press", map[string]any{"part": "ui.counter.nope"}, time.Second, nil)
	var re *RemoteError
	if !errors.As(err, &re) || re.Code != CodeNoPart || re.Message != "no part ui.counter.nope" {
		t.Fatalf("%#v", err)
	}
	if c.Broken() != nil {
		t.Fatal("an error reply must not break the adapter")
	}
	mount(t, c)
}

func broken(t *testing.T, err error, want string) *BrokenError {
	t.Helper()
	var b *BrokenError
	if !errors.As(err, &b) || !strings.Contains(b.Reason, want) {
		t.Fatalf("want a BrokenError containing %q, got %v", want, err)
	}
	return b
}

// R-PROTO-3 (failure: a worker stops mid-task): the adapter exits during a request.
func TestCrash(t *testing.T) {
	c, _ := start(t, "FAKE_CRASH_ON=2")
	c.Call("advertise", nil, time.Second, nil)
	b := broken(t, c.Call("mount", map[string]any{"component": "ui.counter"}, 5*time.Second, nil), "exited with code 3 during mount ui.counter")
	if !strings.Contains(strings.Join(b.StderrTail, "\n"), "fatal: the fake adapter crashed on purpose") {
		t.Errorf("stderr tail: %v", b.StderrTail)
	}
	// Broken stays broken: nothing more is sent.
	broken(t, c.Call("advertise", nil, time.Second, nil), "exited")
}

// R-PROTO-4 (failure: a connection drops): the adapter closes its standard output but keeps running.
func TestClosedOutput(t *testing.T) {
	c, _ := start(t, "FAKE_CLOSE_ON=1")
	broken(t, c.Call("advertise", nil, 5*time.Second, nil), "closed its standard output during advertise")
}

// R-PROTO-5 (failure: a dependency times out): no reply within the timeout; the adapter is killed.
func TestTimeout(t *testing.T) {
	pidFile := filepath.Join(t.TempDir(), "pid")
	c, _ := start(t, "FAKE_HANG_ON=2", "FAKE_PID_FILE="+pidFile)
	c.Call("advertise", nil, time.Second, nil)
	began := time.Now()
	broken(t, c.Call("press", map[string]any{"part": "ui.counter.increment"}, 300*time.Millisecond, nil), "timed out after 300ms on press ui.counter.increment")
	if time.Since(began) > 8*time.Second {
		t.Errorf("took %v", time.Since(began))
	}
	data, _ := os.ReadFile(pidFile)
	pid, _ := strconv.Atoi(string(data))
	if pid == 0 {
		t.Fatal("no pid")
	}
	if alive(pid) {
		t.Errorf("the hung adapter (pid %d) is still running", pid)
	}
}

// R-PROTO-6: anything on standard output that is not a reply breaks the adapter, and is shown.
func TestStrayOutput(t *testing.T) {
	c, _ := start(t, "FAKE_STRAY_ON=1")
	broken(t, c.Call("advertise", nil, time.Second, nil), `stray output on standard output during advertise: "Loading theme… done"`)
}

// R-PROTO-7: a reply with the wrong id breaks the adapter.
func TestWrongID(t *testing.T) {
	c, _ := start(t, "FAKE_WRONG_ID_ON=1")
	broken(t, c.Call("advertise", nil, time.Second, nil), "does not match request id 1")
}

// R-PROTO-8: a command that can't start is an error, not a hang.
func TestBadCommand(t *testing.T) {
	c, err := Start(Options{Target: "x", Dir: t.TempDir(), Command: "exec ./does-not-exist"})
	if err != nil {
		return
	}
	defer c.Stop()
	broken(t, c.Call("advertise", nil, 5*time.Second, nil), "exited with code")
}

// R-PROTO-12: when omnicui is interrupted, KillAll ends every adapter still running, and none is
// left behind (adapters are in process groups of their own, so they don't get the Ctrl+C).
func TestKillAll(t *testing.T) {
	var pids []int
	for i := 0; i < 2; i++ {
		pidFile := filepath.Join(t.TempDir(), "pid")
		c, _ := start(t, "FAKE_PID_FILE="+pidFile)
		if err := c.Call("advertise", nil, 5*time.Second, nil); err != nil {
			t.Fatal(err)
		}
		data, _ := os.ReadFile(pidFile)
		pid, _ := strconv.Atoi(string(data))
		pids = append(pids, pid)
	}
	if n := Running(); n < 2 {
		t.Fatalf("running: %d", n)
	}
	if n := KillAll(); n < 2 {
		t.Fatalf("killed %d", n)
	}
	deadline := time.Now().Add(5 * time.Second)
	for Running() > 0 && time.Now().Before(deadline) {
		time.Sleep(20 * time.Millisecond)
	}
	if Running() != 0 {
		t.Errorf("still running: %d", Running())
	}
	for _, pid := range pids {
		if alive(pid) {
			t.Errorf("adapter %d is still alive", pid)
		}
	}
}
