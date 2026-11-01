#!/usr/bin/env python3
"""Prepare guarded source copies inside this proposal; never apply to canonical files."""
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import sys
ROOT=Path(__file__).resolve().parents[5]
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_reference import load_pack,layout_blocks


def sha(raw):return hashlib.sha256(raw).hexdigest()

def write(path,value):
    path.parent.mkdir(parents=True,exist_ok=True)
    path.write_text(json.dumps(value,indent=2)+'\n')

def rows(grid,width,height):
    result=[]
    for y in range(height):
        x=0
        while x<width:
            end=x+1
            while end<width and grid[y*width+end]==grid[y*width+x]:end+=1
            result.append(dict(name=f'Accepted native row {y:03} x{x:02}..{end-1:02}',kind='block_rect',bounds=[x,y,end,y+1],block=f'{grid[y*width+x]:02x}'))
            x=end
    return result


def main():
    assert not(HERE/'promotion.json').exists(),'Freeze each proposal epoch separately.'
    pack_path=ROOT/'content-packs/minnesota-river-long-v02.crystalpack'
    assert sha(pack_path.read_bytes())=='3ddbad16c6538e52f219c12497e4fbe6e41a7f18af5c42589b7b92357cfad243'
    pack=load_pack(pack_path);manifest_path=ROOT/'planning/minnesota/connections/mississippi-v02/manifest.json';manifest=json.loads(manifest_path.read_text())
    frozen=ROOT/'planning/minnesota/connections/river-continuity-long-v01/source-v02/candidate.contract.json';contract=json.loads(frozen.read_text())
    files=[];catchup=[]
    for prefix,source,target in [
        ('MnMississippiLowerWoods','planning/minnesota/connections/mississippi-v02/MnMississippiLowerWoods.layout.json','planning/minnesota/connections/mississippi-long-v01/MnMississippiLowerWoods.layout.json'),
        ('MnMississippiUpstreamWoods','planning/minnesota/connections/mississippi-v02/MnMississippiUpstreamWoods.layout.json','planning/minnesota/connections/mississippi-long-v01/MnMississippiUpstreamWoods.layout.json'),
        ('R05','planning/minnesota/routes/MN_R05.layout.json','planning/minnesota/routes/MN_R05.layout.json')]:
        source_path=ROOT/source;raw=source_path.read_bytes();before=json.loads(raw);width,height=before['dimensions_metatiles'];grid=[5]*(width*height)
        chunks=[r for r in manifest['new_maps']if r['id'].startswith(prefix)]if prefix!='R05'else[dict(id=f'MnR05C00R{i:02}',source_origin_metatiles=[0,i*24],dimensions_metatiles=[32,pack['data']['maps'][f'MnR05C00R{i:02}']['attributes']['height']])for i in range(3)]
        for r in chunks:
            ox,oy=r['source_origin_metatiles'];w,h=r['dimensions_metatiles'];m=pack['data']['maps'][r['id']]
            assert [m['attributes']['width'],m['attributes']['height']]==[w,h]
            for y in range(h):grid[(oy+y)*width+ox:(oy+y)*width+ox+w]=m['blocks'][y*w:(y+1)*w]
        candidate=deepcopy(before);candidate['layers']=rows(grid,width,height);candidate['shore_fitting']='explicit_source_edges'
        name=source_path.name;bp=HERE/'before'/name;cp=HERE/'candidate'/name;bp.parent.mkdir(exist_ok=True);bp.write_bytes(raw);write(cp,candidate)
        old_grid=layout_blocks(before);assert layout_blocks(candidate)==grid
        changed=[dict(metatile=[i%width,i//width],before=f'{a:02x}',after=f'{b:02x}')for i,(a,b)in enumerate(zip(old_grid,grid))if a!=b]
        if prefix=='R05':
            own=set()
            for r in contract['changes']:
                if r['map'].startswith('MnR05'):
                    row=int(r['map'][-2:])
                    own.update((c['metatile'][0],c['metatile'][1]+row*24)for c in r['cells'])
            catchup=[r for r in changed if tuple(r['metatile'])not in own]
            assert len(own)==126 and len(catchup)==21 and len(changed)==147
            rebuild=deepcopy(candidate);rebuild_grid=grid.copy()
            bridge=[]
            for x,old,native in [(10,0x44,0x83),(11,0x01,0x84),(12,0x46,0x85)]:
                assert grid[54*width+x]==native
                rebuild_grid[54*width+x]=old
                bridge.append(dict(metatile=[x,54],base_source=f'{old:02x}',installed_native=f'{native:02x}'))
            rebuild['layers']=rows(rebuild_grid,width,height);write(HERE/'alternatives'/'MN_R05.before-native-bridge-art.layout.json',rebuild)
        target_path=ROOT/target
        if source!=target:assert not target_path.exists(),('Proposed current source already exists',target)
        files.append(dict(source=source,source_sha256=sha(raw),before_file=str(bp.relative_to(HERE)),candidate_file=str(cp.relative_to(HERE)),candidate_sha256=sha(cp.read_bytes()),canonical_target=target,expected_target_sha256=sha(target_path.read_bytes())if target_path.exists()else None,changed_metatiles=len(changed),allowed_metadata_changes=[]if prefix!='R05'else['shore_fitting'],before_shore_fitting=before['shore_fitting'],candidate_shore_fitting=candidate['shore_fitting'],dimensions_metatiles=[width,height],chunks=chunks,changes=changed))
    guards=[]
    for f in ['planning/minnesota/world.json','planning/minnesota/interiors.json','planning/minnesota/routes/MN_R05.json','planning/minnesota/connections/mississippi-v02/manifest.json','planning/minnesota/connections/mississippi-v02/hand-authored-intent/MnMississippiLowerWoods.layout.json','planning/minnesota/connections/mississippi-v02/hand-authored-intent/MnMississippiUpstreamWoods.layout.json']:
        path=ROOT/f;guards.append(dict(path=f,sha256=sha(path.read_bytes())))
    inventory=dict(schema_version=1,status='CURRENT_FULL_SOURCE_PROPOSAL_NOT_A_BUILD_MANIFEST',accepted_native_pack_sha256=sha(pack_path.read_bytes()),source_contract_sha256=sha(frozen.read_bytes()),full_current_sources=[dict(path=r['canonical_target'],sha256=r['candidate_sha256'],dimensions_metatiles=r['dimensions_metatiles'])for r in files],historical_actor_connection_encounter_manifest='planning/minnesota/connections/mississippi-v02/manifest.json',historical_inputs_remain_immutable=True,world_route_dossier='planning/minnesota/routes/MN_R05.json',rebuild_limits=['Use the current installed native art for the snapshot source; R05 includes the existing native bridge blocks 83–85.','The shared Rust whole-route parser needs a narrow MN_R05 explicit_source_edges allowance before consuming the R05 snapshot.','The alternative R05 source retains base bridge blocks 44/01/46 for a build stage before native bridge-art installation; applying its three declared derived substitutions reproduces the accepted snapshot.','Do not run the old frozen river publisher on these new files or claim its historical preimages remain applicable. Historical mississippi-v02 files/manifest stay exact; current terrain publication still needs an explicit reviewed native overlay stage.'])
    write(HERE/'candidate'/'authoring.json',inventory)
    proposal=dict(status='GUARDED_CANONICAL_PROMOTION_PROPOSAL_NO_APPLY',accepted_native_pack=str(pack_path.relative_to(ROOT)),accepted_native_pack_sha256=sha(pack_path.read_bytes()),source_contract_sha256=sha(frozen.read_bytes()),files=files,guards=guards,r05_older_approved_source_catchup=catchup,r05_existing_derived_bridge_substitutions=bridge,current_inventory_target='planning/minnesota/connections/mississippi-long-v01/authoring.json',current_inventory_candidate='candidate/authoring.json',current_inventory_sha256=sha((HERE/'candidate/authoring.json').read_bytes()),current_inventory_target_must_be_absent=True,required_shared_rust_change='In authored_route_layout.rs allow explicit_source_edges for MN_R05 in the same narrowly scoped profile condition that already supports MN_R03. This proposal does not edit shared Rust.',no_canonical_write_performed=True)
    write(HERE/'promotion.json',proposal)
    print(json.dumps(dict(files=[dict(target=r['canonical_target'],before_sha256=r['expected_target_sha256'],candidate_sha256=r['candidate_sha256'],changed_metatiles=r['changed_metatiles'])for r in files],proposal_sha256=sha((HERE/'promotion.json').read_bytes())),indent=2))


if __name__=='__main__':main()
