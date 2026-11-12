#!/usr/bin/env python3
"""Protocol and real responder smoke test; no omnicui full-check budget is consumed."""
import json
import pathlib
import subprocess
import time

here = pathlib.Path(__file__).resolve().parent
process = subprocess.Popen([str(here / 'build/adapter')], cwd=here, stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
serial = 0

def rpc(method, **params):
    global serial
    serial += 1
    process.stdin.write(json.dumps(dict(jsonrpc='2.0', id=serial, method=method, params=params)) + '\n')
    process.stdin.flush()
    line = process.stdout.readline()
    assert line, f'adapter exited ({process.poll()}) during {method}'
    reply = json.loads(line)
    assert 'error' not in reply, (method, params, reply)
    return reply.get('result')

def mount(component, **props):
    rpc('mount', component=component, props=props, viewport=dict(width=1024, height=720), colorScheme='light')

def read(part, field='text'):
    return rpc('read', part=part, field=field)

def press(part):
    rpc('press', part=part)

try:
    start = time.monotonic()
    assert rpc('advertise', protocol='1')['platform']['os'] == 'macos'
    print(f'advertise startup: {(time.monotonic() - start)*1000:.0f} ms', flush=True)
    mount('todo.list')
    assert read('todo.list.row1') == 'Buy milk'
    assert read('todo.list.row2Check', 'checked') is True
    assert read('todo.list.filterAll', 'selected') is True
    rpc('focus', part='todo.list.row1Check')
    rpc('key', part='todo.list.row1Check', key='Space')
    assert read('todo.list.row1Check', 'checked') is True
    assert read('todo.list.row1Check', 'focused') is True
    press('todo.list.filterDone')
    assert read('todo.list.row1') == 'Buy milk'
    press('todo.list.add')
    assert read('todo.list.newTitle', 'focused') is True
    rpc('type', part='todo.list.newTitle', text='  Pay rent  ')
    assert read('todo.list.newTitle') == '  Pay rent  '
    rpc('key', part='todo.list.newTitle', key='Enter')
    events = rpc('events')
    assert {'name': 'added', 'fields': {'id': 't4', 'title': 'Pay rent'}} in events, events
    mount('todo.app')
    press('todo.app.row1')
    rpc('type', part='todo.app.title', text=' today')
    assert read('todo.app.title') == 'Buy milk today'
    rpc('key', key='Meta+BracketLeft')
    assert read('todo.app.row1') == 'Buy milk today'
    assert rpc('events')[-1] == {'name': 'navigated', 'fields': {'screen': 'list', 'id': ''}}
    press('todo.app.openSettings')
    rpc('focus', part='todo.app.themeDark')
    rpc('key', part='todo.app.themeDark', key='Enter')
    assert read('todo.app.themeDark', 'selected') is True
    assert read('todo.app.themeDark', 'focused') is True
    screenshot = here / 'build/smoke.png'
    shot = rpc('screenshot', path=str(screenshot))
    assert shot['regions'][0]['kind'] == 'titlebar'
    print('settings screenshot:', screenshot, flush=True)
    print('settingsBack bounds:', rpc('bounds', part='todo.app.settingsBack'), flush=True)
    mount('todo.list', seed='empty')
    samples = rpc('trace', part='todo.list.newTaskDialog', property='scale', steps=[dict(method='press', part='todo.list.add')])
    assert min(x['value'] for x in samples) < .95 and samples[-1]['value'] == 1
    print(f"dialog: {len(samples)} rendered scale samples, {samples[0]['value']:.3f} to {samples[-1]['value']:.3f}", flush=True)
    rpc('key', key='Escape')
    mount('todo.detail', taskId='t2')
    rpc('type', part='todo.detail.notes', text='Line one\nLine two')
    assert read('todo.detail.notes') == 'Line one\nLine two'
    changes = [e for e in rpc('events') if e['name'] == 'notesChanged']
    assert len(changes) == len('Line one\nLine two'), changes
    rpc('focus', part='todo.detail.done')
    rpc('key', part='todo.detail.done', key='Enter')
    assert read('todo.detail.done', 'checked') is False
    mount('todo.app', seed='empty')
    for title, expected_id in [('First', 't1'), ('Second', 't2')]:
        press('todo.app.add')
        rpc('type', part='todo.app.newTitle', text=title)
        press('todo.app.confirmAdd')
        assert {'name': 'added', 'fields': {'id': expected_id, 'title': title}} in rpc('events')
        press('todo.app.row1')
        press('todo.app.delete')
    # Dialog controls require setup, so exercise their keys without changing a contract.
    for key in ['Space', 'Enter']:
        press('todo.app.add')
        rpc('type', part='todo.app.newTitle', text='Keyboard task')
        rpc('focus', part='todo.app.confirmAdd')
        rpc('key', part='todo.app.confirmAdd', key=key)
        assert read('todo.app.row1') == 'Keyboard task'
        press('todo.app.row1'); press('todo.app.delete')
        press('todo.app.add')
        rpc('focus', part='todo.list.cancelAdd')
        rpc('key', part='todo.list.cancelAdd', key=key)
        assert read('todo.app.empty') == 'No tasks yet'
    print('smoke passed: multiline edits, per-keystroke events, monotonic IDs, dialog keys, real Cmd+[', flush=True)
finally:
    process.stdin.close()
    process.wait(timeout=10)
