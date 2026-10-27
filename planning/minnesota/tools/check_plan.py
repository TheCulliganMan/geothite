#!/usr/bin/env python3
"""Validate the authored Minnesota plan. This is not a pack/gameplay verifier."""
import argparse
import html
import json
from collections import deque
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]


def read(path):return json.loads(path.read_text())


def check():
    world=read(ROOT/'world.json');story=read(ROOT/'story.json')
    routes=[read(ROOT/r['file']) for r in world['routes']]
    towns={t['name']:t for t in world['towns']}
    route_ids={r['id'] for r in routes}
    by_id={r['id']:r for r in routes}
    assert {'BADGE_3','BADGE_4'}<=set(by_id['MN_R05']['required_flags'])
    assert 'EVENT_MN_SURVEY_RESTORED' in by_id['MN_R13']['required_flags']
    assert {f'BADGE_{i}' for i in range(1,9)}<=set(by_id['MN_R20']['required_flags'])
    assert len(route_ids)==20
    chunks={c['id']:c for c in world['chunks']}
    assert len(chunks)==len(world['chunks'])==sum(r['chunk_columns']*r['chunk_rows'] for r in routes)
    assert len(towns)==19
    for entry in world.get('regional_destinations',[]):
        assert entry['kind']=='planned_fairgrounds',('unknown destination kind',entry['kind'])
        destination=read(ROOT/entry['file'])
        assert destination['id']==entry['id'] and destination['parent_hub']==entry['parent_hub']
        assert entry['parent_hub'] in {t['id'] for t in towns.values()}
        assert destination['access']['from_hub']==entry['parent_hub']
        assert destination['districts'] and destination['activity_contracts']
        # A scope reservation does not create a native map or fake walking edge.
        assert destination['map_name'] not in chunks
    for entry in world.get('transit_networks',[]):
        assert entry['kind']=='planned_rail_service',('unknown transit kind',entry['kind'])
        network=read(ROOT/entry['file'])
        assert network['id']==entry['id']
        stops=network['ordered_stops']
        assert len(stops)>=2 and len({s['station'] for s in stops})==len(stops)
        assert len({s['visited_flag'] for s in stops})==len(stops)
        assert all(s['hub'] is None or s['hub'] in {t['id'] for t in towns.values()} for s in stops)
        assert all(s['visited_flag'].startswith('EVENT_MN_STATION_') for s in stops)
        # Rail reservations are not native maps or edges in symbolic progression.
        # Only registered authored exteriors/door bindings may change reachability.
        assert not {s['station'] for s in stops}&set(chunks)
    for special in world.get('special_zones',[]):
        zone=read(ROOT/special['file'])
        assert zone['id']==special['id'] and zone['parent_hub']==special['hub']
        assert special['hub'] in {t['id'] for t in towns.values()}
        assert zone['story_contract']['paid_expedition_required_for_story'] is False
        assert zone['expedition']['initial_balls']>0 and zone['expedition']['initial_steps']>0
        assert zone['capture_encounters']['commands']==['SAFARI_BALL','BAIT','ROCK','RUN']
        assert not zone['capture_encounters']['player_attacks']
        for habitat in zone['candidate_habitats']:
            assert sum(s['weight'] for s in habitat['weighted_encounters'])==100,habitat['id']
        # Candidate preserve chunks are deliberately excluded until source-art
        # fitting, native entrances and reciprocal connections are registered.
        assert not {c['id'] for c in zone['candidate_chunks']}&set(chunks)
    maps={t['id'] for t in towns.values()}|set(chunks)
    parent={}
    room_contracts={c['interior']:c for c in read(ROOT/'interiors.json')['contracts']}
    expansion_links=set()
    for t in towns.values():
        for interior in t['interiors']:
            assert interior['id'] not in maps
            maps.add(interior['id']);parent[interior['id']]=t['id']
        if 'authoring_file' not in t:
            continue
        authored=read(ROOT/t['authoring_file'])
        assert authored['id']==t['id'] and authored['dimensions_metatiles']==t['dimensions_metatiles'],t['id']+': stale town extent'
        assert authored['ports']==t['ports'],t['id']+': stale town ports'
        doors={d['target']:d for d in authored['doors']}
        assert len(doors)==len(authored['doors']),t['id']+': ambiguous facade pairing'
        roster={r['id']:r for r in t['interiors']}
        assert set(doors)<=set(roster),t['id']+': unregistered authored room'
        expansion_rooms=set()
        if t.get('expansion_contract_file'):
            from check_city_expansion_plan import validate as validate_expansion
            expansion_rooms,links=validate_expansion(ROOT,t,authored,room_contracts,world['connections'])
            expansion_links.update(links)
        for interior in t['interiors']:
            if 'authoring_file' not in interior:
                continue
            source=read(ROOT/interior['authoring_file'])
            room=next((r for r in source['interiors'] if r['id']==interior['id']),None) if 'interiors' in source else source
            assert room and room['id']==interior['id'] and room['dimensions_metatiles']==interior['dimensions_metatiles'],interior['id']+': stale room extent/source'
            contract=room_contracts[interior['id']]
            if interior['id'] in expansion_rooms:
                # The strict expansion adapter checks every indexed incoming
                # and return pair, including upper floors with no street door.
                continue
            if interior.get('access_kind')=='side_gatehouse':
                gate=authored['gatehouse_contract']
                assert room['template']==gate['reference']==contract['reference']=='IlexForestAzaleaGate'
                assert contract['parent']==t['id'] and contract['source_file']==interior['authoring_file']
                assert contract['passive_arrivals']==gate['passive_arrivals']
                assert contract['park']==gate['park'] and contract['route']==gate['route']
                assert [e['tile'] for e in room['exits']]==contract['return_triggers_core']
                assert [e['index'] for e in room['exits']]==[1,2,3,4]
                assert [a['index'] for a in gate['passive_arrivals']]==[5,6,7,8]
                assert room['passive_arrival']=={'index':5,'tile':[1,4]}
                continue
            if interior.get('access_kind')=='internal_stairs':
                stairs=next(s for s in authored['internal_stairs'] if s['to_room']==interior['id'])
                assert contract['parent']==stairs['from_room'] and contract['source_file']==interior['authoring_file']
                assert contract['arrival_core']==stairs['to_tile'] and contract['arrival_warp_index']==stairs['to_warp']
                assert contract['return_trigger_core']==stairs['to_tile'] and contract['return_triggers_core']==[stairs['to_tile']]
                assert contract['return_room']==stairs['from_room'] and contract['return_room_warp']==stairs['from_warp']
                assert room['passive_arrival']['tile']==authored['opening']['bedroom_start']
                continue
            door=doors[interior['id']]
            assert contract['parent']==t['id'] and contract['source_file']==interior['authoring_file']
            for field,key in [('exterior_door_core','tile'),('exterior_safe_return_core','return_tile'),('exterior_warp_index','warp_index'),('exterior_return_warp_index','return_warp_index'),('arrival_warp_index','target_warp')]:
                assert contract[field]==door[key],interior['id']+': stale '+field
            assert contract['arrival_core']==room.get('clear_arrival',room.get('passive_arrival',{}).get('tile',room.get('public_arrival_core'))),interior['id']+': stale arrival'
            exits=room.get('exit_warps',room.get('exits',room.get('exit_tiles_core')))
            for index,exit in enumerate(exits,1):
                if isinstance(exit,dict):
                    assert exit['index']==index and exit['target']==t['id'] and exit['target_warp']==door['return_warp_index'],interior['id']+': unsafe carpet destination/index'
            exits=[e['tile'] if isinstance(e,dict) else e for e in exits]
            assert contract['return_triggers_core']==exits and contract['return_trigger_core']==exits[0],interior['id']+': stale carpet exits'
    flags=set(story['flags'])
    conditions=flags|{f'BADGE_{i}' for i in range(1,9)}
    for r in routes:
        w,h=r['dimensions_metatiles']
        assert next(entry for entry in world['routes'] if entry['id']==r['id'])['dimensions_metatiles']==[w,h],r['id']+': stale world extent'
        if r['id'] in {'MN_R01','MN_R02','MN_R03','MN_R04','MN_R05','MN_R06','MN_R07','MN_R08','MN_R09','MN_R10','MN_R15'}:
            assert [w,h]=={'MN_R01':[32,20],'MN_R02':[30,22],'MN_R03':[24,32],'MN_R04':[28,18],'MN_R05':[32,56],'MN_R06':[32,44],'MN_R07':[32,48],'MN_R08':[40,48],'MN_R09':[64,36],'MN_R10':[48,20],'MN_R15':[32,18]}[r['id']]
        else:
            assert w>0 and h>0
        assert r['chunk_columns']==(w+31)//32 and r['chunk_rows']==(h+23)//24
        assert r['chunk_columns']*r['chunk_rows']==sum(c['route']==r['id'] for c in chunks.values())
        for c in chunks.values():
            if c['route']==r['id']:
                assert c['dimensions_metatiles']==[min(32,w-c['column']*32),min(24,h-c['row']*24)],c['id']
        if r['id']=='MN_R09':
            assert len(r['districts'])==5 and not r['loops']
            assert r['cut_gates'] and r['one_way_ledges'] and r['battle_or_grass_choices']
        elif r['id']=='MN_R10':
            assert len(r['districts'])==3 and len(r['loops'])==1 and r['required_grass_crossings']
        elif r['id'] in {'MN_R01','MN_R02','MN_R03','MN_R04','MN_R05','MN_R06','MN_R07','MN_R08','MN_R15'}:
            assert len(r['districts'])==3 and len(r['loops'])==(2 if r['id'] in {'MN_R05','MN_R06','MN_R08'} else 1)
        else:
            # Draft composition need not preserve the rejected six-district,
            # three-loop template. Physical route checks live in its reviewer.
            assert r['districts'] and len({d['id'] for d in r['districts']})==len(r['districts'])
        assert len(r['encounters']['grass']['day'])==7
        assert len(r['encounters']['water'])==3
        assert set(r['required_flags'])<=conditions,(r['id'],'unknown gate flag')
        for p in r['main_path_metatiles']:
            assert 0<=p[0]<w and 0<=p[1]<h,(r['id'],p)
        for d in r['districts']:
            x0,y0,x1,y1=d['bounds_metatiles']
            assert 0<=x0<x1<=w and 0<=y0<y1<=h
        for endpoint in r['endpoints']:
            assert endpoint['place'] in towns
            ports=[p for p in towns[endpoint['place']]['ports'] if p['route']==r['id']]
            assert len(ports)==1
            assert ports[0]['side']=={'N':'S','S':'N','E':'W','W':'E'}[endpoint['side']]
        for interior in r['interiors']:
            assert interior['id'] not in maps
            maps.add(interior['id']);parent[interior['id']]=r['id']
        for actor in r['trainers']:
            assert actor['defeated_flag'] in flags
            assert actor['party']
        for item in r['items']:assert item['flag'] in flags
        layout=read(ROOT/'routes'/f'{r["id"]}.layout.json')
        assert layout['dimensions_metatiles']==[w,h]
        for layer in layout['layers']:
            if 'bounds' in layer:
                x0,y0,x1,y1=layer['bounds'];assert 0<=x0<x1<=w and 0<=y0<y1<=h,(r['id'],layer)
            for x,y in layer.get('points',[]):assert 0<=x<w and 0<=y<h,(r['id'],layer)
            if 'position' in layer:
                x,y=layer['position'];assert 0<=x<w and 0<=y<h
    for c in world['connections']:
        assert c['a'] in maps and c['b'] in maps,c
        if c['kind']=='connection':
            if (c['a'],c['b']) not in expansion_links:
                assert c['offset_a']==c['offset_b']==0
                assert c['direction_b']=={'east':'west','south':'north'}[c['direction_a']]
        elif c['kind'] in {'gatehouse_warp','guarded_connection'}:
            assert c['route'] in route_ids and set(c['requires_flags'])<=conditions
        else:
            raise AssertionError(('unknown connection kind',c['kind']))
    from check_boundary_walking_links import validate as validate_boundary_walking_links
    walking_link_errors=validate_boundary_walking_links(world,ROOT)
    assert not walking_link_errors,walking_link_errors
    for q in story['quests']:
        assert set(q['requires'])<=conditions,(q['id'],q['requires'])
        assert all(m in maps or m in route_ids for m in q['maps']),(q['id'],q['maps'])
    # Symbolic progression, deliberately distinct from script execution.
    known=set();reachable={towns['WINONA']['id']};used=set()
    def at(location):
        while location in parent:location=parent[location]
        return location in reachable
    changed=True
    while changed:
        before=(len(known),len(reachable),len(used))
        for r in routes:
            a,b=[towns[e['place']]['id'] for e in r['endpoints']]
            if set(r['required_flags'])<=known and (a in reachable or b in reachable):
                reachable.update([a,b,r['id']])
        for gym in story['gyms']:
            if towns[gym['town']]['id'] in reachable and set(gym['requires'])<=known:
                known.update(gym['on_win'])
        for milestone in story['milestones']:
            assert set(milestone['requires'])<=conditions,milestone
            assert set(milestone['sets'])<=flags,milestone
            if at(milestone['location']) and set(milestone['requires'])<=known:
                known.update(milestone['sets']);used.add(milestone['id'])
        changed=before!=(len(known),len(reachable),len(used))
    assert 'EVENT_MN_LEAGUE_CHAMPION' in known,('story deadlock',sorted(known))
    assert {t['id'] for t in towns.values()}<=reachable,('unreachable towns',set(towns)-reachable)
    assert len(used)==len(story['milestones']),('unreachable milestone',used)
    print(f'Plan valid: {len(routes)} routes, {len(towns)} hubs, {len(chunks)} chunks, {len(world["connections"])} adjacency/gate contracts, {len(maps)} planned maps.')
    print('Symbolic story progression reaches all eight badges, all hubs and the champion. This does not execute gameplay.')
    return world,story,routes


def gallery(routes):
    out=ROOT/'renders';out.mkdir(exist_ok=True)
    cards=[]
    current={'MN_R01':'driftless','MN_R02':'prairie-farm','MN_R03':'river-valley','MN_R04':'twin-cities','MN_R05':'river-bluffs','MN_R06':'pine-country','MN_R07':'leech-lake','MN_R08':'northern-lakes','MN_R09':'iron-range','MN_R10':'iron-range-descent','MN_R16':'red-river-prairie','MN_R17':'border-pines','MN_R18':'mille-lacs-shore','MN_R19':'prairie-forest-link','MN_R20':'voyageurs-border-loop','MN_R11':'north-shore','MN_R12':'gunflint','MN_R13':'league-approach','MN_R14':'headwaters','MN_R15':'central-lakes'}
    for r in routes:
        rid=r['id'];a,b=[e['place'] for e in r['endpoints']]
        image=current[rid]+'/whole-route.png' if rid in current else f'routes/{rid}.png'
        label='Current authored terrain; consult the route runbook and native-comparison atlas for verification status.'
        cards.append(f'<article><h2>{rid} {html.escape(r["name"])}</h2><p>{a} → {b} · {r["dimensions_metatiles"][0]} × {r["dimensions_metatiles"][1]} metatiles</p><p>{label}</p><a href="{image}"><img loading="lazy" src="{image}" alt="{rid} current complete terrain"></a><p><a href="../routes/{rid}.json">Route dossier</a> · <a href="../routes/{rid}.layout.json">Authored terrain strokes</a></p></article>')
    text='''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>Minnesota source art planning gallery</title><style>body{margin:0;background:#131d28;color:#e5eef1;font:16px system-ui;line-height:1.5}main{max-width:1600px;margin:32px auto;padding:0 24px}a{color:#9cd5ef}h1{font-size:32px}p{max-width:1050px}.grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(430px,1fr));gap:28px}article{padding:20px;background:#1d2b39;border:1px solid #3b5063;border-radius:8px}img{width:100%;max-height:720px;object-fit:contain;background:#0c141c;image-rendering:pixelated}h2{font-size:21px}nav{display:flex;gap:20px;flex-wrap:wrap;margin:24px 0}.notice{border-left:4px solid #d4b269;padding:12px 20px;background:#2a2b27}</style><main><h1>Minnesota source art planning gallery</h1><p>Original packed tiles at their real proportions. Current source revisions, historical packs and verified native slices are distinguished in the comparison atlas and route runbooks. Map size varies with worthwhile features and traversal. Local publication does not establish the connected regional journey or final visual acceptance.</p><div class="notice">The original Johto and Kanto render controls style. Earlier illustrated route plates are rejected as implementation authority. A picture or plan graph does not prove collision, story scripts or gameplay.</div><nav><a href="../README.md">Implementation handoff</a><a href="source/Route29.png">Native Route 29</a><a href="source/Route42.png">Native Route 42</a><a href="source/Route45.png">Native Route 45</a><a href="source/johto-atlas.png">Source metatiles</a><a href="../reference/johto-kanto-original.png">Original full atlas</a></nav><div class="grid">'''+''.join(cards)+'</div></main></html>'
    (out/'index.html').write_text(text)


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--pack',type=Path);parser.add_argument('--render',action='store_true');args=parser.parse_args()
    _,_,routes=check()
    if args.pack:
        from render_reference import load_pack,render_layouts
        pack=load_pack(args.pack);species=pack['data']['pokemon'];items=pack['data']['items']
        from check_city_expansion_plan import validate as validate_expansion
        current_world=read(ROOT/'world.json')
        registry={row['interior']:row for row in read(ROOT/'interiors.json')['contracts']}
        for town in current_world['towns']:
            if town.get('expansion_contract_file'):
                validate_expansion(ROOT,town,read(ROOT/town['authoring_file']),registry,current_world['connections'],pack)
        for special in read(ROOT/'world.json').get('special_zones',[]):
            zone=read(ROOT/special['file'])
            for habitat in zone['candidate_habitats']:
                for slot in habitat['weighted_encounters']:
                    assert slot['species'] in species and 1<=slot['level']<=100,slot
        for r in routes:
            for timed in r['encounters']['grass'].values():
                for slot in timed:assert slot['species'] in species,slot
            for slot in r['encounters']['water']:assert slot['species'] in species,slot
            for actor in r['trainers']:
                for mon in actor['party']:assert mon['species'] in species,mon
            for item in r['items']:assert item['item'] in items,item
        story=read(ROOT/'story.json')
        for team in story['gyms']+story['bosses']+story['league']:
            for mon in team['party']:assert mon['species'] in species,mon
        from render_reference import layout_blocks
        first=routes[0];layout=read(ROOT/'routes/MN_R01.layout.json')
        blocks=layout_blocks(layout);w,h=first['dimensions_metatiles']
        collision=pack['data']['tilesets']['johto']['collision']
        def token(x,y):
            block=blocks[(y//2)*w+x//2]
            return collision[f'{block:02x}'][(y%2)*2+x%2]
        start=first['endpoints'][0]['landing_metatile']
        origin=(start[0]*2+1,start[1]*2+1);queue=deque([origin]);reached={origin}
        while queue:
            x,y=queue.popleft()
            for nx,ny in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]:
                if 0<=nx<w*2 and 0<=ny<h*2 and (nx,ny) not in reached and token(nx,ny) in ['FLOOR','TALL_GRASS']:
                    reached.add((nx,ny));queue.append((nx,ny))
        for record in first['endpoints']:
            x,y=record['landing_metatile'];assert (x*2+1,y*2+1) in reached,record
        for record in first['trainers']+first['items']:
            x,y=record['anchor_metatile'];assert (x*2+1,y*2+1) in reached,record
        for door in first['doors']:
            x,y=door['tile'];assert token(x,y)==door['collision'],door['id']
            assert tuple(door['return_tile']) in reached,door['id']+': return'
        print('All route, gym, boss and League species and route pickup items resolve in the supplied pack.')
        print(f"Route 1 source collision connects both arrivals, {len(first['trainers'])} trainer anchors and both doors. Runtime acceptance is documented in its native runbook.")
        if args.render:render_layouts(pack,ROOT)
    elif args.render:parser.error('--render requires --pack')
    gallery(routes)


if __name__=='__main__':main()
