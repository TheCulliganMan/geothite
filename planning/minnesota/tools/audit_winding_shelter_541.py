#!/usr/bin/env python3
"""Independent read-only bounded proof of the completed 541-map composition."""
import argparse
import base64
from copy import deepcopy
import hashlib
import io
import json
from pathlib import Path

import cbor2

from render_reference import layout_blocks, load_pack

ROOT=Path(__file__).resolve().parents[3]
INPUTS=[
    ('story','.local/minnesota-city-revisions/leech-shore-03c9d073629c60debc66368f07a4170d0b305026c84fd3d5fdcfd96f95c60a97.crystalpack','03c9d073629c60debc66368f07a4170d0b305026c84fd3d5fdcfd96f95c60a97'),
    ('geometry','content-packs/minnesota-northstar-map-quality.crystalpack','9aa950a35453f06cfee2e4a9b674c6d902af7c311f9ca64a083665ac6c60ddb2'),
    ('final','content-packs/minnesota-northstar-seep-shelter-review.crystalpack','7e5f65b935b1f74766c810ee72f444ad60a7e71e096b5dec10c46f516595344a')]
OWNED=['MnR03Cave1F','MnR05RockCut','MnR06C00R00','MnR08C00R01','MnR10C00R00','MnR10C01R00','MnR10Cave1F','MnR12Cave1F','MnR15C00R00','MnR16C00R00','MnR16C01R00','MnR17Cave1F','MnR19C00R00']
R10=['MnR10C00R00','MnR10C01R00']
ALIAS='mn_r10_rockstairs'
MANIFESTS=['minnesota-r10-rockstairs-art','minnesota-r10-rockstairs-native-stair']
ALIAS_FILES=sorted([f'data/tilesets/{ALIAS}.json',f'data/tilesets/{ALIAS}_metatiles.bin',f'data/tilesets/{ALIAS}_palette_map.json',f'gfx/tilesets/{ALIAS}.2bpp',f'gfx/tilesets/{ALIAS}.png'])
TOWNS=['MnGrandMarais','MnBoundaryWaters','MnGrandRapids','MnDuluth','MnBemidji','MnBrainerd','MnWinona','MnRochester','MnMankato','MnMinneapolis','MnStPaul','MnStCloud','MnLeechLake']
SOURCES={
    'R08':'planning/minnesota/renders/map-quality-541/integration-inputs/R08-candidate-with-541-caps.json',
    'R15':'planning/minnesota/renders/central-lakes/west-apron-20261027/source-handoff-d89922bc4a8bc34f/candidate/MN_R15.west-apron.layout.json',
    'R19':'planning/minnesota/renders/prairie-forest-link/prairie-approach/v01-oak-hook/MN_R19.layout.candidate.json',
    'R12':'planning/minnesota/renders/caves/r12-depth-candidate/v03-cliff-toe/MnR12Cave1F.depth.candidate.json'}


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def fingerprint(value):
    return sha(cbor2.dumps(value,canonical=True))


def raw_identity(path):
    """Rehash exact stored CBOR spans as Rust's five-element identity tuple.

    No alternate encoder chooses float widths or struct ordering. The native
    tuple hashes format/data/audio-manifest/runtime-files/report in that order.
    Raw spans remain in memory; no source or command catalog is written.
    """
    raw=path.read_bytes()[22:];stream=io.BytesIO(raw);head=stream.read(1)[0]
    assert head>>5==5 and (head&31)<24
    decoder=cbor2.CBORDecoder(stream);spans={};values={}
    for _ in range(head&31):
        key=decoder.decode();start=stream.tell();values[key]=decoder.decode();spans[key]=raw[start:stream.tell()]
    digest=sha(b'\x85'+b''.join(spans[k] for k in ['format_version','data','audio_manifest','runtime_files','report']))
    return dict(stored=values['identity']['content_hash'],independently_recomputed=digest,exact=digest==values['identity']['content_hash'])


def source_grids(geometry):
    expected={name:list(geometry['data']['maps'][name]['blocks']) for name in OWNED if name not in ['MnR08C00R01','MnR12Cave1F','MnR15C00R00','MnR19C00R00']}
    for route in ['R08','R15','R19']:
        layout=json.loads((ROOT/SOURCES[route]).read_bytes());blocks=layout_blocks(layout);w,h=layout['dimensions_metatiles']
        for cy in range((h+23)//24):
            for cx in range((w+31)//32):
                cw,ch=min(32,w-cx*32),min(24,h-cy*24)
                expected[f'Mn{route}C{cx:02}R{cy:02}']=[blocks[(cy*24+y)*w+cx*32+x] for y in range(ch) for x in range(cw)]
    room=json.loads((ROOT/SOURCES['R12']).read_bytes());expected[room['id']]=[int(v,16) for row in room['rows'] for v in row.split()]
    return expected


def audit(before,after,geometry,expected):
    checks=[]
    def check(label,valid):checks.append(dict(check=label,passed=bool(valid)))
    a,b=before['data'],after['data'];normalized=deepcopy(b)
    check('same541 exact native map roster',len(a['maps'])==len(b['maps'])==541 and set(a['maps'])==set(b['maps']))
    changed=sorted(name for name in a['maps'] if a['maps'][name]!=b['maps'].get(name));check('exact13 changed whole map modules',changed==OWNED)
    check('all528 unrelated whole modules exact',all(b['maps'].get(name)==module for name,module in a['maps'].items() if name not in OWNED))
    deltas=[]
    for name in OWNED:
        old,new=a['maps'][name],b['maps'][name];module=deepcopy(new);label=old['attributes']['blocks_label'];w=old['attributes']['width']
        check(name+' exact original dimensions and unique block label',[old['attributes']['width'],old['attributes']['height'],len(old['blocks']),label]==[new['attributes']['width'],new['attributes']['height'],len(new['blocks']),new['attributes']['blocks_label']] and sum(m['attributes']['blocks_label']==label for m in a['maps'].values())==1)
        check(name+' exact reviewed source geometry',new['blocks']==expected[name])
        module['blocks']=old['blocks']
        if name in R10:
            check(name+' only exact alias attribute',old['attributes']['tileset_name']=='johto' and new['attributes']['tileset_name']==ALIAS)
            module['attributes']['tileset_name']=old['attributes']['tileset_name'];normalized['map_attributes'][name]=a['map_attributes'][name]
        check(name+' complete typed programs actors events other fields exact',module==old)
        normalized['maps'][name]=old;normalized['map_blocks'][label]=a['map_blocks'][label]
        cells=[dict(at=[i%w,i//w],before=x,after=y) for i,(x,y) in enumerate(zip(old['blocks'],new['blocks'])) if x!=y]
        deltas.append(dict(map=name,changed_metatiles=len(cells),cells=cells))
    for name,blocks in expected.items():
        check(name+' complete frozen source grid exact incl untouched chunks',b['maps'][name]['blocks']==blocks)
    check('exact300 changed terrain metatiles',sum(v['changed_metatiles'] for v in deltas)==300)
    check('canonical original Repel200to100',a['items']['REPEL']['repel_steps']==200 and b['items']['REPEL']['repel_steps']==100)
    normalized['items']['REPEL']['repel_steps']=200
    check('exact new alias definition matches reviewed native9aa9',set(b['tilesets'])==set(a['tilesets'])|{ALIAS} and b['tilesets'][ALIAS]==geometry['data']['tilesets'][ALIAS])
    normalized['tilesets'].pop(ALIAS)
    check('ENTIRE GameDataSet exact after only declared normalization',normalized==a)
    for key in a:
        if key not in ['maps','map_attributes','map_blocks','items','tilesets']:
            check('whole data catalog exact '+key,a[key]==b[key])
    check('all541 native connection vectors and flags exact',all(m['attributes']['connections']==b['maps'][name]['attributes']['connections'] and m['attributes']['connection_flags']==b['maps'][name]['attributes']['connection_flags'] for name,m in a['maps'].items()))
    gyms=sorted(name for name in a['maps'] if name.startswith('Mn') and name.endswith('Gym'))
    check('all8 complete current gyms exact',len(gyms)==8 and all(a['maps'][name]==b['maps'][name] for name in gyms))
    family=sorted(name for name in a['maps'] if any(name.startswith(town) for town in TOWNS))
    check('all13 complete city exterior records exact',all(a['maps'][town]==b['maps'][town] for town in TOWNS))
    check('entire derived city room family exact',all(a['maps'][name]==b['maps'][name] for name in family))
    check('all541 attribute mirrors coherent',set(b['map_attributes'])==set(b['maps']) and all(b['map_attributes'][name]==m['attributes'] for name,m in b['maps'].items()))
    check('all541 NPC mirrors coherent',set(b['npcs'])==set(b['maps']) and all(b['npcs'][name]==m['objects'] for name,m in b['maps'].items()))
    check('all541 actual encoded block mirrors coherent',len(b['map_blocks'])==541 and all(list(base64.b64decode(b['map_blocks'][m['attributes']['blocks_label']],validate=True))==m['blocks'] for m in b['maps'].values()))
    raw=[];body_states={}
    for phase,data in [('before',a),('after',b)]:
        disagreements=[]
        for name,m in data['maps'].items():
            for label,commands in m['scripts'].items():
                if data['map_scripts'].get(label)!=commands:
                    disagreements.append(dict(map=name,label=label,cached_sha256=fingerprint(commands),global_sha256=fingerprint(data['map_scripts'].get(label))))
            if phase=='before':
                continue
            for attribute,field in [('map_scripts_label','map_script_section_commands'),('map_events_label','map_event_section_commands')]:
                original=data['map_scripts'].get(m['attributes'][attribute]);indexed=m[field]
                valid=original==[{k:v for k,v in row.items() if k!='command_index'} for row in indexed] and all(row['command_index']==i for i,row in enumerate(indexed))
                if not valid:raw.append(dict(map=name,field=field))
        body_states[phase]=sorted(disagreements,key=lambda v:(v['map'],v['label']))
    check('all541 raw section mirrors and derived sequential command indices coherent',not raw)
    check('all13 owned map script body mirrors coherent',not any(v['map'] in OWNED for v in body_states['after']))
    check('only exact unchanged9 baseline text body disagreements retained',len(body_states['before'])==9 and body_states['after']==body_states['before'])
    check('all original runtime art/source/audio files byte exact',all(after['runtime_files'].get(name)==data for name,data in before['runtime_files'].items()))
    added=sorted(set(after['runtime_files'])-set(before['runtime_files']));check('only exact five alias runtime files added',added==ALIAS_FILES)
    check('all five alias files byte exact reviewed9aa9',all(after['runtime_files'].get(name)==geometry['runtime_files'][name] for name in ALIAS_FILES))
    report=deepcopy(after['report']);check('only2 exact alias manifests appended',report['manifests']==before['report']['manifests']+MANIFESTS);report['manifests']=before['report']['manifests'];check('remaining whole compile report exact',report==before['report'])
    identity=deepcopy(after['identity']);check('derived runtime ID has only exact alias suffix',identity['runtime_modpack_id']==before['identity']['runtime_modpack_id']+'+'+'+'.join(MANIFESTS));check('content identity changes',identity['content_hash']!=before['identity']['content_hash']);identity['content_hash']=before['identity']['content_hash'];identity['runtime_modpack_id']=before['identity']['runtime_modpack_id'];check('all other identity fields exact',identity==before['identity'])
    for key in before:
        if key not in ['data','runtime_files','report','identity']:check('whole top-level native field exact '+key,before[key]==after[key])
    # Explicit source-declared shore caps, story closures and R10 city mouth
    # overlays remain protected even in revised route chunks.
    caps=[('MnR06C00R00',20,0,5),('MnR06C00R00',22,0,5)]+[('MnR06C00R01',x,19,5) for x in [6,7,9,10]]+[('MnR08C00R00',0,y,5) for y in [17,18]]+[('MnR08C00R01',x,23,5) for x in [11,12,14,15]]+[('MnR07C00R00',11,0,5),('MnR07C00R01',11,23,5)]+[('MnR03C00R01',x,7,5) for x in [7,8,10,11,13,18]]+[('MnBemidji',12,0,5),('MnBemidji',0,12,5),('MnBemidji',23,12,5),('MnBrainerd',16,0,1),('MnBrainerd',0,12,5),('MnBrainerd',31,12,5)]
    for name,x,y,value in caps:
        i=y*a['maps'][name]['attributes']['width']+x;check(name+f' explicit story cap/mouth{x},{y} exact',a['maps'][name]['blocks'][i]==b['maps'][name]['blocks'][i]==value)
    check('all5 installed R10 city-seam cells exact',all(a['maps'][name]['blocks'][y*a['maps'][name]['attributes']['width']+x]==b['maps'][name]['blocks'][y*a['maps'][name]['attributes']['width']+x]==v for name,x,y,v in [('MnR10C00R00',0,5,1),('MnR10C01R00',15,10,5),('MnR10C01R00',15,11,5),('MnR10C01R00',15,12,1),('MnR10C01R00',15,13,5)]))
    return checks,deltas,family,dict(section_errors=raw,retained_preexisting_text_body_disagreements=body_states['after'])


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--output',type=Path,required=True);args=parser.parse_args();assert not args.output.exists()
    packs={};inputs=[]
    for role,path,digest in INPUTS:
        file=ROOT/path;assert sha(file.read_bytes())==digest;packs[role]=load_pack(file);inputs.append(dict(role=role,path=path,file_sha256=digest,raw_content_identity=raw_identity(file)))
    before,geometry,after=[packs[k] for k in ['story','geometry','final']];expected=source_grids(geometry);checks,deltas,family,raw=audit(before,after,geometry,expected)
    checks.append(dict(check='all three content identities independently recomputed from exact native CBOR tuple spans',passed=all(v['raw_content_identity']['exact'] for v in inputs)))
    negatives=[]
    def data_mutation(field,mutator):
        bad=dict(after);bad['data']=dict(after['data']);bad['data'][field]=deepcopy(after['data'][field]);mutator(bad['data'][field]);return bad
    cases=[
        ('R12 actor movement even when NPC mirror follows',lambda:data_mutation('maps',lambda maps:maps['MnR12Cave1F']['objects'][0].update(x=6))),
        ('changed existing original Repel companion item',lambda:data_mutation('items',lambda items:items['SUPER_REPEL'].update(repel_steps=100))),
        ('lost R08 sixth story cap with coherent payload',lambda:data_mutation('maps',lambda maps:maps['MnR08C00R01']['blocks'].__setitem__(23*32+15,2))),
        ('wrong R12 terrain cell',lambda:data_mutation('maps',lambda maps:maps['MnR12Cave1F']['blocks'].__setitem__(4*16+10,2))),
        ('unrelated Pine gym terrain changed',lambda:data_mutation('maps',lambda maps:maps['MnBemidjiGym']['blocks'].__setitem__(0,2))),
        ('global story raw commands removed',lambda:data_mutation('map_scripts',lambda scripts:scripts.pop(next(iter(scripts))))),
        ('missing R10 alias runtime file',lambda:{**after,'runtime_files':{k:v for k,v in after['runtime_files'].items() if k!=ALIAS_FILES[0]}}),
        ('extra unsolicited runtime art',lambda:{**after,'runtime_files':after['runtime_files']|{'gfx/unrelated-unsolicited.png':[]}}),
    ]
    for label,mutator in cases:
        bad=mutator()
        # Keep the two explicitly mirrored corruptions coherent so equality and
        # source guards, rather than stale-mirror detection alone, reject them.
        if 'NPC mirror' in label:
            bad['data']['npcs']=dict(after['data']['npcs']);bad['data']['npcs']['MnR12Cave1F']=bad['data']['maps']['MnR12Cave1F']['objects']
        if 'coherent payload' in label:
            bad['data']['map_blocks']=dict(after['data']['map_blocks']);m=bad['data']['maps']['MnR08C00R01'];bad['data']['map_blocks'][m['attributes']['blocks_label']]=base64.b64encode(bytes(m['blocks'])).decode()
        result,_,_,_=audit(before,bad,geometry,expected);failed=[v['check'] for v in result if not v['passed']];assert failed,label;negatives.append(dict(mutation=label,rejected=True,failed_checks=failed))
    assert all(sha((ROOT/path).read_bytes())==digest for _,path,digest in INPUTS), 'Pack changed during audit'
    report=dict(schema_version=1,status='PASS' if all(v['passed'] for v in checks) else 'FAIL',scope='Read-only exact written-pack proof; no Cargo/export/publication/controller/GPU/save claim',inputs=inputs,checks=checks,changed_modules=deltas,changed_terrain_metatiles=sum(v['changed_metatiles'] for v in deltas),unchanged_whole_modules=528,protected_city_family_modules=family,raw_script_mirror_audit=raw,negative_cases_rejected=negatives,source_authorities=[dict(id=k,path=p,sha256=sha((ROOT/p).read_bytes())) for k,p in SOURCES.items()],new_alias_files=[dict(path=n,sha256=sha(bytes(after['runtime_files'][n]))) for n in ALIAS_FILES],identity_algorithm_reference='crates/crystal-assets/src/map_modules.rs derive_compiled_game_pack_identity_from_manifest + cbor_identity_hash.rs: SHA256 CBOR tuple(format_version,data,audio_manifest,runtime_files,report)',reviewer_sha256=sha(Path(__file__).read_bytes()))
    args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(dict(status=report['status'],checks=len(checks),negative_cases=len(negatives),changed_modules=len(deltas),changed_metatiles=report['changed_terrain_metatiles'],unchanged_modules=528,protected_city_family_modules=len(family),proof_sha256=sha(args.output.read_bytes())),indent=2));assert report['status']=='PASS',[v for v in checks if not v['passed']]


if __name__=='__main__':
    main()
