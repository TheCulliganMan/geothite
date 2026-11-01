#!/usr/bin/env python3
"""Supplement the immutable R18 candidate with paired source views/input courses."""
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import sys
from PIL import Image, ImageDraw
from render_archive import save_render
from render_reference import load_pack
from review_r15_refinement import graph
from review_r18_quality_next import course
from review_r18_shorewalk_refinement import render

ROOT=Path(__file__).resolve().parents[3]
def write(p,v):p.write_text(json.dumps(v,indent=2)+'\n')
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 out=Path(sys.argv[1]); p=load_pack(ROOT/'content-packs/minnesota-three-cities-tight-final-v02.crystalpack')
 d=json.loads((out/'MN_R18.candidate.json').read_text());l=json.loads((out/'MN_R18.layout.candidate.json').read_text());od=json.loads((out/'before-MN_R18.json').read_text());ol=json.loads((out/'before-MN_R18.layout.json').read_text())
 _,kind,allowed,_,_,hazards=graph(d,l,p)
 pair=out/'same-position-weakness-02';pair.mkdir(exist_ok=False)
 points=[('WEST_MARINA_FORK',[11,11]),('FORMER_EMPTY_CENTRE',[47,11]),('WEST_LAKE_BEND',[27,14]),('FINAL_WEAKEST',[70,8])]
 for label,sd,sl in [('before',od,ol),('candidate',d,l)]:
  sd=deepcopy(sd);sd['viewpoints']=[dict(id=n,tile=at)for n,at in points];render(sd,sl,p,pair,label)
 sheet=Image.new('RGB',(640,4*168),(19,25,33));draw=ImageDraw.Draw(sheet)
 for row,(name,at)in enumerate(points):
  draw.text((8,row*168+3),f'{name} core{at}: current660 / candidate source-art views',fill='white')
  for col,label in enumerate(['before','candidate']):
   # Keep exact160 originals and use a compact unscaled paired sheet for inspection.
   sheet.paste(Image.open(pair/f'{label}-{name}-native160.png'),(8+col*320,row*168+20))
 save_render(sheet,out/'same-position-weakest-before-after-native160.png')
 groups={
  'west_east_main':[[2,10],[11,11],[19,14],[27,14],[31,10],[35,13],[42,13],[45,13],[45,16],[48,16],[48,13],[49,19],[59,18],[63,18],[64,18],[71,18],[75,18],[76,10]],
  'protected_shore_return':[[76,10],[75,18],[71,18],[64,18],[63,18],[59,18],[49,19],[48,13]],
  'upper_inhabited_dune_return':[[75,18],[73,11],[65,11],[57,11],[49,11],[48,13]],
  'west_headland_return':[[48,13],[48,16],[45,16],[45,13],[42,13],[35,13],[31,10],[27,12],[27,14],[19,14],[11,11],[2,10]],
  'marina_and_both_working_piers':[[11,11],[13,23],[26,23],[29,23],[13,23],[13,28],[14,33],[19,34],[19,35],[13,35],[13,28],[11,11]],
  'original_walking_item_approaches':[[11,11],[12,31],[14,33],[26,7],[31,10],[35,13],[42,13],[45,16],[48,16],[48,13],[72,9]],
  'habitat_band_both_ways':[[35,13],[42,13],[35,13]],
  'shorewalk_second_native_row':[[59,18],[60,18],[60,17],[63,17],[64,17],[69,17],[69,18],[71,18]],
 }
 result={}
 for name,points in groups.items():
  result[name]=course(kind,allowed,hazards,points)
 dry={a for a in allowed if kind(a)=='FLOOR'}
 result['dry_9_step_bend']=course(kind,dry,hazards,[[45,13],[48,13],[45,13]])
 result['grass_3_step_choice']=course(kind,allowed,hazards,[[45,13],[48,13],[45,13]])
 for name in ['habitat_band_both_ways','grass_3_step_choice']:
  for leg in result[name]:leg['tall_grass_steps']=sum(kind(tuple(at))=='TALL_GRASS'for at in leg['path_core'][1:])
 # This is a navigable Surf geometry course, not an earned Surf proof. Native
 # populated controller acceptance remains the existing original Surf case.
 water={(x,y)for y in range(48)for x in range(80)if kind((x,y)) in {'FLOOR','TALL_GRASS','WATER'}}
 water-=set(tuple(a['tile'])for k in ['trainers','residents','items']for a in d[k])
 result['surf_geometry_after_actual_launch']=course(kind,water,hazards,[[49,20],[63,25],[64,25],[61,31],[61,32],[61,34]])
 write(out/'whole-route-input-courses.json',dict(status='UNEXECUTED input-only plans; not controller proof',initial='Empty party/position2,10/RNG only at setup for geometry. No runtime-state writes after setup. Separate existing legal populated SQ04/Surf fixtures test those mechanics.',execution='For each direction production press; repeat bounded8 only while facing/turn consumes input, assert each actual adjacent global step. Convertglobal x>=64 toMnR18C01R00localx-64. Avoid all six original trainer sight hazards. Ordinary graphs contain FLOOR/TALL_GRASS, actor bodies excluded.',courses=result,surf_native_transition='At49,19 faceDown and invoke genuine Surf dialogue/mount into49,20; use the original populated Surf controller fixture. Surf geometry course must never be executed as ordinary walking. Inspect real closed north/south/west/east water borders; no synthetic Surf state writes.',retained_quest_approaches={'REED':[14,33],'MEADOW':[31,10],'BEACH':[75,18],'RANGER':[26,23]},original_item_approaches={'HYPER_POTION':[12,31],'ULTRA_BALL':[26,7],'MAX_REPEL':[72,9],'SURF_REVIVE':[61,34]},native_bounds='Original native252visible closed outer border core cells retained. Shorewalk newFloor63/64seam on both rows17/18; realWaterseam unchanged.',scope='Native scripts/parties/flags/items/doors/connections/art/encounters unchanged. Original catalog cannot make valid full key pocket without an adequate rod; do not invent that fixture. Original SQ04 subsets/badge/rod/once/Continue and populated Surf tests will rerun separately.'))
 views=[]
 for name,at in [('west-fork',[11,11]),('headland-bank',[19,14]),('lake-bend',[27,14]),('habitat-neck',[38,13]),('dry-dune-turn',[45,16]),('former-empty-lawn',[47,11]),('public-surf-bank',[49,19]),('west-shorewalk',[59,18]),('east-shorewalk',[65,18]),('beach-report',[75,18]),('upper-working-pier',[25,23]),('lower-working-pier',[19,35]),('islet-discovery',[60,34]),('remaining-weakest',[70,8])]:
  assert kind(tuple(at)) in {'FLOOR','TALL_GRASS'} and tuple(at)not in [tuple(a['tile'])for k in ['trainers','residents','items']for a in d[k]],(name,at)
  views.append(dict(id=name,map=f'MnR18C{at[0]//64:02}R00',core=[at[0]%64,at[1]],global_core=at,source_collision=kind(tuple(at)),source_only_candidate=True))
 write(out/'native-views.json',views)
 write(out/'native-camera-limitations.json',{'scope':'These map/localcore targets are checked against candidate collision and native actor anchors, not published gameplay. Source160composites render the entire connected route. Actual captures must use the forthcoming revised native pack.', 'seam_camera_warning':'C01is only8metatileswide. The old single-module pixel comparator cannot prove connected-border/clamped seam frames without assembling neighbor terrain and measuring CHRISorigin. Preserve actualGPUframes; do not compare chunk-localx1/11 with an invented centered whole-map camera. These are valid native positions.', 'party':'Islet local placement in headless captures is only a local visual fixture; genuine Surf access remains separate original populated controller proof.'})
 write(out/'handoff-snapshot-sha256.json',{f.name:sha(f)for f in [out/'MN_R18.candidate.json',out/'MN_R18.layout.candidate.json',out/'native-preimage-contract.json',out/'review.json',out/'whole-route-input-courses.json',out/'native-views.json']})
 (out/'freeze_r18_quality_handoff.py.snapshot').write_bytes(Path(__file__).read_bytes())
 print(json.dumps({'status':'PASS','paired_cameras':4,'courses':{k:sum(v['steps']for v in legs)for k,legs in result.items()},'native_views':len(views)},indent=2))
if __name__=='__main__':main()
