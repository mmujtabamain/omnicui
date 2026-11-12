#!/usr/bin/env python3
"""Supplementary protocol checks; omnicui runs the product contracts."""
import json
from pathlib import Path
import struct
import subprocess
import unittest

ROOT = Path(__file__).resolve().parent

class ProtocolTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.log = (ROOT/'build/protocol-test.log').open('w')
        cls.process = subprocess.Popen(['python3', 'adapter.py'], cwd=ROOT, stdin=subprocess.PIPE,
                                       stdout=subprocess.PIPE, stderr=cls.log, text=True)
        cls.sequence = 0

    @classmethod
    def tearDownClass(cls):
        cls.process.stdin.close()
        cls.process.wait(timeout=10)
        cls.process.stdout.close()
        cls.log.close()

    def rpc(self, method, **params):
        self.__class__.sequence += 1
        identifier = self.sequence
        self.process.stdin.write(json.dumps(dict(jsonrpc='2.0', id=identifier, method=method, params=params))+'\n')
        self.process.stdin.flush()
        reply = json.loads(self.process.stdout.readline())
        self.assertEqual(reply['id'], identifier)
        return reply

    def call(self, method, **params):
        reply = self.rpc(method, **params)
        self.assertNotIn('error', reply, reply)
        return reply['result']

    def mount(self, component, **props):
        self.call('mount', component=component, props=props, viewport=dict(width=360, height=640), colorScheme='light')

    def read(self, part, field):
        return self.call('read', part=part, field=field)

    def test_protocol_and_live_controls(self):
        ad = self.call('advertise', protocol='1')
        self.assertEqual(ad['platform']['os'], 'ios')
        self.assertTrue(ad['platform']['version'].startswith('26.'))
        self.assertIn('key', ad['cannot'])
        self.mount('todo.list')
        for part, role in [('tasksHeading','heading'),('left','status'),('row1','button'),('row1Check','checkbox'),('filterAll','button')]:
            self.assertEqual(self.read('todo.list.'+part, 'role'), role, part)
        self.assertTrue(self.read('todo.list.filterAll', 'selected'))
        self.call('press', part='todo.list.filterActive')
        self.assertTrue(self.read('todo.list.filterActive','selected'))
        bounds = self.call('bounds', part='todo.list.row1')
        self.assertGreater(bounds['height'], 0)
        self.assertGreaterEqual(bounds['x'], 0)
        self.assertLess(bounds['y'], 640)
        self.assertEqual(self.call('bounds',part='todo.list.missing')['width'], 0)
        self.assertEqual(self.rpc('read',part='todo.list.missing',field='text')['error']['code'], -32002)
        self.mount('todo.detail')
        self.assertEqual(self.read('todo.detail.title','role'), 'textbox')
        self.assertEqual(self.read('todo.detail.done','role'), 'switch')
        self.call('type', part='todo.detail.title', text=' abc')
        changes = self.call('events')
        self.assertEqual([e['fields']['title'] for e in changes], ['Buy milk ', 'Buy milk a', 'Buy milk ab', 'Buy milk abc'])
        self.assertEqual(self.call('events'), [])
        self.mount('todo.app')
        self.call('press',part='todo.app.add')
        self.assertTrue(self.read('todo.app.newTitle','focused'))
        self.call('do',name='back')
        self.assertEqual(self.read('todo.app.left','text'),'2 left')
        self.call('press',part='todo.app.add')
        self.assertTrue(self.read('todo.app.newTitle','focused'))
        self.call('press',part='todo.list.cancelAdd')
        self.assertEqual(self.call('events'), [])
        picture=ROOT/'build/protocol.png'
        result=self.call('screenshot',path=str(picture))
        self.assertIn('regions',result)
        self.assertEqual(struct.unpack('>II',picture.read_bytes()[16:24]),(360,640))
        self.assertEqual(self.rpc('mount',component='missing')['error']['code'],-32001)
        self.process.stdin.write('{broken json}\n'); self.process.stdin.flush()
        self.assertEqual(json.loads(self.process.stdout.readline())['error']['code'],-32700)

if __name__ == '__main__':
    unittest.main()
