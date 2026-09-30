"""Repository-contained, hash-verified editable art sources; no network access."""
from pathlib import Path
import base64
import gzip
import hashlib
import json

ROOT = Path(__file__).resolve().parent.parent
SOURCE_DIR = ROOT / 'art/johto/source'
NAMES = {'characters.blend', 'connected-johto.blend', 'foliage-lod.blend',
         'new-bark-kit.blend', 'player-house.blend'}

def load_sources():
    manifest = json.loads((SOURCE_DIR / 'manifest.json').read_text())
    assert manifest['format_version'] == 3
    assert {item['file'] for item in manifest['sources']} == NAMES
    for item in manifest['sources']:
        parts = []
        for part in item['chunks']:
            name = Path(part['file'])
            assert name.parent == Path('chunks') and name.name.startswith(item['file'] + '.')
            assert name.suffix == '.b64' and part['encoding'] == 'base64'
            payload = base64.b64decode((SOURCE_DIR / name).read_bytes(), validate=True)
            assert len(payload) == part['bytes'] <= 65536
            assert hashlib.sha256(payload).hexdigest() == part['sha256']
            parts.append(payload)
        payload = b''.join(parts)
        assert len(payload) == item['bytes']
        assert hashlib.sha256(payload).hexdigest() == item['sha256']
        raw = gzip.decompress(payload)
        assert raw.startswith(b'BLENDER')
        assert len(raw) == item['uncompressed_bytes']
        assert hashlib.sha256(raw).hexdigest() == item['uncompressed_sha256']
        yield item['file'], payload

if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output-dir', type=Path, default=SOURCE_DIR)
    parser.add_argument('--check', action='store_true', help='verify without materializing')
    parser.add_argument('--force', action='store_true', help='replace locally edited output files')
    args = parser.parse_args()
    for name, payload in load_sources():
        if not args.check:
            args.output_dir.mkdir(parents=True, exist_ok=True)
            path = args.output_dir / name
            if path.exists() and path.read_bytes() != payload and not args.force:
                raise SystemExit(f'{path} has local edits; choose another output directory or --force')
            path.write_bytes(payload)
        print(f'Verified {name}: {len(payload)} bytes')
