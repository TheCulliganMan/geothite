#!/usr/bin/env python3
"""Guarded one-room/one-contract R17 source promotion; preflight never writes live files."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import re
import time

ROOT = Path(__file__).resolve().parents[3]
ROOM = 'MnR17Cave1F'
CANDIDATE_SHA = 'f590d69e806a4a9e8aae29d03ca2c3b40a386b45ab366755965730ecc67c5437'
PREVIOUS_SHA = '37ba34f06fe3abf8948643510eaeb2b7a7d65994997c61aed09e0e49ce204967'
PACK_SHA = '4891e6a084e21ea05866740fb04cc8c9891fca2433dbb5a93b58269aed78b72a'
CASE = 'r17_tor_cave_controller_bent_route_directional_roof_revive_once_and_continue'
RETURN = 'Original indoor warp1 at9,13 LADDER returns directly to true passive exterior warp2 at10,44 FLOOR; stable through60 presentation frames and walking away/back. Active CAVE warp1 at10,43 deliberately reenters; indoor clear Floor9,12 is distinct from warp arrival9,13.'
NOTICE = 'VisibleShellController internally settles automatic FoundText and native60-frame fanfare before returning at PocketText. The real input case asserts retained native found payload/ItemBall presentation, both fully printed PocketText pages and movement ownership; it does not claim an intermediate Found/Fanfare snapshot.'
PATHS = [ROOT / 'planning/minnesota/routes/MN_R17.json', ROOT / 'planning/minnesota/interiors.json']
EVIDENCE = [
    'planning/minnesota/renders/caves/r17-native/4891e6a084e21ea0/review-01/native-verification-and-captures.json',
    'planning/minnesota/renders/caves/r17-native/4891e6a084e21ea0/review-01/visual-review.json',
    '.local/r17-tor-controller-pocket-fixed.log',
    '.local/r17-tor-controller-pocket-fixed-receipt.json',
    '.local/r17-tor-native-audio.log',
]


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def require(value, message):
    if not value:
        raise ValueError(message)


def read(path):
    return json.loads(path.read_bytes())


def write(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')


def owned(raw, key):
    text = raw.decode('utf-8')
    matches = list(re.finditer(r'\{\s*"' + re.escape(key) + r'"\s*:\s*"' + ROOM + r'"', text))
    require(len(matches) == 1, 'exactly one owned object required')
    start = matches[0].start()
    record, length = json.JSONDecoder().raw_decode(text[start:])
    indent = text[text.rfind('\n', 0, start) + 1:start]
    require(not indent.strip(), 'owned object must start on its own line')
    return record, len(text[:start].encode()), len(text[:start + length].encode()), indent


def splice(raw, key, expected, proposed):
    record, start, end, indent = owned(raw, key)
    require(record == expected, 'owned source record does not equal frozen original')
    replacement = json.dumps(proposed, indent=2).replace('\n', '\n' + indent).encode()
    result = raw[:start] + replacement + raw[end:]
    require(json.loads(result), 'full proposed source must parse')
    proof = dict(owned_start_byte=start, original_owned_end_byte=end,
                 proposed_owned_end_byte=start + len(replacement),
                 prefix_bytes=start, suffix_bytes=len(raw) - end,
                 prefix_sha256=sha(raw[:start]), suffix_sha256=sha(raw[end:]),
                 prefix_exact=result[:start] == raw[:start],
                 suffix_exact=result[start + len(replacement):] == raw[end:])
    require(proof['prefix_exact'] and proof['suffix_exact'], 'outside-owned bytes changed')
    return result, proof


def certify_evidence():
    proof = read(ROOT / EVIDENCE[0])
    require(proof['status'] == 'PASS' and len(proof['checks']) == 152
            and all(row['passed'] for row in proof['checks'])
            and proof['pack']['file_sha256'] == PACK_SHA and proof['pack']['maps'] == 510
            and proof['source_sha256'] == CANDIDATE_SHA and proof['previous_sha256'] == PREVIOUS_SHA,
            'exact152-check native preservation report required')
    require(len(proof['captures']) == 6 and proof['full_lcd_pixels_compared'] == 138240
            and all(row['full_lcd_mismatches'] == 0 for row in proof['captures']), 'six exact fullLCD views required')
    require(any(row['passed'] and row['check'] == 'same7x8 native attributes and forced NITE'
                for row in proof['checks']), 'original native forcedNITE proof required')
    log = (ROOT / EVIDENCE[2]).read_text()
    require('test result: ok. 1 passed' in log and 'test result: FAILED' not in log
            and re.search(r'::' + CASE + r' \.\.\. ok', log), 'actual one-case controller PASS required')
    receipt = read(ROOT / EVIDENCE[3])
    require(receipt['pack_sha256'] == PACK_SHA and receipt['exit_code'] == 0
            and receipt['private_binary_removed'], 'controller receipt must bind exact pack')
    require('377 audio programs; all PCM hashes, frame counts and loop ranges match'
            in (ROOT / EVIDENCE[4]).read_text(), 'native377 audio proof required')
    return {str(p): sha((ROOT / p).read_bytes()) for p in EVIDENCE}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--snapshot', type=Path, required=True)
    parser.add_argument('--freeze', action='store_true')
    parser.add_argument('--apply', action='store_true')
    args = parser.parse_args()
    require(not (args.freeze and args.apply), 'freeze and apply are distinct operations')
    candidate_path = ROOT / 'planning/minnesota/caves/MnR17Cave1F.candidate.json'
    previous_path = ROOT / 'planning/minnesota/caves/MnR17Cave1F.previous.json'
    require(sha(candidate_path.read_bytes()) == CANDIDATE_SHA, 'candidate bytes changed')
    require(sha(previous_path.read_bytes()) == PREVIOUS_SHA, 'previous bytes changed')
    require(sha((ROOT / 'content-packs/minnesota-northstar-tor-review.crystalpack').read_bytes()) == PACK_SHA,
            'actual installed native review pack changed')
    evidence = certify_evidence()
    candidate, old = read(candidate_path), read(previous_path)
    ignored = {'rows', 'purpose', 'candidate_status', 'review_contract'}
    require(all(old.get(k) == candidate.get(k) for k in set(old) | set(candidate) if k not in ignored),
            'candidate changed original nonterrain gameplay fields')
    require(candidate['dimensions_metatiles'] == [7, 8] and candidate['exits'] == [[9, 13]]
            and candidate['clear_arrival'] == [9, 12] and candidate['residents'] == [], 'original room contract changed')
    snapshot = args.snapshot.resolve()
    if args.freeze:
        snapshot.mkdir(parents=True, exist_ok=False)
        for path in PATHS:
            (snapshot / (path.name + '.before')).write_bytes(path.read_bytes())
        write(snapshot / 'freeze.json', dict(source_files={str(p.relative_to(ROOT)): sha(p.read_bytes()) for p in PATHS},
              candidate_sha256=CANDIDATE_SHA, previous_sha256=PREVIOUS_SHA, native_pack_sha256=PACK_SHA,
              execution_evidence_sha256=evidence, script_sha256=sha(Path(__file__).read_bytes()),
              root_acceptance='Root reported actual one controller PASS2.32s, audio377PASS; personally viewed six frames and accepted152/six. Preflight only; apply awaits separate explicit Root authorization.'))
        for path in EVIDENCE:
            target = snapshot / 'evidence' / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes((ROOT / path).read_bytes())
        for path in [candidate_path, previous_path, Path(__file__)]:
            (snapshot / path.name).write_bytes(path.read_bytes())
    freeze = read(snapshot / 'freeze.json')
    require(freeze['execution_evidence_sha256'] == evidence
            and freeze['script_sha256'] == sha(Path(__file__).read_bytes()), 'reviewed evidence/script changed')
    before = [(snapshot / (path.name + '.before')).read_bytes() for path in PATHS]
    for path, raw in zip(PATHS, before):
        require(sha(raw) == freeze['source_files'][str(path.relative_to(ROOT))]
                and path.read_bytes() == raw, 'complete live source changed since reviewed freeze: ' + str(path))
    require(owned(before[0], 'id')[0] == old, 'current R17 room differs from exact original')
    contract = owned(before[1], 'interior')[0]
    require(contract['exterior_door_core'] == [10, 43]
            and contract['exterior_clear_return_core'] == [10, 44]
            and contract['mouths'][0]['exterior_return_warp'] == 2, 'original passive return contract changed')
    promoted = copy.deepcopy(candidate)
    promoted['candidate_status'] = 'Installed CURRENT510 tor native review candidate on4891; one real local production-controller case PASS. User final approval and earned Border Pines progression remain unclaimed.'
    promoted['review_contract']['unpublished'] = False
    promoted['review_contract']['native_review'] = dict(pack_sha256=PACK_SHA, palette='nite', candidate_sha256=CANDIDATE_SHA,
        preservation_checks=152, production_captures=6, full_lcd_exact_pixels=138240, controller_case=CASE,
        return_semantics=RETURN, item_notice_observation_limit=NOTICE)
    revised = copy.deepcopy(contract)
    revised.update(dimensions_metatiles=[7, 8], native_palette='nite', native_warp_arrival_core=[9, 13],
        clear_arrival_core=[9, 12], runtime_settled_exterior_core=[10, 44], source_collision_checked=True,
        requires_real_collision_check=False, candidate_source_sha256=CANDIDATE_SHA, native_review_pack_sha256=PACK_SHA,
        return_semantics=RETURN, item_notice_observation_limit=NOTICE,
        validation='Actual CURRENT510152-check native preservation/six exact production captures, one real controller case and377audio PASS. Fitted ordinary route, native directional UP_WALL behavior, actual Revive grant/printed Pocket pages/ownership/once/reentry/collected tile/Continue verified. Original whole item/program/flag/warps/art/attributes exact.',
        native_status='Installed7x8 native tor review candidate on4891; forcedNITE. No NPC/stair/ledge/HM additions; original Revive1 at5,5 and passive warp2 retained. Earned regional progression and user visual approval remain separate.')
    after, proofs = [], []
    for path, raw, key, original, proposed in zip(PATHS, before, ['id', 'interior'], [old, contract], [promoted, revised]):
        result, proof = splice(raw, key, original, proposed)
        after.append(result);proofs.append(dict(path=str(path.relative_to(ROOT)), **proof))
        proposed_path = snapshot / (path.name + '.proposed')
        if proposed_path.exists():
            require(proposed_path.read_bytes() == result, 'reviewed proposed full file changed')
        else:
            proposed_path.write_bytes(result)
    for name, value in [('room.before.json', old), ('contract.before.json', contract),
                        ('room.proposed.json', promoted), ('contract.proposed.json', revised)]:
        write(snapshot / name, value)
    write(snapshot / 'outside-owned-bytes.proposed.json', proofs)
    summary = dict(applied=False, snapshot=str(snapshot), before={str(p.relative_to(ROOT)): sha(v) for p, v in zip(PATHS, before)},
                   proposed={str(p.relative_to(ROOT)): sha(v) for p, v in zip(PATHS, after)},
                   candidate_sha256=CANDIDATE_SHA, native_pack_sha256=PACK_SHA, native_palette='nite')
    if args.apply:
        application = snapshot / ('application-' + str(time.time_ns()))
        application.mkdir()
        for path, raw, result in zip(PATHS, before, after):
            (application / (path.name + '.before')).write_bytes(raw)
            (application / (path.name + '.after')).write_bytes(result)
        staged, written = [], []
        try:
            for path, result in zip(PATHS, after):
                temp = path.with_name('.' + path.name + '.r17-promotion.tmp')
                require(not temp.exists(), 'existing staging file')
                temp.write_bytes(result);staged.append(temp)
            require(all(p.read_bytes() == raw for p, raw in zip(PATHS, before)), 'external writer changed source before replacement')
            for temp, path in zip(staged, PATHS):
                os.replace(temp, path);written.append(path)
        except BaseException:
            for path, raw, result in zip(PATHS, before, after):
                if path in written:
                    require(path.read_bytes() == result, 'external writer blocks rollback; immutable snapshots retained')
                    temp = path.with_name('.' + path.name + '.r17-rollback.tmp')
                    require(not temp.exists(), 'existing rollback staging file')
                    temp.write_bytes(raw);os.replace(temp, path)
            raise
        finally:
            for temp in staged:
                temp.unlink(missing_ok=True)
        independent = []
        for path, raw, expected in zip(PATHS, before, after):
            actual = path.read_bytes()
            require(actual == expected, 'full applied file readback differs')
            key = 'id' if path == PATHS[0] else 'interior'
            _, bs, be, _ = owned(raw, key);_, ps, pe, _ = owned(actual, key)
            require(raw[:bs] == actual[:ps] and raw[be:] == actual[pe:], 'independent outside-owned proof failed')
            independent.append(dict(path=str(path.relative_to(ROOT)), prefix_exact=True, suffix_exact=True,
                prefix_sha256=sha(actual[:ps]), suffix_sha256=sha(actual[pe:])))
        write(application / 'outside-owned-bytes.independent.json', independent)
        summary.update(applied=True, application=str(application), after={str(p.relative_to(ROOT)): sha(p.read_bytes()) for p in PATHS})
        write(application / 'promotion.json', summary)
        for path in [snapshot / 'freeze.json', Path(__file__)]:
            (application / path.name).write_bytes(path.read_bytes())
    write(snapshot / ('application-summary.json' if args.apply else 'preflight.json'), summary)
    print(json.dumps(summary, indent=2))


if __name__ == '__main__':
    main()
