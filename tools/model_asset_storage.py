#!/usr/bin/env python3
"""Canonical model data is ordinary, directly readable UTF-8 JSON.

Compression is solely a Cargo build detail under ignored OUT_DIR. Authoring
scripts, external editors and GitHub all read these exact source bytes.
"""
from pathlib import Path
import argparse
import hashlib
import json

MAX_MODEL_BYTES = 16 * 1024 * 1024

def sha(data):
    return hashlib.sha256(data).hexdigest()

def validate_model_bytes(raw):
    if not 0 < len(raw) <= MAX_MODEL_BYTES:
        raise ValueError('model exceeds bounded input size')
    value = json.loads(raw.decode('utf-8'))
    if not isinstance(value, dict) or 'storage' in value:
        raise ValueError('expected a raw canonical model document, not a storage envelope')
    return raw

def read_model_bytes(path):
    return validate_model_bytes(Path(path).read_bytes())

def read_model_text(path):
    return read_model_bytes(path).decode('utf-8')

def read_model_json(path):
    return json.loads(read_model_bytes(path))

def store_model_bytes(path, original):
    """Validate before writing, preserving exact authored bytes and precision."""
    validate_model_bytes(original)
    path = Path(path)
    if path.is_symlink():
        raise ValueError('refusing to replace a symlinked model')
    path.write_bytes(original)
    return {'bytes': len(original), 'sha256': sha(original)}

def validate_model(path):
    original = read_model_bytes(path)
    return len(original), len(original)

def stored_model_size(path):
    return len(read_model_bytes(path))

def validate_storage(root):
    root = Path(root)
    for pattern in ('*.json.gz', '*.json.chunks', '*.json.include.rs', '*.b64'):
        if any(root.rglob(pattern)):
            raise ValueError('encoded model wrappers remain in canonical source')
    paths = sorted(root.rglob('*.json'))
    for path in paths:
        if path.is_symlink():
            raise ValueError('symlinked canonical model')
        read_model_json(path)
    return paths

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='action', required=True)
    validate = sub.add_parser('validate'); validate.add_argument('root', type=Path)
    args = parser.parse_args()
    print(f'Validated {len(validate_storage(args.root))} raw JSON model documents')

if __name__ == '__main__':
    main()
