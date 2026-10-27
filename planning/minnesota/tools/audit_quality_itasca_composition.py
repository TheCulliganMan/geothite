#!/usr/bin/env python3
"""Read-only preservation proof of quality terrain on the newer Itasca story."""
import base64
from copy import deepcopy
import hashlib
import json
from pathlib import Path

from audit_winding_shelter_541 import OWNED, ALIAS, ALIAS_FILES, MANIFESTS, raw_identity
from render_reference import load_pack

ROOT = Path(__file__).resolve().parents[3]


def audit(story, quality, final):
    checks = []
    def check(name, passed):
        checks.append(dict(check=name, passed=bool(passed)))
    a, q, b = story['data'], quality['data'], final['data']
    check('same 543-map native roster', len(a['maps']) == len(b['maps']) == 543 and set(a['maps']) == set(b['maps']))
    changed = sorted(name for name in a['maps'] if a['maps'][name] != b['maps'][name])
    check('exact 13 reviewed terrain modules change', changed == OWNED)
    check('all 530 other complete native modules exact', all(b['maps'][name] == module for name, module in a['maps'].items() if name not in OWNED))
    normalized = deepcopy(b)
    for name in OWNED:
        old, new = a['maps'][name], b['maps'][name]
        check(name + ' exact reviewed native quality module', new == q['maps'][name])
        normalized['maps'][name] = old
        normalized['map_attributes'][name] = a['map_attributes'][name]
        label = old['attributes']['blocks_label']
        normalized['map_blocks'][label] = a['map_blocks'][label]
    check('Repel exact native 200 to 100 correction', a['items']['REPEL']['repel_steps'] == 200 and b['items']['REPEL']['repel_steps'] == 100)
    normalized['items']['REPEL']['repel_steps'] = 200
    check('only reviewed R10 alias added', set(b['tilesets']) == set(a['tilesets']) | {ALIAS} and b['tilesets'][ALIAS] == q['tilesets'][ALIAS])
    normalized['tilesets'].pop(ALIAS, None)
    check('ENTIRE GameDataSet exact after declared normalization', normalized == a)
    check('original runtime files byte exact', all(final['runtime_files'].get(path) == raw for path, raw in story['runtime_files'].items()))
    check('exact five reviewed alias files added', sorted(set(final['runtime_files']) - set(story['runtime_files'])) == ALIAS_FILES and all(final['runtime_files'][path] == quality['runtime_files'][path] for path in ALIAS_FILES))
    report = deepcopy(final['report'])
    check('only two quality manifests append to newer story', report['manifests'] == story['report']['manifests'] + MANIFESTS)
    report['manifests'] = story['report']['manifests']
    check('entire remaining report exact', report == story['report'])
    for key in story:
        if key not in ['identity', 'data', 'runtime_files', 'report']:
            check('exact top-level ' + key, final[key] == story[key])
    for name, module in b['maps'].items():
        coherent = b['map_attributes'][name] == module['attributes'] and b['npcs'][name] == module['objects']
        coherent &= list(base64.b64decode(b['map_blocks'][module['attributes']['blocks_label']], validate=True)) == module['blocks']
        check(name + ' typed attributes/NPC/block mirrors coherent', coherent)
    return checks


def main():
    paths = [ROOT / p for p in [
        'content-packs/minnesota-northstar-itasca-story.crystalpack',
        'content-packs/minnesota-northstar-seep-shelter-review.crystalpack',
        'content-packs/minnesota-northstar-itasca-quality.crystalpack']]
    packs = [load_pack(path) for path in paths]
    checks = audit(*packs)
    identities = [dict(path=str(path.relative_to(ROOT)), sha256=hashlib.sha256(path.read_bytes()).hexdigest(), **raw_identity(path)) for path in paths]
    assert all(identity['exact'] for identity in identities)
    negatives = []
    for label, mutate in [
        ('unrelated original room changed', lambda pack: pack['data']['maps']['MnStPaulUnionDepot']['blocks'].__setitem__(0, 255)),
        ('reviewed geometry corrupted', lambda pack: pack['data']['maps']['MnR12Cave1F']['blocks'].__setitem__(0, 255)),
        ('old checkpoint program lost', lambda pack: pack['data']['maps']['MnDuluth']['scripts'].clear()),
    ]:
        damaged = deepcopy(packs[2]); mutate(damaged)
        failed = [check['check'] for check in audit(packs[0], packs[1], damaged) if not check['passed']]
        negatives.append(dict(corruption=label, rejected=bool(failed), failed_checks=failed))
    result = dict(status='PASS' if all(c['passed'] for c in checks) else 'FAIL', inputs=identities, checks=checks, corruption_negatives=negatives)
    output = ROOT / 'planning/minnesota/reviews/itasca-quality-543-proof.json'
    output.write_text(json.dumps(result, indent=2) + '\n')
    print(result['status'], len(checks), 'checks;', len(negatives), 'corruption negatives')
    assert all(c['passed'] for c in checks), [c for c in checks if not c['passed']]
    assert all(c['rejected'] for c in negatives)


if __name__ == '__main__':
    main()
