#!/usr/bin/env python3
"""Validate original special-room exports; requires no game content.
Usage: python tools/check-special-rooms.py [model-directory]
"""
from pathlib import Path
import runpy,sys
from model_asset_storage import read_model_json
ROOT=Path(__file__).resolve().parents[1]
EXPECTED={'bicycle_side','bicycle_front','bicycle_diagonal','bike_service_rack','mobile_battle_terminal','mobile_trade_terminal','link_battle_console','link_trade_console','link_room_receiver','shrine_dragon_rail','shrine_dragon_mask','shrine_paper_lantern','tower_emblem_panel','tower_round_fixture','elevator_controls','prize_counter','roof_access_hut','roof_planter','gym_stone_partition','facility_instrument_bank','facility_instrument_pair','roof_binoculars','link_round_stool'}
if __name__=='__main__':
    validate=runpy.run_path(str(ROOT/'tools/check-dungeon-models.py'),run_name='geometry_checker')['validate']
    directory=Path(sys.argv[1])if len(sys.argv)>1 else ROOT/'crates/crystal-voxel-view/models/special_rooms'
    paths=sorted(directory.glob('*.mesh.json'))
    assert {p.name.removesuffix('.mesh.json')for p in paths}==EXPECTED
    counts={p.name:validate(p)for p in paths}
    source_sampled={'roof_binoculars':16,'facility_instrument_pair':16,'facility_instrument_bank':32,'link_round_stool':16}
    for name,height in source_sampled.items():
        document=read_model_json(directory/(name+'.mesh.json'))
        assert document['triangle_count']<1000, (name,'small source fixtures must stay compact')
        for primitive in document['primitives']:
            anchor=primitive.get('source_pixel')
            assert isinstance(anchor,list) and len(anchor)==2 and all(type(v)is int for v in anchor), (name,'missing live material anchor')
            assert 0<=anchor[0]<16 and 0<=anchor[1]<height, (name,'material anchor leaves its verified source drawing')
    for name,count in counts.items():print(f'{name}: {count} triangles')
    print(f'PASS: {len(counts)} original special-room meshes, {sum(counts.values())} triangles')
