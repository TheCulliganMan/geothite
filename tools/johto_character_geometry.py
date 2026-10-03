#!/usr/bin/env python3
"""Lossless, deterministic shared geometry for the authored human character kit.

No vertex welding, float rounding, quantization, reordering, normal changes or
material changes are permitted here. Deduplication compares complete canonical
position/normal/index arrays. Colors remain on each original primitive instance.
"""
import argparse
import copy
import hashlib
import json
import os
import struct
import tempfile
from pathlib import Path
from model_asset_storage import read_model_bytes, read_model_json, read_model_text, validate_storage

LIBRARY_FILE = 'shared.geometry.json'
GEOMETRY_KEYS = ('positions', 'normals', 'indices')


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), allow_nan=False).encode()


def digest(value):
    return hashlib.sha256(canonical(value)).hexdigest()


def library_payload(library):
    return {'version': library['version'], 'geometries': library['geometries']}


def validate_library(library):
    if set(library) != {'version', 'id', 'geometries'} or library['version'] != 1:
        raise ValueError('unsupported character geometry library')
    if not library['geometries'] or library['id'] != digest(library_payload(library)):
        raise ValueError('character geometry library identity mismatch')
    keys = []
    for geometry in library['geometries']:
        if set(geometry) != set(GEOMETRY_KEYS):
            raise ValueError('unexpected shared geometry field')
        keys.append(digest(geometry))
    if keys != sorted(set(keys)):
        raise ValueError('geometry table must be unique and SHA-256 ordered')
    return library


def load_library(directory):
    path = Path(directory) / LIBRARY_FILE
    return validate_library(read_model_json(path)) if path.exists() else None


def expand_rig(model, library=None):
    expanded = copy.deepcopy(model)
    version = model.get('version')
    if version == 1:
        if 'geometry_library' in model:
            raise ValueError('inline rig may not identify a shared library')
        for joint in model['joints']:
            for primitive in joint['primitives']:
                if set(primitive) != {*GEOMETRY_KEYS, 'base_color'}:
                    raise ValueError('invalid inline primitive schema')
        return expanded
    if version != 2 or library is None or model.get('geometry_library') != library['id']:
        raise ValueError('unsupported rig or mismatched character geometry library')
    expanded['version'] = 1
    del expanded['geometry_library']
    for joint in expanded['joints']:
        primitives = []
        for primitive in joint['primitives']:
            if set(primitive) != {'geometry', 'base_color'}:
                raise ValueError('invalid shared primitive schema')
            index = primitive['geometry']
            if type(index) is not int or not 0 <= index < len(library['geometries']):
                raise ValueError('invalid character geometry reference')
            geometry = copy.deepcopy(library['geometries'][index])
            geometry['base_color'] = primitive['base_color']
            primitives.append(geometry)
        joint['primitives'] = primitives
    return expanded


def f32_contract_digest(model):
    """Hash exact runtime float bits, integer indices, ordering and joint bindings."""
    h = hashlib.sha256()
    def word(value):
        h.update(struct.pack('<I', value))
    def string(value):
        value = value.encode(); word(len(value)); h.update(value)
    def floats(values):
        word(len(values))
        h.update(struct.pack('<' + 'f' * len(values), *values))
    string(model['name']); string(model['coordinate_system'])
    word(len(model['joints']))
    for joint in model['joints']:
        string(joint['name']); h.update(struct.pack('<i', -1 if joint['parent'] is None else joint['parent']))
        floats(joint['translation']); word(len(joint['primitives']))
        for primitive in joint['primitives']:
            floats(primitive['positions']); floats(primitive['normals']); floats(primitive['base_color'])
            word(len(primitive['indices']))
            h.update(struct.pack('<' + 'I' * len(primitive['indices']), *primitive['indices']))
    return h.hexdigest()


def compact_models(models):
    """Return a stable table and v2 rigs, preserving primitive/joint ordering."""
    unique = {}
    for model in models.values():
        if model['version'] != 1:
            raise ValueError('compact_models requires expanded inline rigs')
        for joint in model['joints']:
            for primitive in joint['primitives']:
                geometry = {key: primitive[key] for key in GEOMETRY_KEYS}
                key = digest(geometry)
                if key in unique and canonical(unique[key]) != canonical(geometry):
                    raise ValueError('shared geometry digest collision')
                unique[key] = geometry
    keys = sorted(unique)
    indices = {key: index for index, key in enumerate(keys)}
    library = {'version': 1, 'geometries': [unique[key] for key in keys]}
    library['id'] = digest(library)
    compact = {}
    for name, model in sorted(models.items()):
        packed = copy.deepcopy(model)
        packed['version'] = 2
        packed['geometry_library'] = library['id']
        for joint in packed['joints']:
            joint['primitives'] = [
                {'geometry': indices[digest({key: p[key] for key in GEOMETRY_KEYS})],
                 'base_color': p['base_color']} for p in joint['primitives']]
        expanded = expand_rig(packed, library)
        if canonical(expanded) != canonical(model) or f32_contract_digest(expanded) != f32_contract_digest(model):
            raise ValueError(f'lossless round-trip failed for {name}')
        compact[name] = packed
    return validate_library(library), compact


def read_models(directory):
    directory = Path(directory)
    library = load_library(directory)
    files = sorted(directory.glob('*.rig.json'))
    if not files:
        raise ValueError('no authored rigs found')
    return {p.name: expand_rig(read_model_json(p), library) for p in files}


def write_snapshot(directory, payloads):
    """Fully stage bytes, then replace individual files; library always comes first.

    Runtime importers accept both schemas. Publication owners can atomically
    exchange the complete model directory once their staged snapshot is verified.
    """
    directory = Path(directory); directory.mkdir(parents=True, exist_ok=True)
    if LIBRARY_FILE in payloads:
        extras = {p.name for p in directory.glob('*.rig.json')} - payloads.keys()
        if extras:
            raise ValueError(f'destination contains rigs outside this complete snapshot: {sorted(extras)}')
    with tempfile.TemporaryDirectory(prefix='.geometry-stage-', dir=directory) as temp:
        temp = Path(temp)
        for name, payload in payloads.items():
            (temp / name).write_bytes(canonical(payload))
        for name in sorted(payloads, key=lambda name: (name != LIBRARY_FILE, name)):
            os.replace(temp / name, directory / name)


def compare_directories(before, after):
    originals, results = read_models(before), read_models(after)
    if originals.keys() != results.keys():
        raise ValueError('character set changed')
    records = {}
    for name in originals:
        left, right = originals[name], results[name]
        if canonical(left) != canonical(right):
            raise ValueError(f'canonical arrays/materials/joints changed: {name}')
        old_bits, new_bits = f32_contract_digest(left), f32_contract_digest(right)
        if old_bits != new_bits:
            raise ValueError(f'f32 or index bits changed: {name}')
        records[name] = {'canonical_before': digest(left), 'canonical_after': digest(right),
                         'f32_contract_before': old_bits, 'f32_contract_after': new_bits,
                         'before_file_sha256': hashlib.sha256((Path(before) / name).read_bytes()).hexdigest(),
                         'after_file_sha256': hashlib.sha256((Path(after) / name).read_bytes()).hexdigest(),
                         'before_bytes': (Path(before) / name).stat().st_size,
                         'after_bytes': (Path(after) / name).stat().st_size}
    old_size = sum((Path(before) / name).stat().st_size for name in records)
    before_library = Path(before) / LIBRARY_FILE
    if before_library.exists(): old_size += before_library.stat().st_size
    library = load_library(after)
    new_size = sum((Path(after) / name).stat().st_size for name in records)
    if library is not None:
        new_size += (Path(after) / LIBRARY_FILE).stat().st_size
    return {'rigs': len(records), 'before_bytes': old_size, 'after_bytes': new_size,
            'saved_bytes': old_size - new_size, 'unique_geometries': len(library['geometries']) if library else None,
            'library_id': library['id'] if library else None,
            'library_file_sha256': hashlib.sha256(read_model_bytes(Path(after) / LIBRARY_FILE)).hexdigest() if library else None,
            'equivalence': records}


def pack_directory(source, destination):
    models = read_models(source)
    library, compact = compact_models(models)
    write_snapshot(destination, {LIBRARY_FILE: library, **compact})
    return library, compact


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    for name in ('pack', 'expand', 'verify'):
        command = commands.add_parser(name)
        command.add_argument('source', type=Path)
        command.add_argument('destination', type=Path)
        if name == 'verify': command.add_argument('--report', type=Path)
    args = parser.parse_args()
    if args.command == 'pack':
        library, compact = pack_directory(args.source, args.destination)
        print(f'Packed {len(compact)} rigs with {len(library["geometries"])} exact shared geometries')
    elif args.command == 'expand':
        write_snapshot(args.destination, read_models(args.source))
        print(f'Expanded authored rigs to {args.destination}')
    else:
        report = compare_directories(args.source, args.destination)
        if args.report: args.report.write_text(json.dumps(report, indent=2) + '\n')
        print(f'Exact canonical and f32/index/joint equivalence: {report["rigs"]} rigs; '
              f'{report["before_bytes"]:,} -> {report["after_bytes"]:,} bytes '
              f'({100 * report["saved_bytes"] / report["before_bytes"]:.2f}% smaller)')


if __name__ == '__main__':
    main()
