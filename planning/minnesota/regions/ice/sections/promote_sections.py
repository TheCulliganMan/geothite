#!/usr/bin/env python3
"""Prepare or apply only Root-accepted winter route fields with complete byte guards.

No game pack is written. The default is a dry run. Applying requires an exact
Root receipt, completed native evidence, and the four original file preimages.
"""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import re
import sys
import uuid

ROOT = next(p for p in Path(__file__).resolve().parents if (p / 'Cargo.toml').is_file())
SOURCE = ROOT / 'planning/minnesota/regions/ice/sections/source-v04'
CONTRACT_SHA = '716a7394c78ab02908a68ce9fe8e3425694f03e6f13630f8458d0ad8ed24a865'
PACK_SHA = '92bb645535db23c78491b6982ddf5fa17b72b1f0434dfb6456a395fbf8fdbd33'
IDENTITY = '043b57b8814714bcf7b14f1571931eadbd139c12ad3e9afba596c193034c358f'
PACK = ROOT / 'content-packs/minnesota-northern-winter-v02.crystalpack'
WRITTEN = ROOT / 'planning/minnesota/renders/ice/sections-native/after-roof-source-v04/written-92bb6455-v02/proof-ee157f7ed01e70cdd6b411008ede107dee60498e3b1958d0c2287af421d843e4.json'
CONTROLLER = ROOT / '.local/winter-sections-controller-v04-receipt.json'
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from render_reference import layout_blocks


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def require(valid, context):
    if not valid:
        raise ValueError(context)


def relative(path):
    return str(path.resolve().relative_to(ROOT))


def top_spans(raw):
    """Locate UTF-8 byte spans of top-level values without reformatting JSON."""
    text = raw.decode('utf-8')
    decoder = json.JSONDecoder()
    def space(i):
        while i < len(text) and text[i].isspace():
            i += 1
        return i
    i = space(0)
    require(text[i] == '{', 'expected a JSON object')
    i = space(i + 1)
    found = {}
    while text[i] != '}':
        key, i = decoder.raw_decode(text, i)
        require(isinstance(key, str) and key not in found, 'duplicate/non-string object field')
        i = space(i)
        require(text[i] == ':', 'expected JSON colon')
        start = space(i + 1)
        _, end = decoder.raw_decode(text, start)
        found[key] = (len(text[:start].encode()), len(text[:end].encode()))
        i = space(end)
        if text[i] == '}':
            break
        require(text[i] == ',', 'expected JSON comma')
        i = space(i + 1)
    require(space(i + 1) == len(text), 'trailing JSON bytes')
    return found


def outside(raw, keys):
    spans = top_spans(raw)
    chunks, end = [], 0
    for key in sorted(keys, key=lambda k: spans[k][0]):
        a, b = spans[key]
        chunks.extend([raw[end:a], ('<OWNED:' + key + '>').encode()])
        end = b
    chunks.append(raw[end:])
    return b''.join(chunks)


def patch(raw, updates):
    spans = top_spans(raw)
    after = raw
    for key, value in sorted(updates.items(), key=lambda row: spans[row[0]][0], reverse=True):
        a, b = spans[key]
        line = raw[raw.rfind(b'\n', 0, a) + 1:a]
        indent = re.match(rb' *', line)[0].decode()
        replacement = json.dumps(value, indent=2, ensure_ascii=False).replace('\n', '\n' + indent).encode()
        after = after[:a] + replacement + after[b:]
    require(outside(raw, updates) == outside(after, updates), 'outside-owned field bytes changed')
    return after


def append(raw, fields):
    require(not set(fields) & json.loads(raw).keys(), 'new field already exists')
    stem = raw.rstrip()[:-1].rstrip()
    body = '\n'.join(json.dumps(fields, indent=2, ensure_ascii=False).splitlines()[1:-1]).encode()
    after = stem + b',\n' + body + b'\n}\n'
    require(after.startswith(stem) and after.endswith(raw[len(stem):]), 'outside appended fields changed')
    return after, {'existing_prefix_bytes': len(stem), 'existing_prefix_sha256': sha(stem),
                   'existing_suffix_sha256': sha(raw[len(stem):])}


def acceptance(path):
    require(sha((SOURCE / 'final-source.contract.json').read_bytes()) == CONTRACT_SHA, 'frozen source contract changed')
    contract = json.loads((SOURCE / 'final-source.contract.json').read_bytes())
    for filename, expected in contract['sources'].items():
        require(sha((ROOT / filename).read_bytes()) == expected, 'frozen source changed: ' + filename)
    require(sha(PACK.read_bytes()) == PACK_SHA, 'final native pack changed')
    proof = json.loads(WRITTEN.read_bytes())
    require(proof['status'] == 'READ_ONLY_FINAL_WRITTEN_SOURCE_ALIGNMENT_PASS'
            and proof['after_pack_sha256'] == PACK_SHA and proof['after_identity'] == IDENTITY
            and len(proof['checks']) == 701 and len(proof['negative_checks']) == 10
            and all(x['status'] == 'PASS' for x in proof['checks'])
            and all(x['rejected'] for x in proof['negative_checks']), 'final native preservation proof required')
    run = json.loads(CONTROLLER.read_bytes())
    require(run['exit_code'] == 0 and run['status'] == 'PASS' and run['pack_sha256'] == PACK_SHA
            and run['content_identity'] == IDENTITY and run['binary_readback_unchanged'], 'fresh final five-case controller receipt required')
    log = ROOT / run['log']
    require(sha(log.read_bytes()) == run['log_sha256']
            and '5 passed; 0 failed' in log.read_text(), 'fresh complete final controller log required')
    root = json.loads(path.read_bytes())
    binding = root.get('winter_sections_promotion', {})
    require(root.get('status', '').startswith('ROOT_ACCEPTED') and root.get('pack_sha256') == PACK_SHA
            and binding.get('source_contract_sha256') == CONTRACT_SHA, 'exact scoped Root acceptance required')
    for key in ['native_preservation_pass', 'native_pixel_comparison_pass', 'controller_pass', 'root_visual_acceptance']:
        require(binding.get(key) is True, 'missing Root acceptance gate: ' + key)
    require(root.get('evidence'), 'hash-bound Root evidence required')
    for row in root['evidence']:
        require(sha((ROOT / row['path']).read_bytes()) == row['sha256'], 'Root evidence drift: ' + row['path'])
    return dict(pack_sha256=PACK_SHA, content_identity=IDENTITY, source_contract_sha256=CONTRACT_SHA,
                root_acceptance=relative(path), root_acceptance_sha256=sha(path.read_bytes()),
                controller_receipt=relative(CONTROLLER), controller_receipt_sha256=sha(CONTROLLER.read_bytes()),
                native_written_proof=relative(WRITTEN), native_written_proof_sha256=sha(WRITTEN.read_bytes()),
                scope='Accepted winter terrain/art, preserved gates/resources, explicit local controller fixtures and matched native pixels. No earned campaign or completed International Falls claim.')


def proposals(evidence):
    result = []
    for route in ['17', '11']:
        for suffix in ['.layout', '']:
            path = ROOT / f'planning/minnesota/routes/MN_R{route}{suffix}.json'
            before = path.read_bytes()
            frozen_before = (SOURCE / f'MN_R{route}{suffix}.json.before').read_bytes()
            require(before == frozen_before, 'canonical full-file preimage changed: ' + relative(path))
            source = SOURCE / (f'MN_R{route}.layout.candidate.json' if suffix else f'MN_R{route}.candidate.json')
            frozen = json.loads(source.read_bytes())
            final = copy.deepcopy(frozen)
            if evidence:
                review = final['winter_section_review']
                review['status'] = 'ROOT_NATIVE_ACCEPTED_SCOPED_WINTER_TERRAIN_ART_AND_FIXTURE_GAMEPLAY'
                review['native_evidence'] = dict(evidence, candidate_file_sha256=sha(source.read_bytes()),
                                                 authoring_bundle_revision='v04')
            require({k:v for k,v in final.items() if k != 'winter_section_review'}
                    == {k:v for k,v in frozen.items() if k != 'winter_section_review'}, 'non-editorial frozen candidate changed')
            require({k:v for k,v in final['winter_section_review'].items() if k not in ['status','native_evidence']}
                    == {k:v for k,v in frozen['winter_section_review'].items() if k not in ['status','native_evidence']},
                    'winter identity/snowline/gate review changed outside explicit editorial fields')
            editorial_keys = sorted(k for k in set(final['winter_section_review']) | set(frozen['winter_section_review'])
                                    if final['winter_section_review'].get(k) != frozen['winter_section_review'].get(k))
            require(editorial_keys == (['native_evidence','status'] if evidence else []), 'unexpected editorial fields')
            if suffix:
                require(layout_blocks(final) == layout_blocks(frozen), 'editorial metadata altered native grid')
                owned = ['compiled_tileset', 'shore_fitting', 'layers']
                updates = {key:final[key] for key in owned}
                middle = patch(before, updates)
                byte_proof = {'outside_owned_spans_sha256':sha(outside(before, updates)),
                              'outside_owned_bytes_exact':outside(before, updates) == outside(middle, updates)}
                added = {'winter_section_review':final['winter_section_review']}
            else:
                require({k:v for k,v in final.items() if k not in ['compiled_tileset','winter_section_review']}
                        == json.loads(before), 'old complete dossier/story changed')
                middle = before
                added = {key:final[key] for key in ['compiled_tileset','winter_section_review']}
                byte_proof = {'all_old_dossier_fields_and_bytes_exact':True}
            after, appended = append(middle, added)
            require(json.loads(after) == final, 'proposed canonical object differs from scoped final candidate')
            result.append(dict(path=path,before=before,after=after,source=source,
                               byte_proof=dict(byte_proof, **appended),
                               editorial_delta={'winter_section_review':{'before':frozen['winter_section_review'],
                                                                         'after':final['winter_section_review']}}))
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root-acceptance',type=Path)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--apply',action='store_true')
    args = parser.parse_args()
    require(not args.apply or args.root_acceptance is not None, 'apply requires Root acceptance')
    evidence = acceptance(args.root_acceptance) if args.root_acceptance else None
    changes = proposals(evidence)
    shared = {ROOT / ('planning/minnesota/' + name): (ROOT / ('planning/minnesota/' + name)).read_bytes()
              for name in ['world.json','interiors.json']}
    args.output.mkdir(parents=True,exist_ok=False)
    for change in changes:
        for stage in ['before','after']:
            target = args.output / stage / change['path'].relative_to(ROOT)
            target.parent.mkdir(parents=True,exist_ok=True)
            target.write_bytes(change[stage])
    for path,raw in shared.items():
        target = args.output / 'before' / path.relative_to(ROOT)
        target.parent.mkdir(parents=True,exist_ok=True)
        target.write_bytes(raw)
    expected = {change['path']:change['before'] for change in changes}
    applied = []
    def guard():
        require(all(path.read_bytes() == raw for path,raw in expected.items()), 'canonical concurrent drift before write/readback')
        require(all(path.read_bytes() == raw for path,raw in shared.items()), 'world/interiors concurrent drift')
    receipt = dict(mode='apply' if args.apply else 'accepted-dry-run' if evidence else 'pending-dry-run',
                   applied=[],source_contract_sha256=CONTRACT_SHA,native_pack_sha256=PACK_SHA,
                   evidence=evidence,canonical_written=False,files=[],
                   shared_unchanged={relative(p):sha(raw) for p,raw in shared.items()})
    try:
        guard()
        if args.apply:
            for change in changes:
                path = change['path']
                temp = path.with_name('.' + path.name + '.winter-' + uuid.uuid4().hex + '.tmp')
                try:
                    with temp.open('xb') as stream:
                        stream.write(change['after']);stream.flush();os.fsync(stream.fileno())
                    os.chmod(temp,path.stat().st_mode & 0o777)
                    guard()
                    os.replace(temp,path)
                    expected[path] = change['after']
                    applied.append(relative(path))
                    guard()
                finally:
                    temp.unlink(missing_ok=True)
        for change in changes:
            actual = change['path'].read_bytes()
            require(actual == (change['after'] if args.apply else change['before']), 'independent canonical readback differs')
            # Regeneration from frozen inputs proves the same outside-byte result,
            # while the full actual readback equality covers added fields too.
            receipt['files'].append(dict(path=relative(change['path']),before_sha256=sha(change['before']),
                                         after_sha256=sha(change['after']),frozen_candidate_sha256=sha(change['source'].read_bytes()),
                                         outside_byte_proof=change['byte_proof'],editorial_delta=change['editorial_delta'],
                                         independent_readback_exact=True))
        guard();receipt.update(status='APPLIED' if args.apply else 'DRY_RUN_PASS',canonical_written=args.apply,applied=applied)
    except Exception as error:
        receipt.update(status='REJECTED_OR_PARTIAL_APPLY',error=str(error),canonical_written=bool(applied),applied=applied)
        (args.output/'failure-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
        raise
    raw=(json.dumps(receipt,indent=2)+'\n').encode();target=args.output/('receipt-'+sha(raw)+'.json');target.write_bytes(raw);print(target)


if __name__ == '__main__':
    main()
