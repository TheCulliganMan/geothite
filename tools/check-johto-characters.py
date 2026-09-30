#!/usr/bin/env python3
"""Validate original source-specific human geometry without Blender or a game pack.

The generator's explicit design table is authoring data, not an exported source
catalog. This check keeps it synchronized with the exact Rust lookup and validates
all 16 joint-bound volumes. It never reads or writes external game content.
"""
import ast
import hashlib
import json
import math
import re
from johto_character_geometry import LIBRARY_FILE, expand_rig, load_library, compact_models, canonical
from pathlib import Path
from model_asset_storage import read_model_bytes, read_model_json, read_model_text, validate_storage

ROOT = Path(__file__).resolve().parent.parent
GENERATOR = ROOT / 'tools/build-johto-characters.py'
RUST = ROOT / 'crates/crystal-voxel-view/src/johto_characters.rs'
MODELS = ROOT / 'crates/crystal-voxel-view/models/johto_characters'


def assigned_literal(tree, name):
    return next(ast.literal_eval(node.value) for node in tree.body
                if isinstance(node, ast.Assign) and any(
                    isinstance(target, ast.Name) and target.id == name
                    for target in node.targets))


def check():
    tree = ast.parse(GENERATOR.read_text())
    designs = assigned_literal(tree, 'DESIGNS')
    joints = assigned_literal(tree, 'JOINTS')
    assert len(designs) == 75
    assert len({tuple(v) for v in designs.values()}) == len(designs), 'Alias-only designs'
    rust = RUST.read_text()
    references = re.findall(r'include(?:_str)?!\(\s*"\.\./models/johto_characters/([^\"]+)"\s*\)', rust)
    references = [name.removesuffix('.include.rs') for name in references]
    expected = {name + '.rig.json' for name in designs} | {LIBRARY_FILE}
    assert expected == set(references), (expected - set(references), set(references) - expected)
    source_block = rust.split('pub(super) const SOURCE_KINDS:', 1)[1].split('];', 1)[0]
    sources = re.findall(r'\("([a-z_]+)", CharacterKind::([A-Za-z]+)\)', source_block)
    assert len(sources) == 74 and len({source for source, _ in sources}) == 74
    assert len({kind for _, kind in sources}) == 74, 'Different source IDs share a rig'
    assert not any(name in dict(sources) for name in ('remote_player', 'surf', 'chris_bike', 'kris_bike'))
    canonical_sources = {name for name in designs if name not in ('trainer', 'trainer_female', 'outdoorsman')}
    canonical_sources.update(('chris', 'kris'))
    assert canonical_sources == {source for source, _ in sources}
    library = load_library(MODELS)
    assert library is not None
    expanded_models = {}
    fingerprints = set()
    triangle_counts = []
    for name in designs:
        path = MODELS / (name + '.rig.json')
        packed = read_model_json(path)
        assert packed['version'] == 2, (name, 'runtime catalog must use shared geometry')
        model = expand_rig(packed, library)
        expanded_models[path.name] = model
        assert model['version'] == 1 and model['name'] == name
        assert len(model['joints']) == 16
        count = 0
        for index, ((joint_name, parent, pivot), joint) in enumerate(zip(joints, model['joints'])):
            assert joint['name'] == joint_name
            parent_index = next((i for i, item in enumerate(joints) if item[0] == parent), None)
            assert joint['parent'] == parent_index
            parent_pivot = joints[parent_index][2] if parent_index is not None else (0, 0, 0)
            local = [pivot[i] - parent_pivot[i] for i in range(3)]
            expected_translation = (local[0], local[2], -local[1])
            assert max(abs(a - b) for a, b in zip(expected_translation, joint['translation'])) < 0.00001
            assert joint['primitives'], (name, joint_name, 'empty articulated joint')
            for primitive in joint['primitives']:
                positions, normals, indices = (primitive[key] for key in ('positions', 'normals', 'indices'))
                assert len(positions) == len(normals) and len(positions) % 3 == 0
                assert len(indices) % 3 == 0 and indices
                assert all(math.isfinite(v) for v in positions + normals)
                assert min(indices) >= 0 and max(indices) < len(positions) // 3
                assert all(0 <= c <= 1 for c in primitive['base_color'])
                for i in range(0, len(normals), 3):
                    length = sum(v * v for v in normals[i:i + 3]) ** .5
                    assert abs(length - 1) < .00001, (name, joint_name, length)
                count += len(indices) // 3
            if joint_name.startswith('shoe'):
                floor = min(p['positions'][i] + pivot[2]
                            for p in joint['primitives']
                            for i in range(1, len(p['positions']), 3))
                assert 0 <= floor < .006, (name, floor)
        # Geometry-only comparison rejects palette-only duplicates as well.
        shapes = [[p['positions'] for p in joint['primitives']] for joint in model['joints']]
        fingerprint = hashlib.sha256(json.dumps(shapes, separators=(',', ':')).encode()).hexdigest()
        assert fingerprint not in fingerprints, (name, 'duplicate full geometry')
        fingerprints.add(fingerprint)
        assert 1000 <= count <= 25000, (name, count)
        triangle_counts.append(count)
    rebuilt_library, rebuilt_models = compact_models(expanded_models)
    assert canonical(rebuilt_library) == read_model_bytes(MODELS / LIBRARY_FILE)
    for filename, model in rebuilt_models.items():
        assert canonical(model) == read_model_bytes(MODELS / filename), filename
    print(f'Validated {len(sources)} exact human source identities, {len(designs)} unique articulated shapes; '
          f'{min(triangle_counts):,}–{max(triangle_counts):,} triangles per model; fixed floor and gait pivots')


if __name__ == '__main__':
    check()
