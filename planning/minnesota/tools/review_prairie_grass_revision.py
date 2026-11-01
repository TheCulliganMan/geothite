#!/usr/bin/env python3
"""Read-only proof: exact golden prairie pack delta, native pixels and collision.

Never writes packs or executes gameplay. --self-test corrupts in-memory values.
All source snapshots, original renders and conversion/crop stages are retained.
"""
import argparse
import base64
from copy import copy
import hashlib
import io
import json
from pathlib import Path

from PIL import Image,ImageDraw
from render_archive import save_render
from render_reference import SourceArt,layout_blocks,load_pack
from render_stages import render_stage
from review_authored_terrain_revision import strict_equal,file_identity,require
from review_boundary_basecamp_native import actor_scene
from review_north_shore import oriented_sprite

ROOT=Path(__file__).resolve().parents[3]
OUTPUT=ROOT/'planning/minnesota/renders/prairie-grass/proof'
SPECS=[('02','kanto','mn_prairie_kanto',128,[1,11],'kanto'),('16','mn_headwaters','mn_prairie_headwaters',131,[2,3],'johto')]
CAMERAS={'02':[('west-grass',(6,30)),('nursery',(40,14)),('seed-spur',(23,34))],
         '16':[('hayfield',(53,15)),('flood-meadow',(39,37)),('east-prairie',(91,35))]}

def changed(tree,path,value):
 """Copy only the edited branch; read-only mutation fixtures share other values."""
 if not path:return value
 result=copy(tree);result[path[0]]=changed(tree[path[0]],path[1:],value);return result

def physical(glyph,palette):return glyph if palette&8==0 else(glyph&127)+96

def sources():
 result=[]
 for n,source,target,first,grass,pixel_source in SPECS:
  paths=[ROOT/f'.local/r{n}-prairie-before.layout.json',ROOT/f'planning/minnesota/routes/MN_R{n}.layout.json',ROOT/f'planning/minnesota/routes/MN_R{n}.json']
  old,new,dossier=[json.loads(p.read_bytes())for p in paths]
  require(old['id']==new['id']==dossier['id']=='MN_R'+n,'route source identity')
  require(old['compiled_tileset']==source and new['compiled_tileset']==target,'route source tileset')
  require(old['dimensions_metatiles']==new['dimensions_metatiles']==dossier['dimensions_metatiles'],'route dimensions')
  strict_equal({k:v for k,v in old.items()if k not in{'layers','derived_blocks','compiled_tileset'}},{k:v for k,v in new.items()if k not in{'layers','derived_blocks','compiled_tileset'}})
  previous=old.get('derived_blocks',[]);recipes=new['derived_blocks'];strict_equal(previous,recipes[:len(previous)])
  expected=[dict(id=f'{first+i:02x}',source_tileset=pixel_source,source_block=f'{b:02x}',palette_override=7,collision=kind,required_typed_extension='build_prairie_grass_art_pack')for i,(b,kind)in enumerate(zip(grass,['FLOOR','TALL_GRASS']))]
  strict_equal(recipes[len(previous):],expected)
  before,after=layout_blocks(old),layout_blocks(new);w,h=old['dimensions_metatiles'];allow=dict(zip(grass,[first,first+1]))
  require(all(a==b or allow.get(a)==b for a,b in zip(before,after)),'only grass coloration may change source terrain')
  require(any(a!=b for a,b in zip(before,after)),'empty prairie source revision')
  chunks=[]
  for row in range((h+23)//24):
   for col in range((w+31)//32):
    x,y=col*32,row*24;cw,ch=min(32,w-x),min(24,h-y)
    clip=lambda bs:[b for yy in range(y,y+ch)for b in bs[yy*w+x:yy*w+x+cw]]
    chunks.append(dict(name=f'{dossier["map_prefix"]}C{col:02}R{row:02}',origin=[x,y],size=[cw,ch],before=clip(before),after=clip(after)))
  result.append(dict(number=n,source=source,target=target,first=first,grass=grass,pixel_source=pixel_source,old=old,new=new,dossier=dossier,chunks=chunks,paths=paths))
 require(sum(len(r['chunks'])for r in result)==3,'expected exactly three prairie exterior maps')
 return result

def validate_art(before,after,spec):
 source,target,first,grass=spec['source'],spec['target'],spec['first'],spec['grass'];files=after['runtime_files'];old_files=before['runtime_files'];old=before['data']['tilesets'][source];new=after['data']['tilesets'][target]
 require(target not in before['data']['tilesets'],'target tileset already exists')
 paths={f'data/tilesets/{target}_metatiles.bin',f'data/tilesets/{target}.json',f'data/tilesets/{target}_palette_map.json',f'gfx/tilesets/{target}.2bpp',f'gfx/tilesets/{target}.png'}
 require(not paths&set(old_files),'new prairie files overwrite existing paths')
 old_blocks=bytes(old_files[f'data/tilesets/{source}_metatiles.bin']);blocks=bytes(files[f'data/tilesets/{target}_metatiles.bin'])
 old_pixels=bytes(old_files[f'gfx/tilesets/{source}.2bpp']);pixels=bytes(files[f'gfx/tilesets/{target}.2bpp'])
 require(len(old_blocks)==first*16 and len(blocks)==len(old_blocks)+32 and blocks[:len(old_blocks)]==old_blocks,'original metatiles or append extent drift')
 require(len(old_pixels)in{96*16,192*16}and len(pixels)==192*16,'native glyph sheet extent')
 expected_pixels=bytearray(old_pixels+b'\0'*(len(pixels)-len(old_pixels)));expected_definition=copy(old);palette=list(old['palette_map']);collision=dict(old['collision']);used=set(old_blocks);protected={physical(g,old['palette_map'][g])for g in used};mapping={};new_slots=set()
 for i,block in enumerate(grass):
  kind='FLOOR'if i==0 else'TALL_GRASS';require(old['collision'][f'{block:02x}']==[kind]*4,'source grass collision authority')
  collision[f'{first+i:02x}']=list(old['collision'][f'{block:02x}'])
  for a,b in zip(old_blocks[block*16:(block+1)*16],blocks[(first+i)*16:(first+i+1)*16]):
   p=new['palette_map'][b];slot=physical(b,p);old_slot=physical(a,old['palette_map'][a])
   require((p&7)==7 and(p&~15)==0 and b==(slot if slot<96 else(slot-96)|128),'gold palette/bank addressing')
   require(b not in used and slot not in protected and slot<192,'new grass overwrites protected original glyph')
   if a in mapping:require(mapping[a]==(b,slot),'one source glyph copied inconsistently')
   else:
    require(slot not in new_slots and b not in{v[0]for v in mapping.values()},'distinct glyphs alias one replacement')
    mapping[a]=(b,slot);new_slots.add(slot)
   expected_pixels[slot*16:(slot+1)*16]=old_pixels[old_slot*16:(old_slot+1)*16];palette[b]=(8 if slot>=96 else 0)|7
 require(pixels==bytes(expected_pixels),'new graphics differ from exact source copies/untouched slots')
 expected_definition['palette_map']=palette;expected_definition['collision']=collision;strict_equal(expected_definition,new)
 for name,value in[(f'data/tilesets/{target}.json',collision),(f'data/tilesets/{target}_palette_map.json',palette)]:
  strict_equal(json.loads(bytes(files[name])),value)
  require(bytes(files[name])==json.dumps(value,sort_keys=isinstance(value,dict),separators=(',',':')).encode(),'noncanonical native JSON art file')
 image=Image.open(io.BytesIO(bytes(files[f'gfx/tilesets/{target}.png'])));require(image.mode=='L'and image.size==(128,96),'native PNG format/extent')
 for y in range(96):
  for x in range(128):
   offset=((y//8)*16+x//8)*16+(y%8)*2;bit=7-x%8;level=((pixels[offset]>>bit)&1)|(((pixels[offset+1]>>bit)&1)<<1)
   require(image.getpixel((x,y))==[255,170,85,0][level],'PNG differs from native 2bpp glyph pixel')
 return paths,dict(source=source,target=target,original_metatiles=first,new_metatiles=2,source_glyphs_copied=len(mapping),gold_palette=7,all_original_art_used_glyphs_preserved=True)

def compare(before,after,revisions):
 """Build the narrowly authorized output from the baseline, then compare all fields."""
 expected=before;added_files=set();added_tilesets=set();art_reports=[];chunks=[]
 for r in revisions:
  paths,art=validate_art(before,after,r);added_files|=paths;added_tilesets.add(r['target']);art_reports.append(art)
  expected=changed(expected,('data','tilesets'),dict(expected['data']['tilesets'],**{r['target']:after['data']['tilesets'][r['target']]}))
  for rule in ['cut','whirlpool']:
   old=before['data']['field_moves'][rule]['replacements'];aliases=dict(expected['data']['field_moves'][rule]['replacements'])
   require(r['target']not in old,'prairie field-move alias already exists')
   if r['source']in old:aliases[r['target']]=old[r['source']]
   expected=changed(expected,('data','field_moves',rule,'replacements'),aliases)
  names={c['name']for c in r['chunks']}
  for pack in [before,after]:require({n for n in pack['data']['maps']if n.startswith(r['dossier']['map_prefix']+'C')}==names,'missing/extra prairie exterior map')
  for c in r['chunks']:
   name=c['name'];old=before['data']['maps'][name];new=after['data']['maps'][name];attrs=old['attributes'];label=attrs['blocks_label']
   require([attrs['width'],attrs['height']]==c['size']and attrs['tileset_name']==r['source'],'old native extent or tileset')
   strict_equal(attrs,before['data']['map_attributes'][name]);strict_equal(old['blocks'],c['before']);strict_equal(new['blocks'],c['after'])
   require(all(type(b)is int and 0<=b<=255 for b in c['after']),'native block is not a byte')
   require(all(label!=m['attributes']['blocks_label']for n,m in before['data']['maps'].items()if n!=name),'block section alias')
   for pack,bs in [(before,c['before']),(after,c['after'])]:require(pack['data']['map_blocks'][label]==base64.b64encode(bytes(bs)).decode(),'noncanonical or stale base64 block payload')
   for a,b in zip(c['before'],c['after']):strict_equal(before['data']['tilesets'][r['source']]['collision'][f'{a:02x}'],after['data']['tilesets'][r['target']]['collision'][f'{b:02x}'])
   new_attrs=dict(attrs,tileset_name=r['target']);module=dict(old,blocks=c['after'],attributes=new_attrs)
   expected=changed(expected,('data','maps',name),module);expected=changed(expected,('data','map_attributes',name),new_attrs);expected=changed(expected,('data','map_blocks',label),base64.b64encode(bytes(c['after'])).decode())
   chunks.append(dict(map=name,blocks_label=label,metatiles=len(c['after']),collision_quadrants_compared=len(c['after'])*4,changed_blocks=sum(a!=b for a,b in zip(c['before'],c['after']))))
 require(set(after['data']['tilesets'])==set(before['data']['tilesets'])|added_tilesets,'exactly two added tilesets')
 require(set(after['runtime_files'])==set(before['runtime_files'])|added_files,'exactly ten added native runtime files')
 expected=changed(expected,('runtime_files',),dict(before['runtime_files'],**{p:after['runtime_files'][p]for p in added_files}))
 manifests=before['report']['manifests']+[f'minnesota-{r["target"]}-art-v1'for r in revisions]
 expected=changed(expected,('report','manifests'),manifests)
 require(before['identity']['content_hash']!=after['identity']['content_hash'],'new art/terrain requires changed identity')
 require(before['identity']['runtime_modpack_id']=='+'.join(before['report']['manifests']),'baseline runtime identity differs from manifest list')
 expected=changed(expected,('identity','runtime_modpack_id'),'+'.join(manifests))
 expected=changed(expected,('identity','content_hash'),after['identity']['content_hash'])
 strict_equal(expected,after)
 return dict(changed_chunks=chunks,art=art_reports,all_other_pack_fields_strictly_unchanged=True,source_art_audio_home_actors_parties_events_encounters_unchanged=True,added_runtime_files=sorted(added_files),added_tilesets=sorted(added_tilesets),manifest_append_exact=True,field_move_aliases_exact=True),expected

def negatives(before,validated,revisions):
 """Mutations share unchanged baseline branches after the full exact comparison."""
 results=[]
 def reject(name,path,value):
  altered=changed(validated,path,value)
  try:compare(before,altered,revisions)
  except(ValueError,KeyError,IndexError,AssertionError)as error:results.append(dict(case=name,rejected_by=str(error)));return
  raise ValueError('Negative accepted: '+name)
 route='MnR02C00R00';other=next(n for n,m in before['data']['maps'].items()if n not in{c['name']for r in revisions for c in r['chunks']}and m['objects'])
 actor=before['data']['maps'][other]['objects'][0];reject('unrelated actor moved',('data','maps',other,'objects',0,'x'),actor['x']+1)
 trainers=before['data']['trainers']['trainers'];trainer=next(k for k,v in trainers.items()if v['party']);level=trainers[trainer]['party'][0]['level'];reject('unrelated trainer party changed',('data','trainers','trainers',trainer,'party',0,'level'),level+1)
 reject('audio metadata changed',('audio_manifest',),dict(before['audio_manifest'],prairie_unrelated=True))
 sourcefile='gfx/tilesets/kanto.2bpp';value=list(validated['runtime_files'][sourcefile]);value[0]^=1;reject('original source glyph changed',('runtime_files',sourcefile),value)
 aliases=validated['data']['field_moves']['cut']['replacements'];alias='mn_prairie_kanto'if'mn_prairie_kanto'in aliases else'mn_prairie_headwaters';reject('Cut alias drift',('data','field_moves','cut','replacements',alias),{})
 bs=list(validated['data']['maps'][route]['blocks']);bs[0]=(bs[0]+1)%128;reject('one published block stale',('data','maps',route,'blocks'),bs)
 reject('grass encounter collision changed',('data','tilesets','mn_prairie_kanto','collision','81'),['FLOOR']*4)
 reject('regional attribute alias diverged',('data','map_attributes',route,'tileset_name'),'kanto')
 reject('original grass palette changed',('data','tilesets','kanto','palette_map',0),bool(before['data']['tilesets']['kanto']['palette_map'][0]))
 label=validated['data']['maps'][route]['attributes']['blocks_label'];reject('encoded map block mismatch',('data','map_blocks',label),'AA==')
 palette=list(validated['data']['tilesets']['mn_prairie_kanto']['palette_map']);blocks=bytes(validated['runtime_files']['data/tilesets/mn_prairie_kanto_metatiles.bin']);glyph=blocks[128*16];palette[glyph]=(palette[glyph]&8)|4;reject('gold glyph palette changed',('data','tilesets','mn_prairie_kanto','palette_map'),palette)
 return results

def render_proof(before,after,revisions,out):
 artifacts=[]
 def save(im,name):path=out/(name+'.png');save_render(im,path);artifacts.append(dict(path=str(path),mode=im.mode,size=list(im.size)));return path
 for r in revisions:
  n=r['number'];w,h=r['old']['dimensions_metatiles'];scenes={}
  for state,pack in [('before',before),('after',after)]:
   art=SourceArt(pack);terrain=Image.new('RGB',(w*32,h*32));actors=[]
   for c in r['chunks']:
    m=pack['data']['maps'][c['name']];x,y=c['origin'];a=m['attributes'];terrain.paste(art.render(a['tileset_name'],a['width'],a['height'],m['blocks']),(x*32,y*32));actors += [dict(o,x=o['x']+x*2,y=o['y']+y*2)for o in m['objects']]
   layout=r['old'if state=='before'else'new'];source=art.render(layout['tileset'],w,h,layout_blocks(layout),derived=layout.get('derived_blocks'))
   save(source,f'r{n}-{state}-source-terrain-original');save(terrain,f'r{n}-{state}-native-terrain-original');require(terrain.tobytes()==source.tobytes(),'native/source whole terrain pixel mismatch R'+n+' '+state)
   rgba=render_stage(terrain,'convert',out,f'r{n}-{state}-rgba','RGBA');save(rgba,f'r{n}-{state}-before-actors-rgba');scene=actor_scene(pack,rgba,actors);save(scene,f'r{n}-{state}-whole-rgba');rgb=render_stage(scene,'convert',out,f'r{n}-{state}-rgb','RGB');save(rgb,f'r{n}-{state}-whole-rgb');scenes[state]=scene
  panel=Image.new('RGB',(w*64+24,h*32+48),(19,29,40));draw=ImageDraw.Draw(panel)
  for i,state in enumerate(['before','after']):x=i*(w*32+12);draw.text((x+4,6),f'R{n} {state.upper()} / completed pack source pixels; static scene',fill='white');panel.paste(scenes[state],(x,30))
  save(panel,f'r{n}-whole-before-after')
  for label,(x,y)in CAMERAS[n]:
   panel=Image.new('RGB',(1304,618),(19,29,40));draw=ImageDraw.Draw(panel)
   for i,state in enumerate(['before','after']):
    crop=render_stage(scenes[state],'crop',out,f'r{n}-{state}-{label}',(x*16+8-80,y*16+8-72,x*16+8+80,y*16+8+72));save(crop,f'r{n}-{state}-{label}-original160');frame=render_stage(crop,'convert',out,f'r{n}-{state}-{label}-rgba','RGBA');frame.alpha_composite(oriented_sprite(after,'SPRITE_CHRIS','south'),(72,60));save(frame,f'r{n}-{state}-{label}-fixture160');big=render_stage(frame,'resize',out,f'r{n}-{state}-{label}-4x',(640,576),Image.Resampling.NEAREST);save(big,f'r{n}-{state}-{label}-4x');draw.text((i*652+6,6),f'{state.upper()} {label} core({x},{y}) / static camera fixture',fill='white');panel.paste(big,(i*652+6,30))
   save(panel,f'r{n}-{label}-before-after')
 return artifacts

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--before',type=Path,default=ROOT/'content-packs/minnesota-geography-review.crystalpack');p.add_argument('--after',type=Path,default=ROOT/'content-packs/minnesota-prairie-review.crystalpack');p.add_argument('--self-test',action='store_true');p.add_argument('--no-render',action='store_true');args=p.parse_args()
 before=load_pack(args.before);after=load_pack(args.after);revisions=sources();report,expected=compare(before,after,revisions)
 report.update(before=file_identity(args.before,before),after=file_identity(args.after,after),source_only_static_visualization=True,native_runtime_loading_and_gameplay_not_proven=True)
 out=OUTPUT/(before['identity']['content_hash'][:12]+'-to-'+after['identity']['content_hash'][:12]);out.mkdir(parents=True,exist_ok=True);snapshots=out/'sources';snapshots.mkdir(exist_ok=True);report['source_snapshots']=[]
 paths=[path for r in revisions for path in r['paths']]+[ROOT/'crates/crystal-assets/src/prairie_grass_art.rs',ROOT/'crates/crystal-assets/src/prairie_grass_modpack.rs',ROOT/'crates/crystal-assets/src/mutation_protocol.rs',ROOT/'planning/minnesota/tools/render_reference.py',ROOT/'planning/minnesota/tools/review_authored_terrain_revision.py',Path(__file__)]
 for path in paths:
  payload=path.read_bytes();digest=hashlib.sha256(payload).hexdigest();target=snapshots/(path.stem+'-'+digest+path.suffix);require(not target.exists()or target.read_bytes()==payload,'immutable source snapshot collision');target.write_bytes(payload);report['source_snapshots'].append(dict(path=str(path),sha256=digest,snapshot=str(target)))
 if args.self_test:report['negative_cases']=negatives(before,expected,revisions)
 if not args.no_render:report['image_artifacts']=render_proof(before,after,revisions,out)
 payload=json.dumps(report,indent=2)+'\n';target=out/('comparison-'+hashlib.sha256(payload.encode()).hexdigest()+'.json');target.write_text(payload);print(json.dumps(dict(ok=True,report=str(target),before=report['before'],after=report['after'],changed_chunks=report['changed_chunks'],negative_cases=len(report.get('negative_cases',[])),image_artifacts=len(report.get('image_artifacts',[]))),indent=2))
if __name__=='__main__':main()
