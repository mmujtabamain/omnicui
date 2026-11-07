#!/usr/bin/env node
// A release's notes on GitHub: its note (notes/<version>.md), then how to use each asset, with
// commands for this version.
//
//   node scripts/release-body.mjs [--version 0.1.0]     prints the title on the first line, then the body
import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { readNote } from './check-notes.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');

/** How to use each asset of `version`: one line on what it is, and a few commands. */
export function usage(version) {
  const fence = '```';
  return `### Using it

**\`omnicui_${version}_<os>_<arch>\`** is the command for one platform, one static file with nothing to install: \`linux\`, \`darwin\` (macOS) or \`windows\` (\`.exe\`), on \`amd64\` or \`arm64\`.
${fence}bash
chmod +x omnicui_${version}_darwin_arm64 && mv omnicui_${version}_darwin_arm64 /usr/local/bin/omnicui
omnicui version
omnicui check
${fence}

**\`omnicui-src-${version}.tar.gz\`** is the command's source (Go), for a project that vendors omnicui and builds it itself.
${fence}bash
mkdir -p vendor/omnicui && tar -xzf omnicui-src-${version}.tar.gz -C vendor/omnicui --strip-components=1
(cd vendor/omnicui && go build -trimpath -ldflags "-X main.Version=${version}" -o ../../bin/omnicui ./cmd/omnicui)
${fence}

**\`SHA256SUMS\`**: \`sha256sum -c SHA256SUMS\` (on macOS, \`shasum -a 256 -c SHA256SUMS\`)
`;
}

/** The release's title (its note's summary) and body. */
export function releaseBody(version) {
  const n = readNote(readFileSync(join(root, 'notes', `${version}.md`), 'utf8'));
  return { title: `omnicui ${version}: ${n.front.summary ?? ''}`.trim(), body: `${n.body.trim()}\n\n${usage(version)}` };
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) {
  const a = process.argv.slice(2);
  const version = (a.includes('--version') ? a[a.indexOf('--version') + 1] : readFileSync(join(root, 'VERSION'), 'utf8')).trim();
  const r = releaseBody(version);
  process.stdout.write(`${r.title}\n${r.body}`);
}
