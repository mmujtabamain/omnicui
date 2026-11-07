// Package rpc runs a target's adapter and talks JSON-RPC 2.0 to it: one request per line on its
// standard input, one response per line on its standard output. Standard error is kept for
// diagnosis and never parsed.
package rpc

import (
	"bufio"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"log/slog"
	"os"
	"os/exec"
	"strings"
	"sync"
	"time"
)

// Protocol is the version omnicui speaks. It is sent in `advertise`.
const Protocol = "1"

// Error codes an adapter may use, in the JSON-RPC server range.
const (
	CodeMethodNotFound = -32601 // the construct isn't supported
	CodeInvalidParams  = -32602
	CodeNoComponent    = -32001 // mount: the target has no such component
	CodeNoPart         = -32002 // the target can't find the part
	CodeStepFailed     = -32003 // the step couldn't be performed
)

// RemoteError is an error reply from the adapter. It fails the test, not the target.
type RemoteError struct {
	Code    int             `json:"code"`
	Message string          `json:"message"`
	Data    json.RawMessage `json:"data,omitempty"`
}

func (e *RemoteError) Error() string { return e.Message }

// BrokenError means the adapter can no longer be used: it timed out, exited, or broke the protocol.
type BrokenError struct {
	Reason     string
	StderrTail []string
}

func (e *BrokenError) Error() string { return e.Reason }

type request struct {
	JSONRPC string `json:"jsonrpc"`
	ID      int64  `json:"id"`
	Method  string `json:"method"`
	Params  any    `json:"params,omitempty"`
}

type response struct {
	JSONRPC string          `json:"jsonrpc"`
	ID      *int64          `json:"id"`
	Result  json.RawMessage `json:"result"`
	Error   *RemoteError    `json:"error"`
}

// Client is one running adapter.
type Client struct {
	Target string

	cmd      *exec.Cmd
	stdin    io.WriteCloser
	lines    chan string
	exited   chan struct{}
	exitErr  error
	stderr   *tail
	nextID   int64
	broken   *BrokenError
	log      *slog.Logger
	wire     io.Writer
	wireMu   *sync.Mutex
	stopOnce sync.Once
}

// Options for starting an adapter.
type Options struct {
	Target  string
	Dir     string
	Command string
	Env     []string
	Log     *slog.Logger
	// Wire, if set, receives every line sent and received, prefixed with the target and a direction.
	Wire   io.Writer
	WireMu *sync.Mutex
}

// ShellCommand is how omnicui runs a command from target.yaml: /bin/sh -c, or cmd /C on Windows.
func ShellCommand(command string) *exec.Cmd { return shellCommand(command) }

// Start runs the adapter's shell command in its target folder.
func Start(o Options) (*Client, error) {
	cmd := shellCommand(o.Command)
	cmd.Dir = o.Dir
	cmd.Env = append(os.Environ(), o.Env...)
	setGroup(cmd)
	stdin, err := cmd.StdinPipe()
	if err != nil {
		return nil, err
	}
	// A pipe of our own (not StdoutPipe), so that reading the last replies never races with Wait.
	stdout, pw, err := os.Pipe()
	if err != nil {
		return nil, err
	}
	cmd.Stdout = pw
	st := newTail(40)
	cmd.Stderr = st
	if err := cmd.Start(); err != nil {
		pw.Close()
		stdout.Close()
		return nil, fmt.Errorf("could not start `%s`: %w", o.Command, err)
	}
	pw.Close()
	c := &Client{Target: o.Target, cmd: cmd, stdin: stdin, lines: make(chan string, 16), exited: make(chan struct{}),
		stderr: st, log: o.Log, wire: o.Wire, wireMu: o.WireMu}
	if c.log == nil {
		c.log = slog.New(slog.NewTextHandler(io.Discard, nil))
	} else {
		c.log = c.log.With("target", o.Target)
	}
	go func() {
		sc := bufio.NewScanner(stdout)
		sc.Buffer(make([]byte, 64*1024), 64*1024*1024)
		for sc.Scan() {
			c.lines <- sc.Text()
		}
		stdout.Close()
		close(c.lines)
	}()
	live.add(c)
	go func() {
		c.exitErr = cmd.Wait()
		live.remove(c)
		close(c.exited)
	}()
	return c, nil
}

// live is every adapter that is still running, so that an interrupted omnicui leaves none behind.
var live = &registry{m: map[*Client]struct{}{}}

type registry struct {
	sync.Mutex
	m map[*Client]struct{}
}

func (r *registry) add(c *Client)    { r.Lock(); r.m[c] = struct{}{}; r.Unlock() }
func (r *registry) remove(c *Client) { r.Lock(); delete(r.m, c); r.Unlock() }

// KillAll ends every adapter that is still running, with everything it started (its process
// group; its process tree on Windows). omnicui calls it when it is interrupted.
func KillAll() int {
	live.Lock()
	defer live.Unlock()
	for c := range live.m {
		killGroup(c.cmd)
	}
	return len(live.m)
}

// Running is how many adapters are running now.
func Running() int {
	live.Lock()
	defer live.Unlock()
	return len(live.m)
}

func (c *Client) record(dir, line string) {
	if c.wire == nil {
		return
	}
	c.wireMu.Lock()
	defer c.wireMu.Unlock()
	fmt.Fprintf(c.wire, "%s %s %s\n", c.Target, dir, line)
}

// SetLog replaces the logger, so that each call is logged with what it belongs to (contract, test,
// combination). The logger should already carry the run and the target.
func (c *Client) SetLog(l *slog.Logger) { c.log = l }

// Broken returns the reason the adapter can't be used any more, or nil.
func (c *Client) Broken() *BrokenError { return c.broken }

func (c *Client) breakWith(reason string) *BrokenError {
	if c.broken == nil {
		// give a dying process a moment to flush its standard error
		select {
		case <-c.exited:
		case <-time.After(300 * time.Millisecond):
		}
		c.broken = &BrokenError{Reason: reason, StderrTail: c.stderr.lines()}
		c.log.Error("target broken", "reason", reason)
		c.Stop()
	}
	return c.broken
}

// Call sends one request and waits for its response, up to timeout.
// It returns *RemoteError for an error reply and *BrokenError when the adapter can't be used.
func (c *Client) Call(method string, params any, timeout time.Duration, result any) error {
	if c.broken != nil {
		return c.broken
	}
	c.nextID++
	id := c.nextID
	line, err := json.Marshal(request{JSONRPC: "2.0", ID: id, Method: method, Params: params})
	if err != nil {
		return err
	}
	started := time.Now()
	c.record(">", string(line))
	if _, err := c.stdin.Write(append(line, '\n')); err != nil {
		// Usually the adapter has already exited; say how, when it has.
		select {
		case <-c.exited:
			return c.breakWith(fmt.Sprintf("the adapter %s before %s", exitDescription(c.exitErr, c.exited), describeCall(method, params)))
		case <-time.After(2 * time.Second):
			return c.breakWith(fmt.Sprintf("could not send %s: the adapter closed its standard input", describeCall(method, params)))
		}
	}
	timer := time.NewTimer(timeout)
	defer timer.Stop()
	for {
		select {
		case l, ok := <-c.lines:
			if !ok {
				select {
				case <-c.exited:
				case <-time.After(2 * time.Second):
				}
				return c.breakWith(fmt.Sprintf("the adapter %s during %s", exitDescription(c.exitErr, c.exited), describeCall(method, params)))
			}
			c.record("<", l)
			if strings.TrimSpace(l) == "" {
				continue
			}
			var r response
			if err := json.Unmarshal([]byte(l), &r); err != nil || r.JSONRPC != "2.0" || r.ID == nil {
				return c.breakWith(fmt.Sprintf("stray output on standard output during %s: %q", describeCall(method, params), clip(l, 200)))
			}
			if *r.ID != id {
				return c.breakWith(fmt.Sprintf("reply id %d does not match request id %d (%s)", *r.ID, id, method))
			}
			c.log.Debug("call", "id", id, "method", method, "ms", time.Since(started).Milliseconds(), "error", r.Error != nil)
			if r.Error != nil {
				return r.Error
			}
			if result != nil && len(r.Result) > 0 {
				if err := json.Unmarshal(r.Result, result); err != nil {
					return c.breakWith(fmt.Sprintf("the reply to %s has the wrong shape: %v", method, err))
				}
			}
			return nil
		case <-timer.C:
			return c.breakWith(fmt.Sprintf("timed out after %s on %s", timeout, describeCall(method, params)))
		}
	}
}

func describeCall(method string, params any) string {
	if m, ok := params.(map[string]any); ok {
		if p, ok := m["part"].(string); ok {
			return method + " " + p
		}
		if c, ok := m["component"].(string); ok {
			return method + " " + c
		}
	}
	return method
}

func exitDescription(err error, exited chan struct{}) string {
	select {
	case <-exited:
	default:
		return "closed its standard output"
	}
	var ee *exec.ExitError
	if errors.As(err, &ee) {
		return fmt.Sprintf("exited with code %d", ee.ExitCode())
	}
	if err != nil {
		return "exited: " + err.Error()
	}
	return "exited"
}

func clip(s string, n int) string {
	if len(s) > n {
		return s[:n] + "…"
	}
	return s
}

// Stop closes the adapter's standard input, waits briefly for it to exit, then kills it.
func (c *Client) Stop() {
	c.stopOnce.Do(func() {
		c.stdin.Close()
		select {
		case <-c.exited:
		case <-time.After(3 * time.Second):
			killGroup(c.cmd)
			<-c.exited
		}
	})
}

// StderrTail returns the last lines the adapter wrote to standard error.
func (c *Client) StderrTail() []string { return c.stderr.lines() }

// tail keeps the last n lines written to it.
type tail struct {
	mu      sync.Mutex
	n       int
	buf     []string
	partial string
}

func newTail(n int) *tail { return &tail{n: n} }

func (t *tail) Write(p []byte) (int, error) {
	t.mu.Lock()
	defer t.mu.Unlock()
	s := t.partial + string(p)
	parts := strings.Split(s, "\n")
	t.partial = parts[len(parts)-1]
	for _, l := range parts[:len(parts)-1] {
		t.buf = append(t.buf, l)
		if len(t.buf) > t.n {
			t.buf = t.buf[len(t.buf)-t.n:]
		}
	}
	return len(p), nil
}

func (t *tail) lines() []string {
	t.mu.Lock()
	defer t.mu.Unlock()
	out := append([]string{}, t.buf...)
	if t.partial != "" {
		out = append(out, t.partial)
	}
	return out
}
