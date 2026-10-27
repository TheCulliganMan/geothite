#!/usr/bin/env python3
"""Read-only exact preservation proof for published authored terrain revisions.

Consumes completed packs; never writes a pack/catalog or executes gameplay.
Example: --before OLD --after NEW --revision OLD_LAYOUT NEW_LAYOUT DOSSIER
Repeat --revision for an atomic multi-route comparison; --self-test adds focused
in-memory corruption checks. Every actual/source render, conversion and rejected
pass is retained through the shared render archive.
"""
import argparse
import base64
from copy import copy, deepcopy
import hashlib
import io
import json
from pathlib import Path
import re

import cbor2
from PIL import Image, ImageChops, ImageDraw

from render_archive import save_render
from render_reference import SourceArt, layout_blocks, load_pack
from render_stages import render_stage
from review_boundary_basecamp_native import actor_scene

ROOT = Path(__file__).resolve().parents[3]
OUTPUT = ROOT / 'planning/minnesota/renders/terrain-revisions'
IDENTITY_FIELDS = ('format_version', 'data', 'audio_manifest', 'runtime_files', 'report')
CAMERAS = {'MN_R04': [('east_gardens', (45, 17)), ('old_forecourt', (45, 19)),
                     ('bridge_arrival', (39, 19))],
           'MN_R06': [('north_junction', (39, 15)), ('southern_shoulder', (39, 19))],
           'MN_R07': [('north_elbow', (20, 14)), ('trainer_pocket', (14, 13))],
           'MN_R05': [('current_walk', (24, 21))],
           'MN_R08': [('main', (49, 47)), ('north', (50, 43)), ('south', (55, 51))],
           'MN_R09': [('north_bench', (53, 19)), ('haul_track', (65, 47)),
                      ('mine_apron', (45, 43)), ('rail_cutting', (94, 27))],
           'MN_R10': [('carrier_bend', (17, 17)), ('grass_neck', (53, 19)),
                      ('weakest_shoulder', (60, 20))],
           'MN_R11': [('north_cascade', (23, 24)), ('south_cascade', (23, 103)),
                      ('north_bridge', (41, 39)), ('south_bridge', (41, 109))]}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def file_identity(path, pack):
    """Hash exact file bytes and the native five-field identity CBOR tuple.

    map_modules.rs::derive_compiled_game_pack_identity_from_manifest hashes the
    definite five-element tuple. Preserve the original serialized field bytes:
    re-encoding decoded floats/maps with a different encoder is not equivalent.
    This validates the recorded identity, not Rust typed-schema/runtime loading.
    """
    raw = path.read_bytes()
    stream = io.BytesIO(raw[22:])
    header = stream.read(1)[0]
    require(header >> 5 == 5 and header & 31 < 24, 'Unsupported native root-map encoding')
    spans = {}
    for _ in range(header & 31):
        key = cbor2.load(stream)
        start = stream.tell()
        cbor2.load(stream)
        require(key not in spans, 'Duplicate native root field')
        spans[key] = (start, stream.tell())
    require(stream.tell() == len(raw)-22, 'Trailing CBOR payload')
    digest = hashlib.sha256(b'\x85')
    for key in IDENTITY_FIELDS:
        start, end = spans[key]
        digest.update(memoryview(raw)[22+start:22+end])
    require(digest.hexdigest() == pack['identity']['content_hash'], 'Native content identity does not match exact serialized tuple')
    require(pack['identity']['format_version'] == pack['format_version'], 'Identity format version mismatch')
    for field, value in [('maps', len(pack['data']['maps'])), ('items', len(pack['data']['items'])),
                         ('moves', len(pack['data']['moves'])), ('pokemon_species', len(pack['data']['pokemon'])),
                         ('music', len(pack['audio_manifest']['music'])),
                         ('sound_effects', len(pack['audio_manifest']['sound_effects'])),
                         ('cries', len(pack['audio_manifest']['cries']))]:
        require(pack['identity'][field] == value, 'Identity count mismatch: '+field)
    return dict(path=str(path.resolve()), bytes=len(raw), sha256=hashlib.sha256(raw).hexdigest(),
                native_content_hash=digest.hexdigest(), exact_serialized_identity_verified=True)


def read_revision(paths):
    old_path, new_path, dossier_path = map(Path, paths)
    old, new, dossier = [json.loads(path.read_text()) for path in [old_path, new_path, dossier_path]]
    require(old['id'] == new['id'] == dossier['id'], 'Source route identity drift')
    require(re.fullmatch(r'MN_R\d\d', dossier['id']) is not None, 'Expected an authored route identity')
    require(dossier['chunk_metatiles'] == [32, 24], 'Unsupported source chunk dimensions')
    require(old['dimensions_metatiles'] == new['dimensions_metatiles'] == dossier['dimensions_metatiles'], 'Source extent drift')
    # Status and crossing annotations are not Rust geometry inputs. Existing
    # crossings must remain exact; additions below must describe real new06 decks.
    metadata = {'layers', 'status', 'crossings'}
    strict_equal({k: v for k, v in old.items() if k not in metadata},
                 {k: v for k, v in new.items() if k not in metadata})
    w, h = old['dimensions_metatiles']
    require((dossier['chunk_columns'], dossier['chunk_rows']) == ((w+31)//32, (h+23)//24), 'Source clipped grid mismatch')
    before, after = layout_blocks(old), layout_blocks(new)
    require(len(before) == len(after) == w*h, 'Source block count mismatch')
    old_crossings, new_crossings = old.get('crossings', []), new.get('crossings', [])
    strict_equal(old_crossings, new_crossings[:len(old_crossings)])
    require(len({c['id'] for c in new_crossings}) == len(new_crossings), 'Duplicate crossing annotation')
    for crossing in new_crossings[len(old_crossings):]:
        require(set(crossing) == {'id', 'blocks', 'necessary'} and type(crossing['necessary']) is bool,
                'Unexpected new crossing annotation fields')
        require(crossing['blocks'], 'New crossing must name its actual decks')
        for x, y in crossing['blocks']:
            require(0 <= x < w and 0 <= y < h and after[y*w+x] == 0x06 and before[y*w+x] != 0x06,
                    'New crossing annotation must match newly placed native06 deck')
    changed = [dict(metatile=[i % w, i//w], before=f'{a:02x}', after=f'{b:02x}')
               for i, (a, b) in enumerate(zip(before, after)) if a != b]
    require(changed, 'Declared source revision has no terrain change')
    chunks = []
    for row in range(dossier['chunk_rows']):
        for column in range(dossier['chunk_columns']):
            x, y = column*32, row*24
            cw, ch = min(32, w-x), min(24, h-y)
            clip = lambda blocks: [value for yy in range(y, y+ch) for value in blocks[yy*w+x:yy*w+x+cw]]
            chunks.append(dict(name=f'{dossier["map_prefix"]}C{column:02}R{row:02}', origin=[x, y],
                               size=[cw, ch], before=clip(before), after=clip(after)))
    return dict(id=dossier['id'], old=old, new=new, dossier=dossier, before=before, after=after,
                chunks=chunks, changed=changed,
                source_files=[dict(path=str(path.resolve()), sha256=hashlib.sha256(path.read_bytes()).hexdigest())
                              for path in [old_path, new_path, dossier_path]])


def validate_chunk(pack, revision, chunk, state):
    name = chunk['name']
    module = pack['data']['maps'][name]
    attrs = module['attributes']
    require(module['id'] == name, name+': native module identity mismatch')
    require([attrs['width'], attrs['height']] == chunk['size'], name+': clipped native extent mismatch')
    require(attrs['tileset_name'] == revision['old']['compiled_tileset'], name+': native tileset mismatch')
    require(pack['data']['map_attributes'][name] == attrs, name+': split map attributes differ')
    labels = [attrs[key] for key in ['blocks_label', 'map_scripts_label', 'map_events_label']]
    require(all(isinstance(label, str) and label for label in labels) and len(set(labels)) == 3,
            name+': invalid/aliased native section labels')
    for owner, other in pack['data']['maps'].items():
        if owner != name:
            foreign = [other['attributes'].get(key) for key in ['blocks_label', 'map_scripts_label', 'map_events_label']]
            require(not set(labels) & set(foreign), name+': section label shared by '+owner)
    expected = chunk[state]
    require(module['blocks'] == expected, name+': entire native terrain differs from '+state+' source')
    require(all(type(block) is int and 0 <= block <= 255 for block in module['blocks']), name+': non-byte native block')
    require(all(type(block) is int and 0 <= block <= 255 for block in expected), 'Non-byte source block')
    label = labels[0]
    encoded = pack['data']['map_blocks'][label]
    require(base64.b64decode(encoded, validate=True) == bytes(expected), name+': encoded map_blocks differ from source/native blocks')
    require(encoded == base64.b64encode(bytes(expected)).decode(), name+': noncanonical base64 block payload')
    return label


def strict_equal(before, after, allowed=frozenset(), path=()):
    """Compare every other field, including types; bool is not an integer here."""
    if path in allowed or before is after:
        return
    def fail(kind):
        raise ValueError('Unexpected '+kind+' change at '+('/'.join(map(str, path)) or '<root>'))
    if type(before) is not type(after):
        fail('type')
    if isinstance(before, dict):
        if {(type(key), key) for key in before} != {(type(key), key) for key in after}:
            fail('key-set')
        for key in before:
            strict_equal(before[key], after[key], allowed, path+(key,))
    elif isinstance(before, list):
        if len(before) != len(after):
            fail('list-length')
        # Native file/audio bytes are serialized as large integer arrays. This
        # fast path still checks exact scalar types, including bool-vs-int, and
        # never skips a container that could contain an allowed descendant.
        if all(type(a) is int and type(b) is int for a, b in zip(before, after)) and before == after:
            return
        for i, (a, b) in enumerate(zip(before, after)):
            strict_equal(a, b, allowed, path+(i,))
    else:
        if before != after:
            fail('value')


def compare_packs(before, after, revisions):
    require(len({revision['id'] for revision in revisions}) == len(revisions), 'Duplicate declared route revision')
    allowed = {('identity', 'content_hash')}
    names, labels, chunk_changes = set(), set(), []
    for revision in revisions:
        expected_names = {chunk['name'] for chunk in revision['chunks']}
        pattern = re.compile(re.escape(revision['dossier']['map_prefix'])+r'C\d\dR\d\d')
        for pack in [before, after]:
            require({name for name in pack['data']['maps'] if pattern.fullmatch(name)} == expected_names,
                    revision['id']+': missing/extra exterior chunk')
        for chunk in revision['chunks']:
            name = chunk['name']
            require(name not in names, 'Route revisions share an intended map')
            names.add(name)
            old_label = validate_chunk(before, revision, chunk, 'before')
            new_label = validate_chunk(after, revision, chunk, 'after')
            require(old_label == new_label and old_label not in labels, name+': block label changed/shared')
            labels.add(old_label)
            if chunk['before'] != chunk['after']:
                allowed.update({('data', 'maps', name, 'blocks'), ('data', 'map_blocks', old_label)})
                chunk_changes.append(dict(map=name, blocks_label=old_label,
                                          changed_blocks=sum(a != b for a, b in zip(chunk['before'], chunk['after']))))
    strict_equal(before, after, allowed)
    require(before['identity']['content_hash'] != after['identity']['content_hash'], 'Changed terrain requires a new native identity')
    return dict(changed_chunks=chunk_changes, exact_allowed_field_paths=['/'.join(path) for path in sorted(allowed)],
                all_other_pack_fields_strictly_unchanged=True)


class Images:
    def __init__(self, directory):
        self.directory, self.paths = directory, {}

    def save(self, name, image):
        digest = hashlib.sha256(f'{image.mode}:{image.size}:'.encode()+image.tobytes()).hexdigest()[:12]
        path = self.directory/f'{name}-{digest}.png'
        save_render(image, path)
        self.paths[name] = dict(path=str(path), mode=image.mode, size=list(image.size))

    def stage(self, image, operation, label, *args):
        return render_stage(image, operation, self.directory, label, *args)


def whole(pack, revision, state, art, source=False):
    w, h = revision['old']['dimensions_metatiles']
    actors = []
    if source:
        layout = revision['old' if state == 'before' else 'new']
        terrain = art.render(layout['tileset'], w, h, revision[state], derived=layout.get('derived_blocks', []))
    else:
        terrain = Image.new('RGB', (w*32, h*32))
    for chunk in revision['chunks']:
        module = pack['data']['maps'][chunk['name']]
        attrs = module['attributes']
        x, y = chunk['origin']
        if not source:
            terrain.paste(art.render(attrs['tileset_name'], attrs['width'], attrs['height'], module['blocks']), (x*32, y*32))
        actors.extend(dict(actor, x=actor['x']+x*2, y=actor['y']+y*2) for actor in module['objects'])
    return terrain, actors


def validate_recipes(pack, revision):
    art = SourceArt(pack)
    layout = revision['old']
    for recipe in layout.get('derived_blocks', []):
        block = int(recipe['id'], 16)
        expected = art.render(layout['tileset'], 1, 1, [block], derived=layout['derived_blocks'])
        actual = art.render(layout['compiled_tileset'], 1, 1, [block])
        require(actual.tobytes() == expected.tobytes(), revision['id']+': native glyph differs from exact source recipe '+recipe['id'])
        if 'collision_source_block' in recipe:
            collision = pack['data']['tilesets'][recipe.get('collision_source_tileset', recipe['source_tileset'])]['collision'][recipe['collision_source_block']]
        elif 'collision' in recipe:
            collision = recipe['collision'].split('/')
            collision *= 4 if len(collision) == 1 else 1
        else:
            # Exact native imports (e.g. Iron Range rails83/84) retain their
            # source collision rather than spelling out an override recipe.
            collision = pack['data']['tilesets'][recipe['source_tileset']]['collision'][recipe['source_block']]
        if 'collision_order' in recipe:
            order = recipe['collision_order']
            require(type(order) is list and len(order) == 4
                    and all(type(index) is int for index in order)
                    and sorted(order) == [0, 1, 2, 3],
                    revision['id']+': collision order must permute the four source quadrants')
            collision = [collision[index] for index in order]
        require(pack['data']['tilesets'][layout['compiled_tileset']]['collision'][recipe['id']] == collision,
                revision['id']+': native recipe collision mismatch '+recipe['id'])


def render_revision(before, after, revision, images):
    scenes, terrains = {}, {}
    route = revision['id'].lower()
    for state, pack in [('before', before), ('after', after)]:
        art = SourceArt(pack)
        for kind in ['native', 'source']:
            name = f'{route}-{state}-{kind}'
            terrain, actors = whole(pack, revision, state, art, source=kind == 'source')
            images.save(name+'-terrain-original', terrain)
            canvas = images.stage(terrain, 'convert', name+'-canvas-rgba', 'RGBA')
            images.save(name+'-canvas-original-rgba', canvas)
            scene = actor_scene(pack, canvas, actors)
            images.save(name+'-whole-original-rgba', scene)
            images.save(name+'-whole-rgb', images.stage(scene, 'convert', name+'-whole-rgb', 'RGB'))
            scenes[state, kind], terrains[state, kind] = scene, terrain
    images.save(route+'-whole-native-difference-original', ImageChops.difference(terrains['before', 'native'], terrains['after', 'native']))
    for label, (x, y) in CAMERAS.get(revision['id'], []):
        frames = []
        for state, kind in [('before', 'native'), ('after', 'native'), ('before', 'source'), ('after', 'source')]:
            name = f'{route}-{label}-{state}-{kind}'
            raw = images.stage(scenes[state, kind], 'crop', name+'-160', (x*16-80, y*16-72, x*16+80, y*16+72))
            images.save(name+'-original-160', raw)
            rgb = images.stage(raw, 'convert', name+'-rgb', 'RGB')
            images.save(name+'-rgb-160', rgb)
            images.save(name+'-4x', images.stage(rgb, 'resize', name+'-4x', (640, 576), Image.Resampling.NEAREST))
            frames.append((state+' '+kind, rgb))
        difference = ImageChops.difference(frames[0][1], frames[1][1])
        images.save(f'{route}-{label}-native-difference-original-160', difference)
        images.save(f'{route}-{label}-native-difference-4x', images.stage(difference, 'resize', route+'-'+label+'-diff-4x', (640, 576), Image.Resampling.NEAREST))
        framed = Image.new('RGB', (4*168+8, 176), (19, 29, 40))
        draw = ImageDraw.Draw(framed)
        for i, (name, frame) in enumerate(frames):
            draw.text((8+i*168, 6), name, fill='white')
            framed.paste(frame, (8+i*168, 24))
        images.save(f'{route}-{label}-before-after-framed-original', framed)
    for state in ['before', 'after']:
        require(terrains[state, 'native'].tobytes() == terrains[state, 'source'].tobytes(),
                revision['id']+': rendered '+state+' native/source terrain pixels differ')


def changed_copy(value, path, replacement):
    """Copy only the mutation path; no pack file or shared object is mutated."""
    if not path:
        return replacement
    result = copy(value)
    result[path[0]] = changed_copy(value[path[0]], path[1:], replacement)
    return result


def adversarial_checks(before, after, revisions):
    # If the actual output is unavailable/stale, build only an explicitly labeled
    # in-memory comparison fixture. Never encode it or use it as native evidence.
    try:
        compare_packs(before, after, revisions)
        fixture = after
        fixture_label = 'Actual loaded after pack; only in-memory mutation paths copied, no file changes'
    except ValueError:
        fixture = before
        for revision in revisions:
            for chunk in revision['chunks']:
                label = before['data']['maps'][chunk['name']]['attributes']['blocks_label']
                fixture = changed_copy(fixture, ('data', 'maps', chunk['name'], 'blocks'), chunk['after'])
                fixture = changed_copy(fixture, ('data', 'map_blocks', label), base64.b64encode(bytes(chunk['after'])).decode())
        fixture = changed_copy(fixture, ('identity', 'content_hash'), 'comparison-fixture-not-a-native-identity')
        fixture_label = 'In-memory comparison fixture only; not exported or asserted to have a valid native identity'
    compare_packs(before, fixture, revisions)
    selected = next((r, c, i) for r in revisions for c in r['chunks'] for i, (a, b) in enumerate(zip(c['before'], c['after'])) if a != b)
    revision, chunk, index = selected
    name = chunk['name']
    label = before['data']['maps'][name]['attributes']['blocks_label']
    trainer = next(iter(before['data']['trainers']['trainers']))
    while not before['data']['trainers']['trainers'][trainer]['party']:
        trainer = next(key for key, value in before['data']['trainers']['trainers'].items() if value['party'])
    guard = next(i for i, actor in enumerate(before['data']['maps']['MnBoundaryWaters']['objects']) if 'GUARD' in actor['object_identifier'])
    unrelated = next(key for key in before['data']['maps'] if key not in {c['name'] for r in revisions for c in r['chunks']} and key != 'MnBoundaryWaters')
    payload = bytearray(chunk['after']); payload[index] ^= 1
    music = next(iter(before['audio_manifest']['music']))
    audio_hash = before['audio_manifest']['music'][music]['payload_hash']
    spawn = next(key for key, value in before['data']['runtime_spawn_points'].items() if value['identifier'] == 0)
    cases = [
        ('stale-old-native-block', ('data', 'maps', name, 'blocks', index), chunk['before'][index]),
        ('unrelated-map-block', ('data', 'maps', unrelated, 'blocks', 0), before['data']['maps'][unrelated]['blocks'][0] ^ 1),
        ('altered-guard', ('data', 'maps', 'MnBoundaryWaters', 'objects', guard, 'event_flag'), 'EVENT_MN_SURVEY_RESTORED'),
        ('altered-trainer-party', ('data', 'trainers', 'trainers', trainer, 'party', 0, 'level'), 99),
        ('altered-HOME-recovery', ('data', 'runtime_spawn_points', spawn, 'tileX'), before['data']['runtime_spawn_points'][spawn]['tileX']+1),
        ('altered-audio-manifest', ('audio_manifest', 'music', music, 'payload_hash'), ('0' if audio_hash[0] != '0' else '1')+audio_hash[1:]),
        ('mismatched-encoded-blocks', ('data', 'map_blocks', label), base64.b64encode(payload).decode()),
        ('altered-report', ('report', 'manifests'), before['report']['manifests']+['undeclared-revision']),
    ]
    results = []
    for title, path, value in cases:
        try:
            compare_packs(before, changed_copy(fixture, path, value), revisions)
        except (ValueError, KeyError, TypeError) as error:
            results.append(dict(case=title, rejected=True, error=str(error)))
        else:
            raise ValueError('Adversarial corruption was accepted: '+title)
    changed_source = deepcopy(revisions)
    changed_source[0]['chunks'][0]['after'][0] ^= 1
    try:
        compare_packs(before, fixture, changed_source)
    except ValueError as error:
        results.append(dict(case='mismatched-new-source', rejected=True, error=str(error)))
    else:
        raise ValueError('Mismatched new source accepted')
    for original in revisions:
        for index, recipe in enumerate(original['old'].get('derived_blocks', [])):
            if 'collision_order' not in recipe:
                continue
            for title, order in [('wrong-fence-quadrant-order', [0, 1, 2, 3]),
                                 ('duplicated-fence-quadrant-order', [0, 0, 2, 3])]:
                altered = deepcopy(original)
                altered['old']['derived_blocks'][index]['collision_order'] = order
                try:
                    validate_recipes(before, altered)
                except ValueError as error:
                    results.append(dict(case=title, rejected=True, error=str(error)))
                else:
                    raise ValueError('Invalid fence collision order accepted: '+title)
    return dict(adversarial_cases=results, adversarial_fixture=fixture_label)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--before', type=Path, required=True)
    parser.add_argument('--after', type=Path, required=True)
    parser.add_argument('--revision', nargs=3, action='append', required=True, metavar=('BEFORE_LAYOUT', 'AFTER_LAYOUT', 'DOSSIER'))
    parser.add_argument('--output', type=Path, default=OUTPUT)
    parser.add_argument('--self-test', action='store_true')
    args = parser.parse_args()
    before, after = load_pack(args.before), load_pack(args.after)
    revisions = [read_revision(paths) for paths in args.revision]
    directory = args.output/(before['identity']['content_hash'][:12]+'-to-'+after['identity']['content_hash'][:12])
    directory.mkdir(parents=True, exist_ok=True)
    images = Images(directory)
    report = dict(status='Read-only static terrain/preservation comparison; not controller, earned progression or visual acceptance',
                  before_file=str(args.before), after_file=str(args.after), errors=[], checks_passed=[],
                  revisions=[dict(id=r['id'], source_files=r['source_files'], dimensions_metatiles=r['old']['dimensions_metatiles'],
                                  changed_metatiles=r['changed'], cameras_core=CAMERAS.get(r['id'], [])) for r in revisions])
    def attempt(label, operation):
        try:
            result = operation()
            if isinstance(result, dict):
                report.update(result)
            report['checks_passed'].append(label)
        except (ValueError, AssertionError, KeyError, TypeError, IndexError) as error:
            report['errors'].append(dict(check=label, error=str(error)))
    for state, pack, path in [('before', before, args.before), ('after', after, args.after)]:
        attempt(state+'-file-and-native-identity', lambda state=state, pack=pack, path=path: {state: file_identity(path, pack)})
    # Render actual and source inputs even when strict comparison rejects a pass.
    for revision in revisions:
        attempt(revision['id']+'-retained-native-source-renders', lambda revision=revision: render_revision(before, after, revision, images))
        for state, pack in [('before', before), ('after', after)]:
            attempt(revision['id']+'-'+state+'-native-recipes', lambda pack=pack, revision=revision: validate_recipes(pack, revision))
    attempt('all-fields-except-exact-declared-terrain', lambda: compare_packs(before, after, revisions))
    if args.self_test:
        attempt('adversarial-in-memory-negative-checks', lambda: adversarial_checks(before, after, revisions))
    report.update(ok=not report['errors'], renders=images.paths)
    encoded = (json.dumps(report, indent=2, sort_keys=True)+'\n').encode()
    path = directory/('comparison-'+hashlib.sha256(encoded).hexdigest()+'.json')
    try:
        with path.open('xb') as stream:
            stream.write(encoded)
    except FileExistsError:
        require(path.read_bytes() == encoded, 'Immutable report collision')
    print(json.dumps(dict(ok=report['ok'], immutable_report=str(path), errors=report['errors'],
                          checks_passed=report['checks_passed'], render_count=len(images.paths)), indent=2))
    if not report['ok']:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
