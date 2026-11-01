#!/usr/bin/env python3
"""Validate canonical species geometry, source routing and runtime storage limits.

No content pack or renderer is started. --allow-pending permits declared
sources not yet installed in the runtime registry; installed files must still
match the registry, including the complete Unown family when present.
"""
import argparse
import ast
import hashlib
import json
import math
import re
from pathlib import Path

from animated_glb import parse_glb
from battle_model_assets import ACTOR_READERS, GLB_READERS, STARTER_READERS, read_species_model, species_paths

ROOT = Path(__file__).resolve().parents[1]
COORDINATES = '+Y up; front +Z; floor-centered root'


def roster_name(name):
    return 'unown' if re.fullmatch(r'unown_[a-z]', name) else name


def read_registry(source):
    pairs = re.findall(r'^\s*"([A-Z0-9_]+)" => "([a-z0-9_]+)",', source, re.M)
    assert pairs, 'empty exact species registry'
    assert len({species for species, _ in pairs}) == len(pairs), 'duplicate species registry ID'
    assert len({name for _, name in pairs}) == len(pairs), 'duplicate species registry source'
    for species, name in pairs:
        expected = 'unown_i' if species == 'UNOWN' else species.lower()
        assert name == expected, (species, name, 'incorrect canonical source identity')
    return dict(pairs)


def authoring_names(builder_source):
    tree = ast.parse(builder_source)
    names = [name.lower() for function in tree.body if isinstance(function, ast.FunctionDef)
             for decorator in function.decorator_list
             if isinstance(decorator, ast.Call) and isinstance(decorator.func, ast.Name)
             and decorator.func.id == 'register' for name in decorator.args[0].value.split()]
    assert len(names) == len(set(names)), 'duplicate authored species'
    # Standard-skin readers supersede the older Blender decorators. Actor-space
    # species remain single canonical sources, rather than duplicate exports.
    declared = {roster_name(name) for name in names}
    declared.update(roster_name(Path(name).stem) for name in GLB_READERS)
    declared.update(Path(name).stem.removeprefix('battle_').removeprefix('icon_')
                    for name in ACTOR_READERS)
    return declared


def runtime_limits(source):
    limits = {}
    for name in ('MAX_BYTES', 'MAX_VERTICES', 'MAX_INDICES', 'MAX_ACCESSORS'):
        found = re.search(rf'^const {name}: usize = ([0-9_* ]+);$', source, re.M)
        assert found, ('cannot read current runtime limit', name)
        factors = found[1].replace('_', '').split('*')
        assert all(factor.strip().isdigit() for factor in factors), ('invalid runtime limit', name)
        limits[name] = math.prod(int(factor) for factor in factors)
    return limits


def validate_storage(path, limits):
    blob = path.read_bytes()
    assert len(blob) <= limits['MAX_BYTES'], (path.name, 'runtime byte limit')
    doc, _ = parse_glb(blob)
    assert len(doc['accessors']) <= limits['MAX_ACCESSORS'], (path.name, 'runtime accessor limit')
    assert len(doc['bufferViews']) <= limits['MAX_ACCESSORS'], (path.name, 'runtime buffer-view limit')
    primitives = doc['meshes'][0]['primitives']
    vertices = sum(doc['accessors'][p['attributes']['POSITION']]['count'] for p in primitives)
    indices = sum(doc['accessors'][p['indices']]['count'] for p in primitives)
    assert vertices <= limits['MAX_VERTICES'], (path.name, 'runtime vertex limit', vertices)
    assert indices <= limits['MAX_INDICES'], (path.name, 'runtime index limit', indices)
    return vertices, indices // 3


def expanded_paths(registry, paths):
    names = {roster_name(name) for name in registry.values()}
    assert set(paths) == names, ('runtime files and registry differ', set(paths) ^ names)
    targets = {species.lower(): paths[roster_name(name)] for species, name in registry.items()}
    if 'UNOWN' in registry:
        # The base roster resolves to I. Validate that same file once, alongside
        # each additional form, without mistaking forms for extra species.
        targets.pop('unown')
        targets.update({'unown_' + letter: paths['unown'].with_name('unown_' + letter + '.glb')
                        for letter in 'abcdefghijklmnopqrstuvwxyz'})
    return targets


def validate_geometry(name, path, model, limits):
    # Actor filenames retain their historic prefixes, but Clefairy/Jigglypuff
    # readers publish canonical species names. Starter authoring names retain
    # their explicit battle_ prefix. Identity follows these reader contracts.
    expected = 'battle_' + name if path.name in STARTER_READERS else name
    assert model['name'] == expected and model['version'] == 1, (name, 'incorrect model identity')
    assert model['coordinate_system'] == COORDINATES, (name, 'incorrect coordinate system')
    points, geometry, parts = [], [], []
    triangles = vertices = 0
    for part in model['primitives']:
        parts.append(part['part'])
        pos, nor, ind, col = (part[k] for k in ('positions', 'normals', 'indices', 'base_color'))
        tag = (name, part['part'])
        assert len(pos) >= 9 and len(pos) % 3 == 0 and len(nor) == len(pos), tag
        assert all(math.isfinite(x) for x in pos + nor), tag
        assert len(col) == 4 and all(math.isfinite(v) and 0 <= v <= 1 for v in col), tag
        assert ind and len(ind) % 3 == 0 and all(type(i) is int and 0 <= i < len(pos) // 3 for i in ind), tag
        ps, ns = list(zip(*[iter(pos)] * 3)), list(zip(*[iter(nor)] * 3))
        points.extend(ps)
        for normal in ns:
            assert abs(sum(x * x for x in normal) - 1) < .000025, (tag, 'normal not unit')
        volume = 0.
        for k in range(0, len(ind), 3):
            x, y, z = (ps[i] for i in ind[k:k + 3])
            u, v = [y[j] - x[j] for j in range(3)], [z[j] - x[j] for j in range(3)]
            cross = [u[1]*v[2]-u[2]*v[1], u[2]*v[0]-u[0]*v[2], u[0]*v[1]-u[1]*v[0]]
            normal = [sum(ns[i][j] for i in ind[k:k + 3]) for j in range(3)]
            assert sum(cross[j] * normal[j] for j in range(3)) >= -1e-7, (tag, 'winding and normal disagree')
            volume += x[0]*(y[1]*z[2]-y[2]*z[1]) + x[1]*(y[2]*z[0]-y[0]*z[2]) + x[2]*(y[0]*z[1]-y[1]*z[0])
        assert volume > 1e-11, (tag, 'flat or inverted closed surface', volume)
        triangles += len(ind) // 3
        vertices += len(ps)
        geometry.append(([round(x, 4) for x in pos], ind))
    assert len(parts) >= 3 and len(set(parts)) == len(parts), (name, 'missing or repeated anatomy names')
    low = [min(v[i] for v in points) for i in range(3)]
    high = [max(v[i] for v in points) for i in range(3)]
    assert abs(low[1]) < .0001, (name, 'not grounded')
    assert all(high[i] - low[i] > .005 for i in range(3)), (name, 'not volumetric')
    assert max(abs(v) for v in low + high) < 4, (name, 'unexpected dimensions')
    assert 100 <= triangles <= limits['MAX_INDICES'] // 3, (name, 'runtime triangle limit', triangles)
    # Packing may share logical anatomy vertices. GLBs use their actual stored
    # counts for loader budgets; pending static models have no such packing.
    if path.suffix != '.glb':
        assert vertices <= limits['MAX_VERTICES'], (name, 'runtime vertex limit', vertices)
    digest = hashlib.sha256(json.dumps(geometry, separators=(',', ':')).encode()).hexdigest()
    return vertices, triangles, digest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--models', type=Path, default=ROOT/'crates/crystal-voxel-view/models/battle_species')
    parser.add_argument('--registry', type=Path, default=ROOT/'crates/crystal-voxel-view/src/battle_species_models.rs')
    parser.add_argument('--allow-pending', action='store_true')
    parser.add_argument('--progress', action='store_true')
    args = parser.parse_args()
    registry = read_registry(args.registry.read_text())
    authored = authoring_names((ROOT/'tools/build-battle-species.py').read_text())
    registered = {roster_name(name) for name in registry.values()}
    assert registered <= authored, ('un-authored species registered', registered - authored)
    if not args.allow_pending:
        assert authored == registered, ('pending authored exports', authored - registered)
    limits = runtime_limits((ROOT/'crates/crystal-voxel-view/src/species_rig.rs').read_text())
    targets = expanded_paths(registry, species_paths(args.models, include_starters=True))
    hashes = {}
    triangles = vertices = 0
    for index, (name, path) in enumerate(targets.items(), 1):
        if args.progress:
            print(f'checking {index}/{len(targets)} variants: {name}', flush=True)
        model = read_species_model(path)
        count_v, count_t, digest = validate_geometry(name, path, model, limits)
        if path.suffix == '.glb':
            validate_storage(path, limits)
        assert digest not in hashes, (name, 'same geometry under different identities', hashes.get(digest))
        hashes[digest] = name
        vertices += count_v
        triangles += count_t
    print(f'PASS: {len(registry)} species; {len(targets)} variants; {triangles:,} triangles; {vertices:,} source vertices; '
          'canonical routing, unique geometry, finite attributes, outward winding, unit normals, floor roots and current runtime storage limits')
    if args.allow_pending:
        print(f'Authoring queue: {len(authored - registered)} declared sources not yet registered')


if __name__ == '__main__':
    main()
