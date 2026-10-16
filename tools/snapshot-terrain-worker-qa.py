#!/usr/bin/env python3
"""Capture ignored, content-verified QA build inputs without pausing editors.

APFS clonefile copies data on write. Source hashes before/after capture and the
snapshot hash must agree; known changes/missing inputs cause a bounded retry.
No target, ignored pack, registry, .git or .local trees are copied.
"""
import argparse
import ctypes
import hashlib
import json
import os
from pathlib import Path
import shutil
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_DESTINATION = ROOT / '.local/terrain-worker-qa-source'
INPUTS = ['Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', '.cargo/config.toml',
          'pret-source.lock.json', 'crates', 'modpacks', 'web-client',
          'assets/terrain', 'art/battles/arena-palettes.json',
          'assets/open-models/interiors/bed_red.mesh.json',
          'assets/open-models/interiors/bookcase.mesh.json',
          'assets/open-models/interiors/chair.mesh.json',
          'assets/open-models/interiors/domestic_table.mesh.json',
          'assets/open-models/interiors/television.mesh.json']
EXCLUDED = {'.git', '.local', 'target', 'content-packs', 'node_modules', '__pycache__'}


def files(root):
    result = []
    for name in INPUTS:
        path = root / name
        if not path.exists():
            raise FileNotFoundError(path)
        if path.is_file():
            result.append(path)
            continue
        for directory, folders, names in os.walk(path, followlinks=False):
            folders[:] = sorted(folder for folder in folders if folder not in EXCLUDED)
            for folder in folders:
                if (Path(directory) / folder).is_symlink():
                    raise ValueError('QA build inputs may not reference mutable external symlinks')
            for filename in sorted(names):
                if filename == '.DS_Store' or filename.endswith(('.crystalpack', '.crystalsave')):
                    continue
                result.append(Path(directory) / filename)
    result.sort()
    if any(path.is_symlink() for path in result):
        raise ValueError('QA build inputs may not reference mutable external symlinks')
    return result


def fingerprint(root):
    digest = hashlib.sha256()
    length = 0
    paths = files(root)
    for path in paths:
        before = path.stat()
        name = path.relative_to(root).as_posix().encode()
        digest.update(len(name).to_bytes(8, 'little'))
        digest.update(name)
        digest.update(before.st_size.to_bytes(8, 'little'))
        with path.open('rb') as stream:
            while chunk := stream.read(1024 * 1024):
                digest.update(chunk)
                length += len(chunk)
        after = path.stat()
        if (before.st_size, before.st_mtime_ns) != (after.st_size, after.st_mtime_ns):
            raise RuntimeError('build input changed while hashing')
    return digest.hexdigest(), length, [p.relative_to(root) for p in paths]


def clone_file(source, destination):
    destination.parent.mkdir(parents=True, exist_ok=True)
    if sys.platform == 'darwin':
        library = ctypes.CDLL('/usr/lib/libSystem.B.dylib', use_errno=True)
        clone = library.clonefile
        clone.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_int]
        clone.restype = ctypes.c_int
        if clone(os.fsencode(source), os.fsencode(destination), 0):
            error = ctypes.get_errno()
            raise OSError(error, os.strerror(error), str(source))
    else:
        shutil.copy2(source, destination)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--destination', type=Path, default=DEFAULT_DESTINATION)
    destination = parser.parse_args().destination.resolve()
    if destination == ROOT / '.local' or not destination.is_relative_to(ROOT / '.local'):
        raise ValueError("QA source snapshots must be inside this checkout's ignored .local directory")
    destination.parent.mkdir(exist_ok=True)
    for attempt in range(1, 6):
        temporary = Path(tempfile.mkdtemp(prefix='terrain-worker-qa-capture-', dir=destination.parent))
        try:
            before, length, paths = fingerprint(ROOT)
            for path in paths:
                clone_file(ROOT / path, temporary / path)
            after, _, _ = fingerprint(ROOT)
            captured, _, _ = fingerprint(temporary)
            if before != after or before != captured:
                raise RuntimeError('build inputs changed during APFS capture')
            previous = destination.with_name(destination.name + '.previous')
            if previous.exists():
                raise RuntimeError('previous QA snapshot still exists; refusing to replace it')
            if destination.exists():
                destination.rename(previous)
            temporary.rename(destination)
            if previous.exists():
                shutil.rmtree(previous)
            print(json.dumps({'path': str(destination), 'files': len(paths), 'bytes': length,
                              'sha256': captured, 'attempts': attempt}))
            return
        except (FileNotFoundError, RuntimeError) as error:
            if temporary.exists():
                shutil.rmtree(temporary)
            print(f'QA snapshot attempt {attempt} rejected: {error}', file=sys.stderr)
            if attempt == 5:
                raise
        except BaseException:
            if temporary.exists():
                shutil.rmtree(temporary)
            raise


if __name__ == '__main__':
    main()
