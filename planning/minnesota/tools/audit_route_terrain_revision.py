#!/usr/bin/env python3
"""Independently check a written, terrain-only authored route composition.

Only declared route block arrays, uniquely owned encoded payloads and derived
content hashes may change. This is not GPU, controller or progression evidence.
"""
import argparse
import base64
from copy import deepcopy
import hashlib
import json
from pathlib import Path

from audit_city_expansions import fingerprint
from audit_winding_shelter_541 import raw_identity
from render_reference import layout_blocks, load_pack


def grids(layout, prefix):
    width, height = layout['dimensions_metatiles']
    blocks = layout_blocks(layout)
    return {f'{prefix}C{cx:02}R{cy:02}': [blocks[(cy*24+y)*width+cx*32+x]
            for y in range(min(24,height-cy*24)) for x in range(min(32,width-cx*32))]
            for cy in range((height+23)//24) for cx in range((width+31)//32)}


def audit(before, after, routes):
    checks, changes = [], []
    def equal(label, expected, actual):
        valid = expected == actual
        checks.append(dict(check=label, passed=valid, **({} if valid else dict(
            expected_sha256=fingerprint(expected), actual_sha256=fingerprint(actual)))))
    a, b = before['data'], after['data']
    equal('complete map roster unchanged', set(a['maps']), set(b['maps']))
    owned = {}
    for previous, revised, dossier in routes:
        equal(dossier['id']+' route identity', previous['id'], revised['id'])
        old = grids(previous, dossier['map_prefix']); new = grids(revised, dossier['map_prefix'])
        assert not set(owned) & set(old), 'repeated route declaration'
        owned.update({name:(old[name], new[name]) for name in old})
    normalized = dict(b, maps=dict(b['maps']), map_blocks=dict(b['map_blocks']))
    expected_changed = set()
    for name, (old_blocks, new_blocks) in sorted(owned.items()):
        old, new = a['maps'][name], b['maps'][name]
        equal(name+' complete old native source preimage', old_blocks, old['blocks'])
        equal(name+' complete new native source layout', new_blocks, new['blocks'])
        equal(name+' entire actor/event/script/attribute module preserved', old, dict(new,blocks=old['blocks']))
        label = old['attributes']['blocks_label']
        equal(name+' unique owned native block label', [name], sorted(n for n,m in a['maps'].items() if m['attributes']['blocks_label']==label))
        equal(name+' old encoded payload', old_blocks, list(base64.b64decode(a['map_blocks'][label],validate=True)))
        equal(name+' new encoded payload', new_blocks, list(base64.b64decode(b['map_blocks'][label],validate=True)))
        normalized['maps'][name] = old
        normalized['map_blocks'][label] = a['map_blocks'][label]
        if old_blocks != new_blocks:
            expected_changed.add(name)
            width = old['attributes']['width']
            changes.append(dict(map=name,cells=[dict(meta=[i%width,i//width],before=f'{x:02x}',after=f'{y:02x}')
                                               for i,(x,y) in enumerate(zip(old_blocks,new_blocks)) if x!=y]))
    equal('only exact declared changed native module roster', expected_changed,
          {name for name in a['maps'] if a['maps'][name] != b['maps'].get(name)})
    equal('ENTIRE data set preserved after only declared block arrays/payloads restored', a, normalized)
    for name in sorted(set(a['maps'])-set(owned)):
        equal(name+' unrelated complete native module exact', a['maps'][name], b['maps'].get(name))
    for key in before:
        if key not in ['data','identity']:
            equal('whole native top-level field exact '+key, before[key], after[key])
    equal('identity changes only its derived content hash', before['identity'], dict(after['identity'],content_hash=before['identity']['content_hash']))
    return checks, changes


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--before', type=Path, required=True)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--route', nargs=3, action='append', required=True, metavar=('BEFORE_LAYOUT','AFTER_LAYOUT','DOSSIER'))
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    inputs = [(path,path.read_bytes()) for path in [args.before,args.pack,*[Path(v) for row in args.route for v in row]]]
    before, after = load_pack(args.before), load_pack(args.pack)
    routes = [[json.loads(Path(path).read_text()) for path in row] for row in args.route]
    checks, changes = audit(before, after, routes)
    assert all(row['passed'] for row in checks), [row for row in checks if not row['passed']]
    owned = changes[0]['map']; other = next(name for name in before['data']['maps'] if name not in {row['map'] for row in changes})
    negatives = []
    for label, name, field in [('unrelated native map drift',other,'blocks'),('owned native actor drift',owned,'objects'),('wrong authored native terrain',owned,'blocks')]:
        bad = dict(after, data=dict(after['data'],maps=dict(after['data']['maps'])))
        module = deepcopy(after['data']['maps'][name]); bad['data']['maps'][name] = module
        if field == 'blocks':
            module['blocks'][0] = (module['blocks'][0]+1)%256
            bad['data']['map_blocks'] = dict(after['data']['map_blocks'])
            bad['data']['map_blocks'][module['attributes']['blocks_label']] = base64.b64encode(bytes(module['blocks'])).decode()
        else:
            module['objects'][0]['x'] += 1
            bad['data']['npcs'] = dict(after['data']['npcs'], **{name:module['objects']})
        rejected, _ = audit(before,bad,routes)
        failed = [row['check'] for row in rejected if not row['passed']]
        assert failed, label
        negatives.append(dict(mutation=label,rejected=True,failed_checks=failed))
    bad = dict(after, runtime_files=dict(after['runtime_files']))
    file = next(iter(bad['runtime_files'])); bad['runtime_files'][file] = b'corrupted in memory'
    rejected,_ = audit(before,bad,routes); failed = [row['check'] for row in rejected if not row['passed']]
    assert failed; negatives.append(dict(mutation='unrelated runtime art/audio/source drift',rejected=True,failed_checks=failed))
    assert all(path.read_bytes()==raw for path,raw in inputs), 'input changed during audit'
    identities = {str(path):raw_identity(path) for path in [args.before,args.pack]}
    assert all(value['exact'] for value in identities.values()), identities
    report = dict(status='PASS',scope=__doc__,inputs=[dict(path=str(path),sha256=hashlib.sha256(raw).hexdigest()) for path,raw in inputs],
                  checks=checks,changed_modules=changes,changed_metatiles=sum(len(row['cells']) for row in changes),
                  negative_cases=negatives,raw_content_identities=identities,reviewer_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest())
    raw = (json.dumps(report,indent=2)+'\n').encode()
    args.output.mkdir(parents=True,exist_ok=False)
    path = args.output/('proof-'+hashlib.sha256(raw).hexdigest()+'.json'); path.write_bytes(raw)
    print(json.dumps(dict(status='PASS',checks=len(checks),changed_modules=len(changes),changed_metatiles=report['changed_metatiles'],negatives=len(negatives),proof=str(path))))


if __name__=='__main__':
    main()
