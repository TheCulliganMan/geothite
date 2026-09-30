"""Repository-contained, hash-verified editable art sources; no network access."""
from pathlib import Path
import base64
import gzip
import hashlib
import json

ROOT = Path(__file__).resolve().parent.parent
SOURCE_DIR = ROOT / 'art/johto/source'
REQUIRED_SOURCES = {'characters.blend', 'connected-johto.blend', 'foliage-lod.blend',
                    'new-bark-kit.blend', 'player-house.blend'}

def load_sources():
    manifest = json.loads((SOURCE_DIR / 'manifest.json').read_text())
    assert manifest['format_version'] == 3
    names = [item['file'] for item in manifest['sources']]
    assert len(set(names)) == len(names), 'duplicate editable source identity'
    assert REQUIRED_SOURCES.issubset(names), 'an existing editable kit is missing'
    for item in manifest['sources']:
        name = Path(item['file'])
        assert name.name == item['file'] and name.suffix == '.blend'
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

def store_source(path, name=None):
    """Publish an authored Blender source as bounded, ordinary Git text.

    Blender's gzip/Zstandard save and an uncompressed stream are accepted.
    No network transport or executable content is loaded. Existing kits and
    their identities are retained; stale chunks for just this kit are removed.
    """
    list(load_sources())  # Never replace a partially readable manifest.
    name = name or path.name
    assert Path(name).name == name and name.endswith('.blend'), name
    payload = path.read_bytes()
    if payload.startswith(b'BLENDER'):
        raw = payload
        payload = gzip.compress(raw, compresslevel=9, mtime=0)
    elif payload.startswith(b'\x28\xb5\x2f\xfd'):
        # Blender 4's native compressed saves use concatenated Zstandard
        # frames. This authoring-only dependency is unnecessary when checking
        # or reconstructing the repository's standard-library gzip sources.
        import io
        import zstandard
        with zstandard.ZstdDecompressor().stream_reader(io.BytesIO(payload)) as stream:
            raw = stream.read()
        payload = gzip.compress(raw, compresslevel=9, mtime=0)
    else:
        raw = gzip.decompress(payload)
    assert raw.startswith(b'BLENDER'), 'not a Blender source'
    manifest_path = SOURCE_DIR / 'manifest.json'
    manifest = json.loads(manifest_path.read_text())
    previous = next((item for item in manifest['sources'] if item['file'] == name), None)
    item = {
        'file': name,
        'encoding': 'gzip-blend',
        'bytes': len(payload),
        'sha256': hashlib.sha256(payload).hexdigest(),
        'uncompressed_bytes': len(raw),
        'uncompressed_sha256': hashlib.sha256(raw).hexdigest(),
        'chunks': [],
    }
    for offset in range(0, len(payload), 65536):
        part = payload[offset:offset + 65536]
        relative = f'chunks/{name}.{offset // 65536:03d}.b64'
        (SOURCE_DIR / relative).write_bytes(base64.b64encode(part))
        item['chunks'].append({
            'file': relative, 'bytes': len(part),
            'sha256': hashlib.sha256(part).hexdigest(), 'encoding': 'base64',
        })
    manifest['sources'] = sorted(
        [source for source in manifest['sources'] if source['file'] != name] + [item],
        key=lambda source: source['file'],
    )
    manifest_path.write_text(json.dumps(manifest, indent=2) + '\n')
    if previous:
        retained = {part['file'] for part in item['chunks']}
        for part in previous['chunks']:
            if part['file'] not in retained:
                (SOURCE_DIR / part['file']).unlink()
    list(load_sources())
    return item

if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output-dir', type=Path, default=SOURCE_DIR)
    parser.add_argument('--check', action='store_true', help='verify without materializing')
    parser.add_argument('--force', action='store_true', help='replace locally edited output files')
    parser.add_argument('--store', type=Path, help='encode an original authored Blender source')
    parser.add_argument('--name', help='stable editable source name used with --store')
    args = parser.parse_args()
    if args.name and not args.store:
        parser.error('--name requires --store')
    if args.store:
        item = store_source(args.store, args.name)
        print(f"Stored {item['file']}: {len(item['chunks'])} verified chunks")
        raise SystemExit(0)
    for name, payload in load_sources():
        if not args.check:
            args.output_dir.mkdir(parents=True, exist_ok=True)
            path = args.output_dir / name
            if path.exists() and path.read_bytes() != payload and not args.force:
                raise SystemExit(f'{path} has local edits; choose another output directory or --force')
            path.write_bytes(payload)
        print(f'Verified {name}: {len(payload)} bytes')
