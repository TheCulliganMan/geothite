#!/usr/bin/env python3
"""Read-only quality audit of the actual R04 and frozen pond/bridge candidate."""
from copy import deepcopy
from collections import deque
import hashlib
import json
from pathlib import Path
import sys

from PIL import Image, ImageDraw

ROOT=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/"planning/minnesota/tools"))
from render_reference import SourceArt,layout_blocks,load_pack,room_sheet
from review_major_city import ReviewArt,render_map,player_camera
from render_archive import save_render

sha=lambda b:hashlib.sha256(b).hexdigest()
canonical=lambda v:json.dumps(v,sort_keys=True,separators=(",",":"),ensure_ascii=False).encode()


def audit(output):
    output.mkdir(parents=True,exist_ok=False)
    pack_path=ROOT/"content-packs/minnesota-house-joins-v04.crystalpack"
    assert sha(pack_path.read_bytes())=="63117daf68439663d7632293f8bded7352bf8e39b96d30edcc14f035871d4d6b"
    pack=load_pack(pack_path)
    frozen=ROOT/"planning/minnesota/connections/river-continuity-r04-v01/source-v04"
    pre=json.loads((frozen/"preimage.contract.json").read_bytes())
    contract=json.loads((frozen/"candidate.contract.json").read_bytes())
    layout=json.loads((frozen/"MN_R04.candidate.layout.json").read_bytes())
    module=pack["data"]["maps"]["MnR04C00R00"]
    assert sha(canonical(module))==pre["native_module_sha256"]
    previous=json.loads((frozen/"MN_R04.previous.layout.json").read_bytes())
    assert layout_blocks(previous)==module["blocks"]
    preview=deepcopy(pack)
    ts="mn_river_park"
    raw=bytes(pack["runtime_files"][f"data/tilesets/{ts}_metatiles.bin"])
    assert len(raw)==68*16
    new=bytearray(raw)
    for recipe in contract["native_art_recipes"]:
        assert len(new)==int(recipe["id"],16)*16
        for row in recipe["tile_row_sources"]:
            offset=int(row["source_block"],16)*16+row["source_row"]*4
            new.extend(raw[offset:offset+4])
        preview["data"]["tilesets"][ts]["collision"][recipe["id"]]=recipe["collision"]
    preview["runtime_files"][f"data/tilesets/{ts}_metatiles.bin"]=bytes(new)
    objects=[dict(id=o.get("object_identifier")or o["script"],sprite=o["sprite"],
                  tile=[o["x"],o["y"]],facing={"UP":"north","DOWN":"south","LEFT":"west","RIGHT":"east"}.get(o["spritemovedata"].rsplit("_",1)[-1],"south"),native_palette_override=o.get("pal",0))for o in module["objects"]]
    views=[("east-open-forecourt",[40,18]),("east-elm-bend",[46,20]),
           ("southwest-seating",[10,27]),("west-grass-choice",[15,11]),
           ("pond-bank-rail",[23,10]),("picnic-habitat-neck",[43,27])]
    frames=[];scenes={};grounds={};footings=[]
    for label,active,grid in [("actual-native",pack,module["blocks"]),("frozen-source-v04",preview,layout_blocks(layout))]:
        art=ReviewArt(active);local=dict(module,blocks=grid)
        ground,scene,_=render_map(art,local,objects,"day",output/label)
        scenes[label]=scene
        grounds[label]=ground
        for name,core in views:
            x,y=core;block=grid[y//2*28+x//2]
            token=active["data"]["tilesets"][ts]["collision"][f"{block:02x}"][y%2*2+x%2]
            assert token in {"FLOOR","TALL_GRASS","LONG_GRASS"},(name,core,token)
            assert core not in [o["tile"]for o in objects],(name,core)
            frame,_=player_camera(art,local,objects,ground,scene,"day",dict(core=core),output/label/"views"/name)
            frames.append((name+" / "+label,frame))
            footings.append(dict(view=name,state=label,core=core,collision=token))
    save_render(room_sheet(frames,3),output/"actual-versus-frozen-player-cameras.png")
    for name in ["NationalPark","Route34"]:
        m=pack["data"]["maps"][name];a=m["attributes"]
        save_render(SourceArt(pack).render(a["tileset_name"],a["width"],a["height"],m["blocks"]),output/f"{name}-authentic-whole-native.png")
    references=[];reference_positions=[]
    for name,label,core in [("NationalPark","stepped-circulation",[18,12]),
                            ("NationalPark","habitat-corner",[8,24]),
                            ("Route34","ordinary-lane-grass",[8,24])]:
        m=pack["data"]["maps"][name];a=m["attributes"];x,y=core
        lookup=pack["data"]["tilesets"][a["tileset_name"]]["collision"]
        def reference_kind(x,y):
            b=m["blocks"][y//2*a["width"]+x//2]
            return lookup[f"{b:02x}"][y%2*2+x%2]
        token=reference_kind(x,y)
        requested=list(core)
        if token not in {"FLOOR","TALL_GRASS","LONG_GRASS"}:
            valid=[(abs(xx-x)+abs(yy-y),xx,yy)for yy in range(a["height"]*2)for xx in range(a["width"]*2)
                   if reference_kind(xx,yy)in {"FLOOR","TALL_GRASS","LONG_GRASS"}]
            _,x,y=min(valid);core=[x,y];token=reference_kind(x,y)
        assert token in {"FLOOR","TALL_GRASS","LONG_GRASS"},(name,core,token)
        reference_positions.append(dict(map=name,label=label,requested_core=requested,actual_valid_core=core,collision=token,terrain_only_no_native_actor_comparison=True))
        reviewer=ReviewArt(pack)
        rg,rs,_=render_map(reviewer,m,[],"day",output/"authentic-reference-views"/label)
        frame,_=player_camera(reviewer,m,[],rg,rs,"day",dict(core=core),output/"authentic-reference-views"/label)
        references.append((name+" / "+label+" (terrain only)",frame))
    save_render(room_sheet(references,3),output/"authentic-player-scale-terrain-reference.png")
    annotations=scenes["frozen-source-v04"].convert("RGB");d=ImageDraw.Draw(annotations)
    issues=[("1 broad east apron",[18,8,27,11]),("2 repeated rose boxes",[19,5,26,8]),("3 repeated rectangular habitat",[22,12,24,15])]
    for title,(x0,y0,x1,y1)in issues:
        d.rectangle((x0*32,y0*32,x1*32-1,y1*32-1),outline="#ff245b",width=2)
        d.text((x0*32+4,y0*32+4),title,fill="#ff245b")
    d.rectangle((3*32,12*32,7*32-1,15*32-1),outline="#36dba4",width=2)
    d.text((3*32+4,12*32+4),"Quiet rest: retain",fill="#36dba4")
    save_render(annotations,output/"frozen-candidate-quality-annotations.png")
    grid=layout_blocks(layout);c=preview["data"]["tilesets"][ts]["collision"]
    kinds=lambda x,y:c[f"{grid[y//2*28+x//2]:02x}"][y%2*2+x%2]
    scans=[]
    for y in [17,18,19,20]:
        runs=[];run=[]
        for x in range(36,54):
            if kinds(x,y)=="FLOOR":run.append(x)
            elif run:runs.append(run);run=[]
        if run:runs.append(run)
        scans.append(dict(core_y=y,floor_runs_x=[[r[0],r[-1]]for r in runs]))
    # Detached east-apron proposal. All blocks are existing ordinary canopy,
    # and the existing picnic bench/cache/habitat are left completely alone.
    proposed=list(grid)
    patches=[dict(metatile=list(at),before="02",after="00",purpose=purpose)
             for at,purpose in [((19,8),"north shoulder below eastern flower display"),
                                ((20,8),"north shoulder narrows broad ordinary apron"),
                                ((21,8),"join shoulder to existing elm tongue"),
                                ((24,8),"shade north shoulder of east exit"),
                                ((25,8),"continue north shoulder beside existing east sign"),
                                ((24,10),"close unused lower shoulder after bend")]]
    owned={tuple(p["at_meta"])for p in contract["changes"]}
    for patch in patches:
        x,y=patch["metatile"];assert proposed[y*28+x]==int(patch["before"],16)
        assert (x,y)not in owned
        proposed[y*28+x]=int(patch["after"],16)
    def token(g,x,y):return c[f"{g[y//2*28+x//2]:02x}"][y%2*2+x%2]
    assert c["00"]==["WALL"]*4,c["00"]
    occupied={tuple(o["tile"])for o in objects}
    walk={"FLOOR","TALL_GRASS","LONG_GRASS"}
    def search(g,start,weighted=False):
        todo=deque([start]);cost={start:0}
        while todo:
            x,y=todo.popleft()
            for q in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]:
                if not(0<=q[0]<56 and 0<=q[1]<36)or q in occupied:continue
                kind=token(g,*q)
                if kind not in walk:continue
                extra=int(kind!="FLOOR")if weighted else 1
                value=cost[(x,y)]+extra
                if value<cost.get(q,10**9):
                    cost[q]=value
                    (todo.append if extra else todo.appendleft)(q)
        return cost
    costs=[search(g,(42,27),True).get((48,27))for g in [grid,proposed]]
    assert costs==[4,4],costs
    before_reach=search(grid,(0,18));after_reach=search(proposed,(0,18))
    assert (55,18)in before_reach and(55,18)in after_reach
    interactions=[]
    for obj in objects:
        x,y=obj["tile"]
        assert token(grid,x,y)==token(proposed,x,y),obj
        b=[q for q in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]if q in before_reach]
        a=[q for q in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]if q in after_reach]
        assert not b or a,obj
        interactions.append(dict(id=obj["id"],core=[x,y],before_reachable_adjacent=len(b),after_reachable_adjacent=len(a)))
    assert proposed[12*28+24]==grid[12*28+24]==0x0e
    assert token(proposed,48,27)=="FLOOR" and token(proposed,49,27)=="FLOOR"
    assert all(grid[y*28+x]==proposed[y*28+x]for y in range(18)for x in range(18)),"west route changed"
    assert all(grid[y*28+x]==proposed[y*28+x]for y in range(12,18)for x in range(28)),"southern habitat changed"
    signs=[]
    authored=json.loads((frozen/"MN_R04.candidate.json").read_bytes())
    for sign in authored["signs"]:
        x,y=sign["tile"]
        b=[q for q in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]if q in before_reach]
        a=[q for q in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]if q in after_reach]
        assert b and a,sign
        signs.append(dict(id=sign["id"],core=[x,y],before_reachable_adjacent=len(b),after_reachable_adjacent=len(a)))
    def vertical_run(g,x,y):
        lo=hi=y
        while lo>0 and token(g,x,lo-1)=="FLOOR":lo-=1
        while hi<35 and token(g,x,hi+1)=="FLOOR":hi+=1
        return dict(y_range_inclusive=[lo,hi],walking_tiles=hi-lo+1)
    widths=[dict(core=[x,18],before=vertical_run(grid,x,18),proposal=vertical_run(proposed,x,18))for x in [40,42,50]]
    proposal=dict(module,blocks=proposed);review=ReviewArt(preview)
    pg,ps,_=render_map(review,proposal,objects,"day",output/"proposal-only")
    save_render(room_sheet([("frozen pond/bridge v04",scenes["frozen-source-v04"]),("proposal: elm shoulders",ps)],2),output/"proposal-only"/"whole-before-proposal.png")
    pairs=[]
    for name,core in [("east-open-forecourt",[40,18]),("east-elm-bend",[46,20]),("east-return-lane",[50,18]),("west-choice-unchanged",[15,11]),("picnic-cache-unchanged",[48,27])]:
        assert token(proposed,*core)in walk and tuple(core)not in occupied
        pf,_=player_camera(review,proposal,objects,pg,ps,"day",dict(core=core),output/"proposal-only"/"views"/name)
        original,_=player_camera(review,dict(module,blocks=grid),objects,grounds["frozen-source-v04"],scenes["frozen-source-v04"],"day",dict(core=core),output/"proposal-only"/"before-views"/name)
        save_render(room_sheet([("frozen v04 / "+name,original),("proposal only / "+name,pf)],2),output/"proposal-only"/(name+"-before-proposal.png"))
        pairs.extend([(name+" / frozen v04",original),(name+" / proposal",pf)])
    save_render(room_sheet(pairs,2),output/"proposal-only"/"player-camera-before-proposal.png")
    proposal_record=dict(status="BOUNDED_PROPOSAL_ONLY_NOT_APPLIED_OR_ACCEPTED",source_candidate_contract_sha256=sha((frozen/"candidate.contract.json").read_bytes()),patches=patches,changed_metatiles=len(patches),ordinary_floor_quadrants_removed=24,minimum_grass_to_existing_cache_approach_before=4,minimum_grass_to_existing_cache_approach_proposed=4,actor_footing_unchanged=True,actor_interaction_adjacencies=interactions,sign_interaction_adjacencies=signs,ordinary_walking_widths=widths,item_core=[49,27],item_approach_core=[48,27],existing_picnic_bench_metatile=[24,12],picnic_cache_and_entire_southern_habitat_exactly_unchanged=True,west_trainer_and_grass_route_exactly_unchanged=True,west_east_walk_before_steps=before_reach[(55,18)],west_east_walk_proposed_steps=after_reach[(55,18)],pond_bridge_seams_unchanged=True,new_canopy_collision=["WALL"]*4,incidental_cut_quadrants=0,source_or_pack_writes=0)
    (output/"proposal-only"/"proposal.json").write_text(json.dumps(proposal_record,indent=2)+"\n")
    receipt=dict(status="READ_ONLY_SOURCE_QUALITY_AUDIT_NOT_REDESIGN_OR_NATIVE_PUBLICATION",actual_pack=str(pack_path),actual_pack_sha256=sha(pack_path.read_bytes()),actual_r04_module_sha256=sha(canonical(module)),frozen_preimage_matches_actual_pack=True,source_v04_not_installed=True,source_v04_candidate_contract_sha256=sha((frozen/"candidate.contract.json").read_bytes()),candidate_delta_metatiles=sum(a!=b for a,b in zip(module["blocks"],grid)),cameras=footings,east_floor_scans=scans,read_only=True,pack_files_written=0,canonical_or_frozen_files_changed=0,notes=["Source candidate uses the actual current native R04 preimage and exact frozen four-metatile append, in memory only.","Actual native and source-v04 are distinct states; no implementation or gameplay claim.","Views use authored native actors and valid walking/grass footing; no actor pose/time progression claim."])
    receipt["proposal_only"]=proposal_record
    receipt["authentic_reference_positions"]=reference_positions
    (output/"receipt.json").write_text(json.dumps(receipt,indent=2)+"\n")
    print(json.dumps(receipt))


if __name__=="__main__":audit(Path(sys.argv[1]))
