#!/usr/bin/env node
// The release-notes check: strict, run on every merge request into main and by the
// release job. It fails, naming the file and line, unless:
//
//   - VERSION is a valid version (whole numbers and dots: 0.3.0; no "v", no leading zeros, no
//     suffix), and with --base-version strictly higher than the base branch's. Raising an earlier
//     number (0.3.1 -> 0.4.0) passes with a warning, so a big step is a deliberate one.
//   - notes/<VERSION>.md exists, named exactly as VERSION writes the version.
//   - Every note, old and new, starts with front matter giving its version (its file's name), a
//     date (YYYY-MM-DD, a day that exists) and a summary (one sentence, at most 140 characters).
//
// Warnings (it still passes): a heading other than New, Changed or Fixed, a section with nothing
// in it, a note with no sections, a key written twice, a .md file not named for a version.
//
//   node scripts/check-notes.mjs [--version-file VERSION] [--notes-dir notes] [--base-version 0.3.0]
//   node scripts/check-notes.mjs --notes-for 0.3.0      prints that note's summary and body, for a release
//
// Exit codes: 0 passes (maybe with warnings), 1 fails, 2 a usage mistake. No dependencies.

import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { join, relative } from 'node:path';
import { pathToFileURL } from 'node:url';

export const SECTIONS = ['New', 'Changed', 'Fixed'];
export const SUMMARY_MAX = 140;

/** Why `s` is not a version, or null when it is one. */
export function versionProblem(s) {
  if (!s) return 'it is empty';
  if (/^v/i.test(s)) return `it starts with "v"; write the bare number, ${s.slice(1)}`;
  if (/[-+]/.test(s)) return 'it has a suffix; a version is whole numbers and dots only';
  if (!/^\d+(\.\d+)*$/.test(s)) return 'a version is whole numbers separated by dots, such as 0.3.0';
  if (s.split('.').some((p) => p.length > 1 && p.startsWith('0'))) return 'a number in it has a leading zero';
  return null;
}

/** Compares two versions number by number (the shorter padded with zeros): -1, 0 or 1. */
export function compareVersions(a, b) {
  const x = a.split('.').map(BigInt);
  const y = b.split('.').map(BigInt);
  for (let i = 0; i < Math.max(x.length, y.length); i++) {
    const p = x[i] ?? 0n;
    const q = y[i] ?? 0n;
    if (p !== q) return p < q ? -1 : 1;
  }
  return 0;
}

/** The step from `base` to `head`: 'last' (the last number raised) or 'earlier', for a higher version. */
export function bumpLevel(base, head) {
  const x = base.split('.').map(BigInt);
  const y = head.split('.').map(BigInt);
  const n = Math.max(x.length, y.length);
  for (let i = 0; i < n; i++) if ((y[i] ?? 0n) !== (x[i] ?? 0n)) return i === n - 1 ? 'last' : 'earlier';
  return 'same';
}

/** The next version after `v`: its last number plus one. */
export const nextVersion = (v) => v.split('.').map((p, i, all) => (i === all.length - 1 ? String(BigInt(p) + 1n) : p)).join('.');

export function isDate(s) {
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(s);
  if (!m) return false;
  const d = new Date(Date.UTC(+m[1], +m[2] - 1, +m[3]));
  return d.getUTCFullYear() === +m[1] && d.getUTCMonth() === +m[2] - 1 && d.getUTCDate() === +m[3];
}

/** Characters as a reader counts them: an emoji or an accented letter is one. */
export const summaryLength = (s) => [...s.normalize('NFC')].length;

/** A note: its front matter (with each key's line), its body, and the problems reading it. */
export function readNote(text) {
  const lines = text.replace(/^﻿/, '').split(/\r?\n/);
  const front = {};
  const at = {};
  const errors = [];
  const warnings = [];
  if (lines[0]?.trim() !== '---') return { front, at, body: lines.join('\n'), errors: [{ line: 1, message: 'it does not start with front matter: a "---" line, then version, date and summary, then "---"' }], warnings };
  let end = -1;
  for (let i = 1; i < lines.length; i++) {
    const l = lines[i];
    if (l.trim() === '---') {
      end = i;
      break;
    }
    if (!l.trim() || l.trim().startsWith('#')) continue;
    const m = /^([A-Za-z][\w-]*)\s*:\s*(.*)$/.exec(l);
    if (!m) {
      errors.push({ line: i + 1, message: `line ${i + 1} of the front matter is not "key: value"; found ${quote(l)}` });
      continue;
    }
    let v = m[2].trim();
    if (v.length > 1 && ((v.startsWith('"') && v.endsWith('"')) || (v.startsWith("'") && v.endsWith("'")))) v = v.slice(1, -1);
    if (m[1] in front) warnings.push({ line: i + 1, message: `${m[1]} is written twice; the last one counts` });
    front[m[1]] = v;
    at[m[1]] = i + 1;
  }
  if (end < 0) errors.push({ line: 1, message: 'its front matter is never closed: add a "---" line after the summary' });
  return { front, at, body: end < 0 ? '' : lines.slice(end + 1).join('\n').replace(/^\n+/, ''), bodyLine: end + 2, errors, warnings };
}

/** A quoted bit of a file, on one line and short, for a message. */
export function quote(s) {
  const one = String(s).replace(/\s+/g, ' ').trim();
  return JSON.stringify(one.length > 40 ? one.slice(0, 39) + '…' : one);
}

/** A note to copy when the one for `version` is missing. */
export function template(version, date = new Date().toISOString().slice(0, 10)) {
  return `---\nversion: ${version}\ndate: ${date}\nsummary: One sentence on the biggest change, for omnicui's users (at most ${SUMMARY_MAX} characters).\n---\n## New\n- **What it is.** What someone can now do, and how.\n## Changed\n- **What changed.** A breaking change comes first and says what to do instead.\n## Fixed\n- What went wrong, as the author saw it.\n`;
}

/** Checks VERSION and every note. Returns the problems, each with a file and line. */
export function checkNotes({ root = process.cwd(), versionFile = 'VERSION', notesDir = 'notes', baseVersion } = {}) {
  const errors = [];
  const warnings = [];
  const err = (file, line, message) => errors.push({ file, line, message });
  const warn = (file, line, message) => warnings.push({ file, line, message });
  const vPath = join(root, versionFile);
  let version = null;
  if (!existsSync(vPath)) err(versionFile, 1, `there is no ${versionFile} file: add one holding the version, such as 0.3.0`);
  else {
    const raw = readFileSync(vPath, 'utf8').replace(/^﻿/, '').trim();
    const why = versionProblem(raw);
    if (why) err(versionFile, 1, `${versionFile} is ${quote(raw)}, which is not a valid version (${why})`);
    else version = raw;
  }
  if (version && baseVersion !== undefined) {
    const why = versionProblem(baseVersion);
    if (why) throw new UsageError(`--base-version ${quote(baseVersion)} is not a version (${why})`);
    if (compareVersions(version, baseVersion) <= 0) err(versionFile, 1, `${versionFile} is ${version}, which is not higher than main's ${baseVersion}: every merge raises it. The next one would be ${nextVersion(baseVersion)}`);
    else if (bumpLevel(baseVersion, version) === 'earlier') warn(versionFile, 1, `${baseVersion} to ${version} raises an earlier number: make sure the big step is meant (a breaking change)`);
  }
  const dir = join(root, notesDir);
  const files = existsSync(dir) ? readdirSync(dir).filter((f) => f.endsWith('.md')).sort() : [];
  const named = new Map();
  for (const f of files) {
    const rel = `${notesDir}/${f}`;
    const stem = f.slice(0, -3);
    if (f === 'README.md') continue;
    if (versionProblem(stem)) {
      warn(rel, 1, `${f} is not named for a version, so it is not a release note; name it <version>.md, such as 0.3.0.md`);
      continue;
    }
    const n = readNote(readFileSync(join(dir, f), 'utf8'));
    for (const e of n.errors) err(rel, e.line, `${rel} cannot be read: ${e.message}`);
    for (const w of n.warnings) warn(rel, w.line, w.message);
    if (n.errors.length) continue;
    const { front, at } = n;
    if (!front.version) err(rel, 1, `${rel} has no version: add "version: ${stem}" to its front matter`);
    else if (front.version !== stem) err(rel, at.version, compareVersions(front.version, stem) === 0 && !versionProblem(front.version) ? `${rel} says version ${quote(front.version)}, the same version written differently; it must say ${stem}` : `${rel} says version ${quote(front.version)}; it must say ${stem}, its file's name`);
    if (!front.date) err(rel, 1, `${rel} has no date: add "date: YYYY-MM-DD", the day it ships`);
    else if (!isDate(front.date)) err(rel, at.date, `${rel} has the date ${quote(front.date)}; write it as YYYY-MM-DD, a day that exists`);
    if (!front.summary) err(rel, 1, `${rel} has no summary: add "summary: …", one sentence of at most ${SUMMARY_MAX} characters`);
    else if (summaryLength(front.summary) > SUMMARY_MAX) err(rel, at.summary, `${rel}'s summary is ${summaryLength(front.summary)} characters; it can be at most ${SUMMARY_MAX}. The detail belongs in the sections`);
    const heads = [...n.body.matchAll(/^##\s+(.+?)\s*$/gm)].map((m) => ({ name: m[1], index: m.index }));
    if (!heads.length && !n.body.trim()) warn(rel, n.bodyLine ?? 1, `${rel} has no sections: a maintenance release may have none, but say so in its summary`);
    heads.forEach((h, i) => {
      const line = (n.bodyLine ?? 1) + n.body.slice(0, h.index).split('\n').length - 1;
      if (!SECTIONS.includes(h.name)) warn(rel, line, `the heading "## ${h.name}" is not one of ${SECTIONS.map((s) => `## ${s}`).join(', ')}`);
      const content = n.body.slice(h.index, heads[i + 1]?.index ?? n.body.length).split('\n').slice(1).join('\n');
      if (!content.trim()) warn(rel, line, `"## ${h.name}" has nothing in it: leave the section out`);
    });
    named.set(stem, rel);
  }
  if (version && !named.has(version) && !errors.some((e) => e.file === `${notesDir}/${version}.md`)) {
    const near = files.find((f) => f !== `${version}.md` && f.replace(/^v/i, '').replace(/\.(md|markdown)$/, '') === version);
    err(`${notesDir}/${version}.md`, 1, `there are no release notes for ${version}: add ${notesDir}/${version}.md${near ? ` (${notesDir}/${near} is there: name it ${version}.md)` : ''}. Start from:\n\n${template(version)}`);
  }
  return { version, errors, warnings };
}

export class UsageError extends Error {}

function args(argv) {
  const out = {};
  const known = ['--version-file', '--notes-dir', '--base-version', '--notes-for'];
  for (let i = 0; i < argv.length; i++) {
    const [k, eq] = argv[i].split(/=(.*)/s);
    if (!known.includes(k)) throw new UsageError(`unknown option ${argv[i]}`);
    const v = eq !== undefined ? eq : argv[++i];
    if (v === undefined) throw new UsageError(`${k} needs a value`);
    out[k.slice(2)] = v;
  }
  return out;
}

function main(argv) {
  const usage = 'usage: node scripts/check-notes.mjs [--version-file VERSION] [--notes-dir notes] [--base-version 0.3.0] | --notes-for <version>';
  let a;
  try {
    a = args(argv);
  } catch (e) {
    console.error(`${e.message}\n${usage}`);
    return 2;
  }
  const notesDir = a['notes-dir'] ?? 'notes';
  if (a['notes-for']) {
    const f = join(process.cwd(), notesDir, `${a['notes-for']}.md`);
    if (!existsSync(f)) {
      console.error(`there is no ${relative(process.cwd(), f)}`);
      return 1;
    }
    const n = readNote(readFileSync(f, 'utf8'));
    process.stdout.write(`${n.front.summary ?? ''}\n\n${n.body.trim()}\n`);
    return 0;
  }
  let r;
  try {
    r = checkNotes({ versionFile: a['version-file'], notesDir, baseVersion: a['base-version'] });
  } catch (e) {
    if (e instanceof UsageError) {
      console.error(`${e.message}\n${usage}`);
      return 2;
    }
    throw e;
  }
  for (const e of r.errors) console.log(`error   ${e.file}:${e.line}  ${e.message}`);
  for (const w of r.warnings) console.log(`warning ${w.file}:${w.line}  ${w.message}`);
  if (!r.errors.length) console.log(`release notes: ${r.version} has its note${r.warnings.length ? `, with ${r.warnings.length} warning${r.warnings.length > 1 ? 's' : ''}` : ''}`);
  return r.errors.length ? 1 : 0;
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) process.exit(main(process.argv.slice(2)));
export { main };
