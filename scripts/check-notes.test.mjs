// scripts/check-notes.mjs: VERSION and the release notes, as CI checks them. Run: node --test scripts/
import { describe, it } from 'node:test';
import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { bumpLevel, checkNotes, compareVersions, isDate, nextVersion, readNote, summaryLength, versionProblem } from './check-notes.mjs';

/** The few matchers these tests use, on node:assert. */
const expect = (a) => ({
  toBe: (b) => assert.equal(a, b),
  toEqual: (b) => assert.deepEqual(a, b),
  toBeNull: () => assert.equal(a, null),
  toContain: (b) => assert.ok(String(a).includes(b), `${JSON.stringify(a)} should contain ${JSON.stringify(b)}`),
  toMatch: (re) => assert.match(String(a), re),
  not: { toBeNull: () => assert.notEqual(a, null) },
});

const root = fileURLToPath(new URL('..', import.meta.url));

/** A scratch repo: VERSION and notes/<name> files. */
function repo(version, notes) {
  const dir = mkdtempSync(join(tmpdir(), 'omnicui-notes-'));
  if (version !== null) writeFileSync(join(dir, 'VERSION'), version + '\n');
  mkdirSync(join(dir, 'notes'));
  for (const [k, v] of Object.entries(notes)) writeFileSync(join(dir, 'notes', k), v);
  return dir;
}
const note = (v, extra = '', body = '## Fixed\n- Something no longer goes wrong.\n') => `---\nversion: ${v}\ndate: 2026-11-07\nsummary: A short summary.\n${extra}---\n${body}`;
const codes = (r) => r.errors.map((e) => e.message);

describe('versions', () => {
  it('are whole numbers and dots, written one way', () => {
    expect(versionProblem('0.3.0')).toBeNull();
    expect(versionProblem('1')).toBeNull();
    expect(versionProblem('v0.3.0')).toContain('starts with "v"');
    expect(versionProblem('0.03')).toContain('leading zero');
    expect(versionProblem('0.3-beta')).toContain('suffix');
    expect(versionProblem('0..3')).not.toBeNull();
  });
  it('compare number by number, never as text', () => {
    expect(compareVersions('0.3.10', '0.3.9')).toBe(1);
    expect(compareVersions('1', '1.0.0')).toBe(0);
    expect(compareVersions('1.99999999999999999999', '1.99999999999999999998')).toBe(1);
    expect(bumpLevel('0.3.1', '0.3.2')).toBe('last');
    expect(bumpLevel('0.3.1', '0.4.0')).toBe('earlier');
    expect(nextVersion('0.3.9')).toBe('0.3.10');
  });
  it('dates must exist, and summaries count characters as a reader does', () => {
    expect(isDate('2026-02-28')).toBe(true);
    expect(isDate('2026-02-30')).toBe(false);
    expect(summaryLength('é'.normalize('NFD').repeat(3))).toBe(3);
  });
});

describe('the check', () => {
  it('passes on this repo, and fails a merge that does not raise VERSION', () => {
    expect(codes(checkNotes({ root }))).toEqual([]);
    const v = checkNotes({ root }).version;
    const same = checkNotes({ root, baseVersion: v });
    expect(codes(same).join('\n')).toContain(`The next one would be ${nextVersion(v)}`);
  });
  it('warns on a big step, and fails a lower version', () => {
    const dir = repo('0.4.0', { '0.4.0.md': note('0.4.0') });
    const r = checkNotes({ root: dir, baseVersion: '0.3.1' });
    expect(r.errors).toEqual([]);
    expect(r.warnings[0].message).toContain('raises an earlier number');
    expect(codes(checkNotes({ root: dir, baseVersion: '0.5.0' }))[0]).toContain('not higher than');
  });
  it('fails with a template when the note is missing, and names a near miss', () => {
    const r = checkNotes({ root: repo('0.3.1', { 'v0.3.1.md': note('0.3.1') }) });
    expect(codes(r)[0]).toContain('add notes/0.3.1.md (notes/v0.3.1.md is there: name it 0.3.1.md)');
    expect(codes(r)[0]).toContain('version: 0.3.1\ndate: ');
    expect(r.warnings[0].message).toContain('not named for a version');
  });
  it('fails a bad VERSION', () => {
    expect(codes(checkNotes({ root: repo('v0.3.0', {}) }))[0]).toContain('is "v0.3.0", which is not a valid version');
    expect(codes(checkNotes({ root: repo(null, {}) }))[0]).toContain('there is no VERSION file');
  });
  it('fails any note, old or new, with bad front matter, naming the line', () => {
    const r = checkNotes({
      root: repo('0.3.2', {
        '0.3.0.md': '## New\n- No front matter.\n',
        '0.3.1.md': '---\nversion: 0.3.1\ndate: 2026-02-30\nsummary: ' + 'x'.repeat(141) + '\n---\n',
        '0.3.2.md': '---\nversion: 0.3.20\ndate: 2026-11-07\n---\n## New\n- x\n',
      }),
    });
    const m = r.errors.map((e) => `${e.file}:${e.line} ${e.message}`).join('\n');
    expect(m).toContain('notes/0.3.0.md:1 notes/0.3.0.md cannot be read: it does not start with front matter');
    expect(m).toContain('notes/0.3.1.md:3 notes/0.3.1.md has the date "2026-02-30"');
    expect(m).toContain('notes/0.3.1.md:4 notes/0.3.1.md\'s summary is 141 characters');
    expect(m).toContain('notes/0.3.2.md:2 notes/0.3.2.md says version "0.3.20"; it must say 0.3.2');
    expect(m).toContain('notes/0.3.2.md has no summary');
  });
  it('reads CRLF, a BOM, quotes and colons in values, and only warns on odd headings', () => {
    const text = '﻿---\r\nversion: "0.3.0"\r\ndate: 2026-11-07\r\nsummary: Slots: only through containers.\r\n---\r\n## Added\r\n- x\r\n## Fixed\r\n';
    const n = readNote(text);
    expect(n.front.summary).toBe('Slots: only through containers.');
    const r = checkNotes({ root: repo('0.3.0', { '0.3.0.md': text }) });
    expect(r.errors).toEqual([]);
    expect(r.warnings.map((w) => w.message).join('\n')).toMatch(/"## Added" is not one of[\s\S]*"## Fixed" has nothing in it/);
  });
});
