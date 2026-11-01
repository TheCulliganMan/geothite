#!/usr/bin/env python3
"""Independent exact read-only audit of the story-preserving geometry merge."""
import argparse
import base64
from copy import deepcopy
import hashlib
import json
from pathlib import Path

from render_reference import load_pack

ROOT = Path(__file__).resolve().parents[3]
SPECS = [
    ('quality', 'content-packs/minnesota-northstar-rock-stairs-review.crystalpack', '6071912eeb2fde6982f9e857bed9157d8d86da0f7ae56dba38665c809ab100e1'),
    ('story', 'content-packs/minnesota-northstar-lake-country.crystalpack', 'da27b381f69f64d6530383216077de0802c5b9dc42b12c316319cbc5daeb6d50'),
    ('merged', 'content-packs/minnesota-northstar-map-quality.crystalpack', '9aa950a35453f06cfee2e4a9b674c6d902af7c311f9ca64a083665ac6c60ddb2'),
]
OWNED = ['MnR03Cave1F','MnR05RockCut','MnR06C00R00','MnR10C00R00','MnR10C01R00','MnR10Cave1F','MnR16C00R00','MnR16C01R00','MnR17Cave1F']
R10 = ['MnR10C00R00','MnR10C01R00']
CAVES = ['MnR01Cave1F','MnR03Cave1F','MnR05RockCut','MnR09Cave1F','MnR10Cave1F','MnR12Cave1F','MnR13Cave1F','MnR17Cave1F','MnR20Cave1F']
ALIAS = 'mn_r10_rockstairs'
MANIFESTS = ['minnesota-r10-rockstairs-art','minnesota-r10-rockstairs-native-stair']


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--output',type=Path,required=True);args=parser.parse_args();assert not args.output.exists()
    packs={};identities=[]
    for label,path,digest in SPECS:
        file=ROOT/path;assert sha(file.read_bytes())==digest;packs[label]=load_pack(file);identities.append(dict(role=label,path=path,file_sha256=digest,content_identity=packs[label]['identity']['content_hash']))
    quality,before,after=[packs[k]for k in ['quality','story','merged']];q,b,a=[v['data']for v in [quality,before,after]];checks=[]
    def check(label,valid):
        checks.append(dict(check=label,passed=bool(valid)))
    check('exact same538 map roster',len(a['maps'])==len(b['maps'])==538 and set(a['maps'])==set(b['maps']))
    changed=[name for name in sorted(b['maps'])if a['maps'][name]!=b['maps'][name]]
    check('exact nine changed native modules',changed==OWNED)
    check('all529 other modules exact',all(a['maps'][name]==module for name,module in b['maps'].items()if name not in OWNED))
    normalized=deepcopy(a);deltas=[]
    for name in OWNED:
        old,new=b['maps'][name],a['maps'][name];module=deepcopy(new);width=old['attributes']['width'];cells=[dict(local_metatile=[i%width,i//width],before=f'{x:02x}',after=f'{y:02x}')for i,(x,y)in enumerate(zip(old['blocks'],new['blocks']))if x!=y]
        check(name+' retains native extent and block length',[old['attributes']['width'],old['attributes']['height'],len(old['blocks'])]==[new['attributes']['width'],new['attributes']['height'],len(new['blocks'])])
        module['blocks']=old['blocks']
        if name in R10:
            check(name+' exact dedicated alias',new['attributes']['tileset_name']==ALIAS and old['attributes']['tileset_name']=='johto')
            module['attributes']['tileset_name']=old['attributes']['tileset_name']
            normalized['map_attributes'][name]=b['map_attributes'][name]
        check(name+' all events/actors/programs/other fields exact',module==old)
        check(name+' split attributes exact cached',a['map_attributes'][name]==new['attributes'])
        label=new['attributes']['blocks_label'];check(name+' existing owned unique label retained',label==old['attributes']['blocks_label']and all(m['attributes'].get('blocks_label')!=label for other,m in a['maps'].items()if other!=name))
        check(name+' exact encoded block payload',list(base64.b64decode(a['map_blocks'][label]))==new['blocks'])
        normalized['maps'][name]=old;normalized['map_blocks'][label]=b['map_blocks'][label]
        deltas.append(dict(map=name,changed_metatile_count=len(cells),cells=cells,attribute_fields_changed=[k for k in old['attributes']if old['attributes'][k]!=new['attributes'].get(k)],payload_label=label))
    check('exact onlyREPEL duration change200to100',b['items']['REPEL']['repel_steps']==200 and a['items']['REPEL']['repel_steps']==100)
    normalized['items']['REPEL']['repel_steps']=200
    check('exact added R10 alias matches completed6071 definition',a['tilesets'][ALIAS]==q['tilesets'][ALIAS]and set(a['tilesets'])==set(b['tilesets'])|{ALIAS})
    normalized['tilesets'].pop(ALIAS)
    check('entire GameDataSet exact after only declared geometry alias duration normalization',normalized==b)
    for key in sorted(set(a)|set(b)):
        if key not in ['maps','map_attributes','map_blocks','items','tilesets']:
            check('entire story/catalog field exact '+key,a.get(key)==b.get(key))
    check('all538 connection vectors/flags exact',all(a['maps'][name]['attributes']['connections']==module['attributes']['connections']and a['maps'][name]['attributes']['connection_flags']==module['attributes']['connection_flags']for name,module in b['maps'].items()))
    gyms=[name for name in b['maps']if name.startswith('Mn')and name.endswith('Gym')];check('all eight story gyms whole modules exact',len(gyms)==8 and all(a['maps'][name]==b['maps'][name]for name in gyms))
    newtownmaps=sorted(set(b['maps'])-set(q['maps']));check('all28 newtown/room modules whole exact',len(newtownmaps)==28 and all(a['maps'][name]==b['maps'][name]for name in newtownmaps))
    check('all original runtime files byte exact',all(after['runtime_files'].get(name)==raw for name,raw in before['runtime_files'].items()))
    added=sorted(set(after['runtime_files'])-set(before['runtime_files']));expected=sorted([f'data/tilesets/{ALIAS}.json',f'data/tilesets/{ALIAS}_metatiles.bin',f'data/tilesets/{ALIAS}_palette_map.json',f'gfx/tilesets/{ALIAS}.2bpp',f'gfx/tilesets/{ALIAS}.png']);check('exact five new alias files',added==expected)
    check('new alias files byte exact6071 source',all(after['runtime_files'][name]==quality['runtime_files'][name]for name in added))
    report=deepcopy(after['report']);check('exact two appended alias manifests',report['manifests']==before['report']['manifests']+MANIFESTS);report['manifests']=before['report']['manifests'];check('remaining compile report exact',report==before['report'])
    for key in sorted(set(before)|set(after)):
        if key not in ['data','runtime_files','report','identity']:
            check('entire native pack field exact '+key,before.get(key)==after.get(key))
    identity=deepcopy(after['identity']);check('derived runtime ID only two expected manifests',identity['runtime_modpack_id']==before['identity']['runtime_modpack_id']+'+'+'+'.join(MANIFESTS));check('derived content hash changed',identity['content_hash']!=before['identity']['content_hash']);identity['runtime_modpack_id']=before['identity']['runtime_modpack_id'];identity['content_hash']=before['identity']['content_hash'];check('all remaining identity fields exact',identity==before['identity'])
    matches=[]
    for name in CAVES+['MnR06C00R00','MnR06C00R01','MnR16C00R00','MnR16C01R00']+R10:
        expected=list(q['maps'][name]['blocks']);caps=[]
        if name.startswith('MnR06'):
            points=[(20,0),(22,0)]if name.endswith('R00')else[(6,19),(7,19),(9,19),(10,19)]
            width=q['maps'][name]['attributes']['width']
            for x,y in points:
                i=y*width+x;check(name+f' story cap{x},{y} retains forest',expected[i]==2 and b['maps'][name]['blocks'][i]==a['maps'][name]['blocks'][i]==5);expected[i]=5;caps.append([x,y])
        check(name+' geometry matches6071 plus only specified retained caps',a['maps'][name]['blocks']==expected)
        matches.append(dict(map=name,extent_metatiles=[a['maps'][name]['attributes']['width'],a['maps'][name]['attributes']['height']],completed6071_match=True,retained_story_caps=caps))
    for name in ['MnR03C00R00','MnR03C00R01','MnBrainerd']:
        check(name+' whole story boundary/connection/ranger state preserved',a['maps'][name]==b['maps'][name])
    check('pack input file SHA stable after read',[sha((ROOT/path).read_bytes())for _,path,_ in SPECS]==[digest for _,_,digest in SPECS])
    result=dict(schema_version=1,status='PASS'if all(v['passed']for v in checks)else'FAIL',scope='Independent semantic whole-pack preservation proof of exact written packs; no Cargo/export/publication, native movement, renderer or save-migration claim.',packs=identities,checks=checks,changed_modules=deltas,changed_metatile_count=sum(v['changed_metatile_count']for v in deltas),unchanged_other_module_count=529,completed_geometry_matches=matches,preserved_eight_story_gyms=sorted(gyms),preserved28new_town_room_ids=newtownmaps,new_alias_runtime_files=[dict(path=name,sha256=sha(bytes(after['runtime_files'][name])))for name in added],repel=dict(before=200,after=100),derived_manifest_suffix=MANIFESTS,script_sha256=sha(Path(__file__).read_bytes()))
    args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(dict(status=result['status'],checks=len(checks),changed_maps=changed,changed_metatiles=result['changed_metatile_count'],unchanged_other_modules=529),indent=2));print('REPORT_SHA',sha(args.output.read_bytes()));assert result['status']=='PASS',[v for v in checks if not v['passed']]


if __name__=='__main__':main()
