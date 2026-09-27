#!/usr/bin/env python3
"""Prepare a verified Flygon candidate for Vercel; never bundle game packs."""
import argparse
import base64
import hashlib
import json
from pathlib import Path
import re
import shutil

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('candidate', type=Path)
parser.add_argument('output', type=Path)
args = parser.parse_args()
source, out = args.candidate.resolve(), args.output.resolve()
if out.exists():
    parser.error('Choose a new output directory')
manifest = json.loads((source / 'flygon-release.json').read_text())
# Verify every source file and reject unlisted files before copying anything.
actual = set()
for path in source.rglob('*'):
    if path.is_symlink():
        parser.error('Symlinks are not release assets')
    if path.is_file():
        actual.add(path.relative_to(source).as_posix())
if actual != set(manifest['files']) | {'flygon-release.json'}:
    parser.error('Candidate inventory mismatch')
for name, expected in manifest['files'].items():
    path = source / name
    if not path.resolve().is_relative_to(source):
        parser.error('Unsafe release path')
    if path.stat().st_size != expected['bytes'] or hashlib.sha256(path.read_bytes()).hexdigest() != expected['sha256']:
        parser.error(f'Candidate hash mismatch: {name}')
    if '.crystalpack' in name or path.suffix in {'.gb', '.gbc'}:
        parser.error('Game content must remain external')
out.mkdir(parents=True)
for name in manifest['files']:
    # Avoid stale precompressed copies of updated UI and raw release inventories.
    if name.endswith('.gz'):
        continue
    target = out / name
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source / name, target)
game = (out / 'index.html').read_text()
# The embedded runtime remains addressable, but direct visits return to Flygon.
guard = "if (window.top === window.self) location.replace('/');"
game = game.replace('<head>', '<head><script>' + guard + '</script>', 1)
(out / 'embedded-game.html').write_text(game)
flygon = (out / 'flygon.html').read_text().replace('src="./?multiplayer=off&flygon=1"', 'src="./embedded-game.html?multiplayer=off&flygon=1"')
(out / 'index.html').write_text(flygon)
(out / 'flygon.html').write_text(flygon)
hashes = set()
for page in (game, flygon):
    for attributes, body in re.findall(r'<script\b([^>]*)>(.*?)</script>', page, re.S | re.I):
        if not re.search(r'\bsrc\s*=', attributes):
            hashes.add("'sha256-" + base64.b64encode(hashlib.sha256(body.encode()).digest()).decode() + "'")
csp = "; ".join([
    "default-src 'self'", "script-src 'self' 'wasm-unsafe-eval' " + ' '.join(sorted(hashes)),
    "style-src 'self' 'unsafe-inline'", "img-src 'self' data: blob:",
    "connect-src 'self'", "worker-src 'self'", "media-src 'self' blob:",
    "frame-src 'self'", "frame-ancestors 'self'", "object-src 'none'",
    "base-uri 'self'", "form-action 'none'",
])
headers = {
    'Content-Security-Policy': csp,
    'X-Content-Type-Options': 'nosniff', 'X-Frame-Options': 'SAMEORIGIN',
    'Referrer-Policy': 'no-referrer',
    'Permissions-Policy': 'camera=(), microphone=(), geolocation=(), payment=(), usb=(), tools=(self)',
    'Cache-Control': 'public, max-age=0, must-revalidate',
}
config = {
    'framework': None,
    'headers': [{'source': '/(.*)', 'headers': [{'key': k, 'value': v} for k, v in headers.items()]}],
    'rewrites': [
        {'source': '/flygon', 'destination': '/index.html'},
        {'source': '/flygon/', 'destination': '/index.html'},
        # Explicit external endpoints only: no generic API or filesystem proxy.
        {'source': '/realtime-clock.browser.crystalpack', 'destination': 'https://geothite.ryanculligan.com/realtime-clock.browser.crystalpack'},
        {'source': '/v1/clock', 'destination': 'https://geothite.ryanculligan.com/v1/clock'},
    ],
}
(out / 'vercel.json').write_text(json.dumps(config, indent=2) + '\n')
print(f'Prepared {out}; game pack and server clock stay on the existing host')
