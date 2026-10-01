#!/usr/bin/env python3
"""Check complete native furniture identities and original rig/mesh fitting.

External pack data is used only in temporary fixtures. No Cargo, Blender,
GUI, new geometry, source catalog or gameplay changes are involved.
"""
from pathlib import Path
from model_asset_storage import read_model_json as load
import argparse, importlib.util, json, math, shutil, subprocess, tempfile
ROOT = Path(__file__).resolve().parent.parent

def vertices(part):
    return list(zip(*[iter(part['positions'])] * 3))

def bounds(points):
    return [min(p[k] for p in points) for k in range(3)], [max(p[k] for p in points) for k in range(3)]

def intersects(a, b):
    return all(a[1][k] > b[0][k] + 1e-5 and b[1][k] > a[0][k] + 1e-5 for k in range(3))

def fit(model, target):
    lo, hi = bounds([v for p in model['primitives'] for v in vertices(p)])
    west, east, north, south, rise = target
    scale = [(east-west)/(hi[0]-lo[0]), rise/(hi[1]-lo[1]), (south-north)/(hi[2]-lo[2])]
    out = []
    for part in model['primitives']:
        vv = [tuple((v[k]-lo[k])*scale[k]+[west,0,north][k] for k in range(3)) for v in vertices(part)]
        assert all(math.isfinite(c) for v in vv for c in v)
        assert all(west-1e-5 <= x <= east+1e-5 and -1e-5 <= y <= rise+1e-5 and north-1e-5 <= z <= south+1e-5 for x,y,z in vv)
        for normal in zip(*[iter(part['normals'])]*3):
            n = [normal[k]/scale[k] for k in range(3)]
            length = math.sqrt(sum(c*c for c in n))
            assert math.isfinite(length) and length > 0
            assert abs(sum((c/length)**2 for c in n)-1) < 1e-8
        out.append((part.get('part',part.get('name','component')), bounds(vv)))
    return out

def check_geometry(root):
    models = root/'crates/crystal-voxel-view/models'
    library = load(models/'johto_characters/shared.geometry.json')
    checks = 0
    for name, actors in [('interiors/arcade_stool',['super_nerd','gym_guide']),
                         ('dungeons/ship_stool',['sailor','teacher','youngster','pokefan_m','bug_catcher'])]:
        parts = fit(load(models/(name+'.mesh.json')), [2,14,0,12,7])
        for actor in actors:
            rig = load(models/'johto_characters'/(actor+'.rig.json'))
            translations, actor_parts = [], []
            for joint in rig['joints']:
                t = joint['translation']
                if joint['parent'] is not None:
                    t = [v+w for v,w in zip(t, translations[joint['parent']])]
                translations.append(t)
                for p in joint['primitives']:
                    actor_parts.append((joint['name'],[tuple((a+b)*16 for a,b in zip(v,t)) for v in vertices(library['geometries'][p['geometry']])]))
            for degree in range(0,360,5):
                yaw = math.radians(degree)
                for label, vv in actor_parts:
                    bb = bounds([(8+x*math.cos(yaw)+z*math.sin(yaw), y+.04, 16-x*math.sin(yaw)+z*math.cos(yaw)) for x,y,z in vv])
                    assert not any(intersects(bb,b) for _,b in parts), (actor,degree,label)
                checks += 1
    bunk = load(models/'dungeons/ship_bunk.mesh.json')
    fit(bunk, [0,16,0,32,7])
    assert bunk['name'] == 'ship_bunk' and sum(len(p['indices']) for p in bunk['primitives']) > 300
    print(f'PASS geometry: canonical round stools clear 7 actual rest rigs at 72 yaw samples each ({checks} poses); complete bunk fits 16x32x7 with finite transformed normals')
    print('LIMIT: rest geometry and source-contained bounds are checked; idle/walk animation and camera readability require native review')

BODY = r'''
use source::{Asset,Identity};
fn placements(a:Asset,cells:&[Identity],w:usize,h:usize)->Vec<(usize,usize)>{
 let mut out=Vec::new();for y in 0..h{for x in 0..w{
  if source::complete(a,w,h,x,y,|xx,yy|cells.get(yy*w+xx).copied()){out.push((x,y));}
 }}out
}
fn intended_bed_read(map:&str,a:Asset,origin:(usize,usize),event:(usize,usize,&str,&str))->bool{
 map=="FastShipCabins_SW_SSW_NW" && a==Asset::JoinedBunk && origin==(14,2)
  && matches!(event,(14,2|4,"BGEVENT_READ","FastShipBed"))
}
fn check(map:&str,cells:Vec<Identity>,w:usize,h:usize,expected:[usize;4],warp:&[(usize,usize)],coord:&[(usize,usize)],bg:&[(usize,usize,&str,&str)]){
 assert!(source::applies(map));let assets=[Asset::StoolEast,Asset::StoolWest,Asset::FullBunk,Asset::JoinedBunk];
 for (a,count) in assets.into_iter().zip(expected){
  let ps=placements(a,&cells,w,h);assert_eq!(ps.len(),count,"{map} {a:?}");
  if a==Asset::JoinedBunk {
   let exact=match map {
    "FastShipCabins_NNW_NNE_NE"|"FastShipCabins_SW_SSW_NW"=>vec![(14,2)],
    "FastShipCabins_SE_SSE_CaptainsCabin"=>vec![(14,26),(18,50)],
    "FastShipB1F"=>vec![],_=>unreachable!(),
   };
   assert_eq!(ps,exact,"joined berths retain the inspected source coordinates");
  }
  for (x,y) in ps{
   let (aw,ah,_)=a.size();
   for &(wx,wy) in warp {assert!(x+aw<=wx||wx+2<=x||y+ah<=wy||wy+2<=y,"{map} {a:?} warp overlap");}
   for dy in 0..ah{for dx in 0..aw{for field in 0..5{
    let mut changed=cells.clone();let s=&mut changed[(y+dy)*w+x+dx];
    match field{0=>s.0^=1,1=>s.1^=1,2=>s.2^=1,3=>s.3^=1,_=>{}}
    assert!(!source::complete(a,w,h,x,y,|xx,yy|{
      if field==4&&(xx,yy)==(x+dx,y+dy){None}else{changed.get(yy*w+xx).copied()}
    }),"every art/block/phase/atlas field guards {map} {a:?}");
   }}}
   assert!(!source::complete(a,x+aw-1,h,x,y,|xx,yy|cells.get(yy*w+xx).copied()));
   assert!(!source::complete(a,w,y+ah-1,x,y,|xx,yy|cells.get(yy*w+xx).copied()));
  }
 }
 // Inspect every ship-room asset, using its actual sparse claims so native
 // bulkhead openings stay available. Only the exact player's berth reads may
 // overlap; no map-wide, asset-wide or script-name-only event exception.
 let mut bed_reads=Vec::new();
 for a in Asset::ALL {for (x,y) in placements(a,&cells,w,h){
  let (aw,ah,_)=a.size();
  let overlaps=|ex:usize,ey:usize|(0..ah).any(|dy|(0..aw).any(|dx|
   a.claim(dx,dy)&&x+dx>=ex&&x+dx<ex+2&&y+dy>=ey&&y+dy<ey+2));
  for &(ex,ey) in warp.iter().chain(coord.iter()){
   assert!(!overlaps(ex,ey),"{map} {a:?} warp/coordinate-event overlap at {ex},{ey}");
  }
  for &(ex,ey,kind,script) in bg {if overlaps(ex,ey){
   assert!(intended_bed_read(map,a,(x,y),(ex,ey,kind,script)),
    "unexpected background interaction: {map} {a:?} {ex},{ey} {kind} {script}");
   bed_reads.push((ex,ey));
  }}
 }}
 bed_reads.sort();
 assert_eq!(bed_reads,if map=="FastShipCabins_SW_SSW_NW"{vec![(14,2),(14,4)]}else{vec![]});
 assert!(!intended_bed_read(map,Asset::FullBunk,(14,2),(14,4,"BGEVENT_READ","FastShipBed")));
 assert!(!intended_bed_read(map,Asset::JoinedBunk,(14,2),(14,6,"BGEVENT_READ","FastShipBed")));
 assert!(!intended_bed_read(map,Asset::JoinedBunk,(14,2),(14,4,"BGEVENT_READ","OtherScript")));
 assert!(!source::applies("FastShipB1FBeta"));assert!(!source::applies("OlivineLighthouse1F"));
 println!("PASS {map}: furniture {:?}; full source/crop guards, no warp/coord overlap, exact bed-read background interactions {:?}",expected,bed_reads);
}
'''

def check_pack(root, source_root, pack_path, rustc):
    spec=importlib.util.spec_from_file_location('facility_check',root/'tools/check-facility-radio.py')
    helper=importlib.util.module_from_spec(spec);spec.loader.exec_module(helper)
    pack=helper.load_pack(pack_path)
    module=source_root/'crates/crystal-voxel-view/src/mesh/ship_room_sources.rs'
    code=f'#[path="{module}"]mod source;\n'+BODY+'\nfn main(){\n'
    cases={'FastShipB1F':[3,3,8,0],'FastShipCabins_NNW_NNE_NE':[3,3,2,1],
           'FastShipCabins_SE_SSE_CaptainsCabin':[5,1,2,2],'FastShipCabins_SW_SSW_NW':[4,4,3,1]}
    for name,expected in cases.items():
        m=pack['data']['maps'][name];w=m['attributes']['width']*4;h=m['attributes']['height']*4
        assert m['attributes']['tileset_name']=='lighthouse'
        raw=pack['runtime_files']['data/tilesets/lighthouse_metatiles.bin']
        cells=[]
        for y in range(h):
            for x in range(w):
                block=m['blocks'][y//4*(w//4)+x//4]
                cells.append((block,x%4,y%4,raw[block*16+y%4*4+x%4]))
        warp=[(o['x']*2,o['y']*2)for o in m['events']['warps']]
        coord=[(o['x']*2,o['y']*2)for o in m['events']['coord_events']]
        background=m['events']['bg_events']
        if name=='FastShipCabins_SW_SSW_NW':
            bed_reads=[(o['x'],o['y'],o['event_type']) for o in background if o['script']=='FastShipBed']
            assert sorted(bed_reads)==[(7,1,'BGEVENT_READ'),(7,2,'BGEVENT_READ')]
            # Prove these are the native sleep/heal interaction, not merely an
            # identically named background callback. Keep the source external.
            script=m['scripts']['FastShipBed']
            for command,args in [('writetext',['FastShipBedText1']),('special',['FadeOutToBlack']),
                                 ('special',['HealParty']),('special',['FadeInFromBlack']),
                                 ('writetext',['FastShipBedText2']),('checkevent',['EVENT_FAST_SHIP_HAS_ARRIVED'])]:
                assert any(step['command']==command and step['args']==args for step in script)
            collision=pack['data']['tilesets']['lighthouse']['collision']
            assert collision['38'][3]=='WALL' and collision['39'][1]=='WALL'
        bg='['+','.join(f"({o['x']*2},{o['y']*2},{json.dumps(o['event_type'])},{json.dumps(o['script'])})" for o in background)+']'
        code+=f'check("{name}",vec!{cells!r},{w},{h},{expected!r},&{warp!r},&{coord!r},&{bg});\n'
    code+='}\n'
    with tempfile.TemporaryDirectory(prefix='geothite-room-furniture-') as temp:
        temp=Path(temp);(temp/'check.rs').write_text(code)
        subprocess.run([rustc,'--edition=2024','-A','dead_code',str(temp/'check.rs'),'-o',str(temp/'check')],check=True)
        subprocess.run([str(temp/'check')],check=True)
        # The production source module tests include every new exact selector.
        (temp/'source-tests.rs').write_text(f'#[path="{module}"]mod source;\n')
        subprocess.run([rustc,'--edition=2024','--test','-A','dead_code',str(temp/'source-tests.rs'),'-o',str(temp/'source-tests')],check=True)
        subprocess.run([str(temp/'source-tests')],check=True)
    print('PASS ship accounting: 26 round stools (104 cells), 19 complete beds (152 cells); joined38/39 berths own32 cells including16 additional bed-foot cells')

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root',type=Path,default=ROOT)
    parser.add_argument('--source-root',type=Path)
    parser.add_argument('--pack',type=Path)
    parser.add_argument('--rustc',default=shutil.which('rustc'))
    args=parser.parse_args();check_geometry(args.root)
    if args.pack:check_pack(args.root,args.source_root or args.root,args.pack,args.rustc)
