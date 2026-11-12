#!/usr/bin/env python3
"""One JSON-RPC stream, relayed to one UIKit process on one reusable Simulator."""
import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import sys

ROOT = Path(__file__).resolve().parent

def run(*args, **kwargs):
    return subprocess.run(['xcrun', 'simctl', *args], check=True, stdout=sys.stderr, **kwargs)

def main():
    if not (ROOT/'build/device').exists():
        raise SystemExit('Run ./setup.sh first (build the app and host once).')
    device = (ROOT/'build/device').read_text().strip()
    listener = socket.socket()
    listener.bind(('127.0.0.1', 0))
    listener.listen(1)
    listener.settimeout(90)
    devices = json.loads(subprocess.check_output(['xcrun','simctl','list','devices','-j']))['devices']
    state = next(d['state'] for group in devices.values() for d in group if d['udid']==device)
    if state != 'Booted':
        run('boot', device)
    run('bootstatus', device, '-b')
    run('install', device, str(ROOT/'build/Host.app'))
    subprocess.run(['xcrun','simctl','terminate',device,'org.omnicui.todo.host'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    env = dict(os.environ, SIMCTL_CHILD_OMNICUI_PORT=str(listener.getsockname()[1]))
    launch = subprocess.Popen(['xcrun','simctl','launch','--console',device,'org.omnicui.todo.host'], env=env, stdin=subprocess.PIPE, stdout=sys.stderr, stderr=sys.stderr)
    connection, _ = listener.accept()
    listener.close()
    connection.settimeout(14)
    stream = connection.makefile('rwb', buffering=0)
    def cleanup():
        connection.close()
        launch.terminate()
        subprocess.run(['xcrun','simctl','terminate',device,'org.omnicui.todo.host'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    def interrupted(signum, _frame):
        cleanup()
        raise SystemExit(128 + signum)
    signal.signal(signal.SIGTERM, interrupted)
    signal.signal(signal.SIGINT, interrupted)
    try:
        for line in sys.stdin.buffer:
            stream.write(line)
            reply = stream.readline()
            if not reply:
                raise RuntimeError('The UIKit host exited.')
            sys.stdout.buffer.write(reply)
            sys.stdout.buffer.flush()
    finally:
        cleanup()

if __name__ == '__main__':
    main()
