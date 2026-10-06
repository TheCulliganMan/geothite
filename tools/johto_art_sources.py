"""Repository-contained, hash-verified native Blender sources; no network access."""
from pathlib import Path
import gzip
import hashlib
import json

ROOT = Path(__file__).resolve().parent.parent
SOURCE_DIR = ROOT / 'art/johto/source'
REQUIRED_SOURCES = {'characters.blend', 'connected-johto.blend', 'foliage-lod.blend',
                    'new-bark-kit.blend', 'player-house.blend'}

def read_source(source_dir, item):
    """Return verified native gzip .blend bytes from the given source directory."""
    name = Path(item['file'])
    if name.name != item['file'] or name.suffix != '.blend' or item['encoding'] != 'gzip-blend' or 'chunks' in item:
        raise ValueError('invalid native Blender source identity')
    source_dir = Path(source_dir)
    path = source_dir / name
    if not path.resolve().is_relative_to(source_dir.resolve()):
        raise ValueError('Blender source escapes its source directory')
    payload = path.read_bytes()
    if len(payload) != item['bytes'] or hashlib.sha256(payload).hexdigest() != item['sha256']:
        raise ValueError(f'compressed Blender source identity mismatch: {name}')
    raw = gzip.decompress(payload)
    if not raw.startswith(b'BLENDER') or len(raw) != item['uncompressed_bytes'] or hashlib.sha256(raw).hexdigest() != item['uncompressed_sha256']:
        raise ValueError(f'expanded Blender source identity mismatch: {name}')
    return payload

def source_manifest(source_dir):
    """Validate the catalog structure separately from mutable source bytes."""
    source_dir = Path(source_dir)
    manifest = json.loads((source_dir / 'manifest.json').read_text())
    if manifest['format_version'] != 4:
        raise ValueError('editable sources must use the native-file manifest format')
    names = [item['file'] for item in manifest['sources']]
    if len(set(names)) != len(names) or not REQUIRED_SOURCES.issubset(names):
        raise ValueError('duplicate or missing editable source identity')
    if (source_dir / 'chunks').exists():
        raise ValueError('obsolete Blender source transport chunks remain')
    for item in manifest['sources']:
        name = Path(item['file'])
        if name.name != item['file'] or name.suffix != '.blend' or item['encoding'] != 'gzip-blend' or 'chunks' in item:
            raise ValueError('invalid native Blender source identity')
    return manifest


def load_sources(source_dir=None):
    source_dir = Path(source_dir) if source_dir is not None else SOURCE_DIR
    manifest = source_manifest(source_dir)
    for item in manifest['sources']:
        yield item['file'], read_source(source_dir, item)

def store_source(path, name=None):
    """Store an authored native gzip Blender file and update its identity record."""
    path = Path(path)
    name = name or path.name
    if Path(name).name != name or not name.endswith('.blend'):
        raise ValueError('invalid Blender source filename')
    manifest = source_manifest(SOURCE_DIR)
    destination = SOURCE_DIR / name
    if not destination.resolve().is_relative_to(SOURCE_DIR.resolve()):
        raise ValueError('Blender source escapes its source directory')
    # Editing a normal tracked .blend intentionally changes its old identity.
    # Permit only the explicitly replaced item to differ; other corruption must
    # stop the operation before any source or manifest is overwritten.
    for source in manifest['sources']:
        if source['file'] != name:
            read_source(SOURCE_DIR, source)
    payload = path.read_bytes()
    if payload.startswith(b'BLENDER'):
        raw = payload
        payload = gzip.compress(raw, compresslevel=9, mtime=0)
    elif payload.startswith(b'\x28\xb5\x2f\xfd'):
        # Optional authoring dependency for Blender 4 native compressed saves.
        import io
        import zstandard
        with zstandard.ZstdDecompressor().stream_reader(io.BytesIO(payload)) as stream:
            raw = stream.read()
        payload = gzip.compress(raw, compresslevel=9, mtime=0)
    else:
        raw = gzip.decompress(payload)
    if not raw.startswith(b'BLENDER'):
        raise ValueError('not a Blender source')
    manifest_path = SOURCE_DIR / 'manifest.json'
    item = {'file': name, 'encoding': 'gzip-blend', 'bytes': len(payload),
            'sha256': hashlib.sha256(payload).hexdigest(), 'uncompressed_bytes': len(raw),
            'uncompressed_sha256': hashlib.sha256(raw).hexdigest()}
    destination.write_bytes(payload)
    manifest['sources'] = sorted([source for source in manifest['sources'] if source['file'] != name] + [item], key=lambda source: source['file'])
    manifest_path.write_text(json.dumps(manifest, indent=2) + '\n')
    list(load_sources())
    return item

if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output-dir', type=Path, default=SOURCE_DIR)
    parser.add_argument('--check', action='store_true', help='verify without copying files')
    parser.add_argument('--force', action='store_true', help='replace locally edited output files')
    parser.add_argument('--store', type=Path, help='store an original authored Blender source')
    parser.add_argument('--name', help='stable editable source name used with --store')
    args = parser.parse_args()
    if args.name and not args.store:
        parser.error('--name requires --store')
    if args.store:
        item = store_source(args.store, args.name)
        print(f"Stored {item['file']}: {item['bytes']} native bytes")
        raise SystemExit(0)
    for name, payload in load_sources():
        if not args.check:
            args.output_dir.mkdir(parents=True, exist_ok=True)
            path = args.output_dir / name
            if path.exists() and path.read_bytes() != payload and not args.force:
                raise SystemExit(f'{path} has local edits; choose another output directory or --force')
            if not path.exists() or path.read_bytes() != payload:
                path.write_bytes(payload)
        print(f'Verified {name}: {len(payload)} bytes')
