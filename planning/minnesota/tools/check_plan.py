#!/usr/bin/env python3
"""Validate the authored Minnesota plan. This is not a pack/gameplay verifier."""
import argparse
import html
import json
from collections import deque
from pathlib import Path

from check_safari_plan import safari_tables

ROOT=Path(__file__).resolve().parents[1]


def read(path):return json.loads(path.read_text())


def check():
    world=read(ROOT/'world.json');story=read(ROOT/'story.json')
    routes=[read(ROOT/r['file']) for r in world['routes']]
    towns={t['name']:t for t in world['towns']}
    endpoint_hubs={name:town['id'] for name,town in towns.items()}
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
        assert entry['kind'] in {'planned_fairgrounds','authored_fairgrounds'},('unknown destination kind',entry['kind'])
        destination=read(ROOT/entry['file'])
        assert destination['id']==entry['id'] and destination['parent_hub']==entry['parent_hub']
        assert entry['parent_hub'] in {t['id'] for t in towns.values()}
        access=destination['access']
        assert access['from_hub'] in set(chunks)|{t['id'] for t in towns.values()}
        assert destination['districts'] and destination['activity_contracts']
        # A scope reservation does not create a native map or fake walking edge.
        assert destination['map_name'] not in chunks
        if entry['kind']=='authored_fairgrounds':
            native=destination['native_map']
            assert native['id']==destination['map_name']
            layout=read(ROOT/entry['layout_file'])
            assert layout['id']==native['id'] and layout['dimensions_metatiles']==native['dimensions_metatiles']
            seam=entry['native_connection']
            assert seam['a']==access['from_hub'] and seam['b']==native['id']
            road=chunks[seam['a']]
            assert (seam['direction_a'],seam['direction_b'],access['city_side'])==('west','east','W')
            assert len(native['ports'])==1
            port=native['ports'][0]
            assert (port['route'],port['side'])==(road['route'],'E')
            gates=access['city_gate_anchor_core']
            assert gates==seam['road_core']==[[0,2*access['city_gate_metatile'][1]+dy] for dy in (0,1)]
            assert 0<=gates[0][1]<gates[1][1]<2*road['dimensions_metatiles'][1]
            assert seam['offset_a']==-seam['offset_b']
            width,height=native['dimensions_metatiles']
            assert seam['fair_core']==[[2*width-1,y-2*seam['offset_a']] for _,y in gates]
            assert [y for _,y in seam['fair_core']]==[2*port['gate_metatile'][1]+dy for dy in (0,1)]
            assert all(0<=y<2*height for _,y in seam['fair_core'])
            assert access['city_landing_metatile']==[seam['bounded_trail_metatiles'],gates[0][1]//2]
            registry={c['interior']:c for c in read(ROOT/'interiors.json')['contracts']}
            for room in native['interiors']:
                door=next(d for d in native['doors'] if d['target']==room['id'])
                contract=registry[room['id']]
                assert contract['parent']==native['id'] and contract['source_room_id']==room['id']
                assert contract['arrival_core']==room['passive_arrival']['tile'] and contract['arrival_warp_index']==room['passive_arrival']['index']
                assert contract['return_triggers_core']==[e['tile'] for e in room['exits']]
                assert contract['exterior_door_core']==door['tile'] and contract['exterior_safe_return_core']==door['return_tile']
                assert contract['exterior_warp_index']==door['warp_index'] and contract['exterior_return_warp_index']==door['return_warp_index']
    for entry in world.get('transit_networks',[]):
        assert entry['kind'] in {'planned_rail_service','native_passenger_service'},('unknown transit kind',entry['kind'])
        network=read(ROOT/entry['file'])
        assert network['id']==entry['id']
        stops=network['ordered_stops']
        assert len(stops)>=2 and len({s['station'] for s in stops})==len(stops)
        assert len({s['visited_flag'] for s in stops})==len(stops)
        assert all(s['hub'] is None or s['hub'] in {t['id'] for t in towns.values()} for s in stops)
        assert all(s['visited_flag'].startswith('EVENT_MN_STATION_') for s in stops)
        if entry['kind']=='native_passenger_service':
            service=network['current_service']
            stations=read(ROOT/service['source_file'])['stations']
            assert [(s['station'],s['visited_flag']) for s in stops]==[(s['map'],s['visited_flag']) for s in stations]
            assert service['native_directed_branches']==len(stations)*(len(stations)-1)
            assert service['first_visit_callbacks']==len(stations)
        # Visiting a station permits travel there; rail does not unlock a new hub.
        assert not {s['station'] for s in stops}&set(chunks)
    for special in world.get('special_zones',[]):
        zone=read(ROOT/special['file'])
        assert zone['id']==special['id'] and zone['parent_hub']==special['hub']
        assert special['hub'] in {t['id'] for t in towns.values()}
        assert zone['story_contract']['paid_expedition_required_for_story'] is False
        assert zone['expedition']['initial_balls']>0 and zone['expedition']['initial_steps']>0
        assert zone['capture_encounters']['commands']==['SAFARI_BALL','BAIT','ROCK','RUN']
        assert not zone['capture_encounters']['player_attacks']
        safari_tables(ROOT/special['file'],zone,ROOT.parents[1]/world['region_ecology_revision']['source_file'])
        # Candidate preserve chunks are deliberately excluded until source-art
        # fitting, native entrances and reciprocal connections are registered.
        assert not {c['id'] for c in zone['candidate_chunks']}&set(chunks)
    maps={t['id'] for t in towns.values()}|set(chunks)
    parent={}
    room_contracts={c['interior']:c for c in read(ROOT/'interiors.json')['contracts']}
    revisions=read(ROOT/'gyms/current.json')['overrides']
    revision_sources={r['map']:'gyms/'+r['source'] for r in revisions}
    assert len(revision_sources)==len(revisions),'duplicate current gym revision'
    seen_revisions=set()
    expansion_links=set()
    for t in towns.values():
        for interior in t['interiors']:
            assert interior['id'] not in maps
            maps.add(interior['id']);parent[interior['id']]=t['id']
            if interior['id'] in revision_sources:
                path=revision_sources[interior['id']]
                # The original regional index owns the legacy street bindings.
                # Each town also names a subsequent hand-authored gym layout.
                base=read(ROOT/path)
                source=read(ROOT/interior['authoring_file'])
                assert all(s['schema_version']==1 and len(s['gyms'])==1 for s in (base,source))
                previous=base['gyms'][0];gym=source['gyms'][0];contract=room_contracts[interior['id']]
                assert gym['id']==previous['id']==interior['id'] and gym['slot']==previous['slot']
                if interior['kind']=='canonical_gym_reference':
                    assert interior['source_reference']['gym_id']==gym['id']
                    assert interior['native_constant']==gym['constant']
                else:
                    assert interior['kind']==f"gym_{gym['slot']}"
                assert gym['dimensions_metatiles']==interior['dimensions_metatiles']
                assert contract['revision_source_file']==path and contract['parent']==t['id']
                assert contract['arrival_core']==previous['arrival_core'] and contract['arrival_warp_index']==3
                assert contract['return_triggers_core']==previous['exit_tiles_core']
                assert contract['return_trigger_core']==previous['exit_tiles_core'][0]
                width,height=gym['dimensions_metatiles']
                assert len(gym['rows'])==height and all(len(row)==width for row in gym['rows'])
                assert len(gym['exit_tiles_core'])==2
                assert all(0<=x<2*width and 0<=y<2*height for x,y in [gym['arrival_core'],*gym['exit_tiles_core']])
                seen_revisions.add(interior['id'])
        if 'authoring_file' not in t:
            continue
        authored=read(ROOT/t['authoring_file'])
        assert authored['id']==t['id'] and authored['dimensions_metatiles']==t['dimensions_metatiles'],t['id']+': stale town extent'
        if t['id']=='MnDetroitLakes':
            ports = [{k:v for k,v in entry.items() if k!='native_via'} for entry in authored['ports']]
            gate = next(link for link in world['connections'] if link['b']==t['id'] and link['route']=='MN_R19')
            ports.append(dict(route=gate['route'],side='N',gate_metatile=gate['b_gate_metatile'],landing_metatile=gate['b_arrival_metatile']))
            assert ports==t['ports']
        else:
            assert authored['ports']==t['ports'],t['id']+': stale town ports'
        doors={d['target']:d for d in authored['doors']}
        assert len(doors)==len(authored['doors']),t['id']+': ambiguous facade pairing'
        overlays=authored.get('transit_overlays',[])
        assert overlays==t.get('transit_overlays',[]),t['id']+': stale transit overlay registration'
        for overlay in overlays:
            source=read(ROOT/overlay['source_file'])
            room=overlay['room_id'];binding=source['exterior_binding'];door=binding['door']
            assert source['id']==room and binding['hub']==t['id']
            assert binding['expected_dimensions_metatiles']==authored['dimensions_metatiles']
            assert room not in doors and door['target']==room,t['id']+': duplicate overlay door'
            assert room_contracts[room]['source_file']==overlay['source_file']
            assert next(r for r in t['interiors'] if r['id']==room)['authoring_file']==overlay['source_file']
            doors[room]=door
        roster={r['id']:r for r in t['interiors']}
        assert set(doors)<=set(roster),t['id']+': unregistered authored room'
        expansion_rooms=set()
        if t.get('recomposition_contract_file'):
            from check_city_binding_plan import current_city as validate_recomposition
            expansion_rooms,links=validate_recomposition(ROOT,t,authored,room_contracts,world['connections'])
            expansion_links.update(links)
        elif t.get('expansion_contract_file'):
            from check_city_expansion_plan import validate as validate_expansion
            expansion_rooms,links=validate_expansion(ROOT,t,authored,room_contracts,world['connections'])
            expansion_links.update(links)
        for interior in t['interiors']:
            if 'authoring_file' not in interior or interior['id'] in revision_sources:
                continue
            source=read(ROOT/interior['authoring_file'])
            room=next((r for r in source['interiors'] if r['id']==interior['id']),None) if 'interiors' in source else source
            assert room and room['id']==interior['id'] and room['dimensions_metatiles']==interior['dimensions_metatiles'],interior['id']+': stale room extent/source'
            contract=room_contracts[interior['id']]
            if interior['id'] in expansion_rooms:
                # The strict expansion adapter checks every indexed incoming
                # and return pair, including upper floors with no street door.
                continue
            if interior.get('access_kind')=='published_road_gatehouse':
                from check_shore_gate_plan import validate as validate_gate
                validate_gate(ROOT,t,interior,room,contract)
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
            assert contract['parent']==t['id']
            if contract['source_file']!=interior['authoring_file']:
                binding=read(ROOT/contract['source_file'])
                prior=next(r for r in binding['interiors'] if r['id']==interior['id'])
                for key in ('dimensions_metatiles','clear_arrival','passive_arrival','exits'):
                    assert prior.get(key)==room.get(key),(interior['id'],key)
                assert next(d for d in binding['doors'] if d['target']==interior['id'])==door
            for field,key in [('exterior_door_core','tile'),('exterior_safe_return_core','return_tile'),('exterior_warp_index','warp_index'),('exterior_return_warp_index','return_warp_index'),('arrival_warp_index','target_warp')]:
                if field in ('exterior_door_core','exterior_safe_return_core') and contract[field]!=door[key]:
                    # These originals remain registered at their pre-revision coordinates.
                    from check_city_binding_plan import archived_facade
                    archived_facade(ROOT,t,door,contract)
                else:
                    assert contract[field]==door[key],interior['id']+': stale '+field
            assert contract['arrival_core']==room.get('clear_arrival',room.get('passive_arrival',{}).get('tile',room.get('public_arrival_core'))),interior['id']+': stale arrival'
            exits=room.get('exit_warps',room.get('exits',room.get('exit_tiles_core')))
            for index,exit in enumerate(exits,1):
                if isinstance(exit,dict):
                    assert exit['index']==index and exit['target']==t['id'] and exit['target_warp']==door['return_warp_index'],interior['id']+': unsafe carpet destination/index'
            exits=[e['tile'] if isinstance(e,dict) else e for e in exits]
            assert contract['return_triggers_core']==exits and contract['return_trigger_core']==exits[0],interior['id']+': stale carpet exits'
    assert seen_revisions==set(revision_sources),'unregistered current gym revision'
    intake = room_contracts['MnBoundaryIntake']
    assert intake['parent'] in maps and intake['paired_bindings'][0]['parent'] == intake['parent']
    assert read(ROOT / intake['source_file'])['id'] == intake['interior']
    maps.add(intake['interior'])
    parent[intake['interior']] = intake['parent']
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
                expected=[min(32,w-c['column']*32),min(24,h-c['row']*24)]
                if c['id']=='MnR11C01R00':
                    terminal=r['compact_terminal']
                    expected[0]=terminal['native_width_metatiles']
                    assert terminal['source_canvas_retired_bounds']==[c['source_origin_metatiles'][0]+expected[0],0,w,expected[1]]
                assert c['dimensions_metatiles']==expected,c['id']
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
            if r['id']=='MN_R12':
                assert d['id'] in {'LOONS_SHORES','RAVEN_FROST_BROW','STONE_BOG_RANGER'} and d['purpose']
                continue
            x0,y0,x1,y1=d['bounds_metatiles']
            assert 0<=x0<x1<=w and 0<=y0<y1<=h
        for endpoint in r['endpoints']:
            if endpoint['place']=='CEDAR APPROACH':
                cedar=read(ROOT/'towns/MnMaraisCedarApproach.json')
                assert r['id']=='MN_R12' and endpoint['side']=='E'
                assert cedar['id']==world['coastal_cedar_warmth_revision']['map']
                ports=r['proposed_ports']
                assert {'direction':'west','target_map':ports['east_native_map'],'offset':-ports['east_offset']} in cedar['native_connections']
                southern=next(c for c in cedar['native_connections'] if c['direction']=='south')
                assert southern['target_map']==towns['GRAND MARAIS']['id']
                maps.add(cedar['id'])
                endpoint_hubs[endpoint['place']]=southern['target_map']
                continue
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
        for item in r['items']:assert item['flag'] in flags or item['flag'] in world['planned_flags'],item['id']
        layout=read(ROOT/'routes'/f'{r["id"]}.layout.json')
        assert layout['dimensions_metatiles']==[w,h]
        for layer in layout['layers']:
            if 'bounds' in layer:
                x0,y0,x1,y1=layer['bounds'];assert 0<=x0<x1<=w and 0<=y0<y1<=h,(r['id'],layer)
            for x,y in layer.get('points',[]):assert 0<=x<w and 0<=y<h,(r['id'],layer)
            if 'position' in layer:
                x,y=layer['position'];assert 0<=x<w and 0<=y<h
    icefield=read(ROOT/'regions/ice/MnRainyRiverIcefields.json')
    assert icefield['ports'][0]['target'] in chunks and chunks[icefield['ports'][0]['target']]['route']=='MN_R20'
    maps.add(icefield['id'])
    for c in world['connections']:
        assert c['a'] in maps and c['b'] in maps,c
        if c['kind']=='connection':
            if 'MnMaraisCedarApproach' in (c['a'],c['b']):
                side='a' if c['a']=='MnMaraisCedarApproach' else 'b'
                other='b' if side=='a' else 'a'
                connection=dict(direction=c['direction_'+side],target_map=c[other],offset=c['offset_'+side])
                assert connection in cedar['native_connections'] and c['offset_'+side]==-c['offset_'+other]
                assert c['direction_'+other]=={'west':'east','south':'north'}[connection['direction']]
            elif (c['a'],c['b']) not in expansion_links:
                assert c['offset_a']==c['offset_b']==0
                assert c['direction_b']=={'east':'west','south':'north'}[c['direction_a']]
        elif c['kind'] in {'direct_connection','native_map_connection'}:
            assert c['route'] in route_ids and set(c['requires_flags'])<=conditions
            legacy=c['kind']=='native_map_connection'
            a,b=(c[k] for k in (('a_side','b_side') if legacy else ('direction_a','direction_b')))
            off_a,off_b=(c[k] for k in (('a_offset','b_offset') if legacy else ('offset_a','offset_b')))
            left,right=(c[k] for k in (('a_core_lanes','b_core_lanes') if legacy else ('a_edge_core','b_edge_core')))
            assert b=={'east':'west','west':'east','north':'south','south':'north'}[a] and off_a==-off_b
            axis=int(a in ('east','west'))
            assert len(left)==len(right)==2 and left[1][axis]==left[0][axis]+1
            assert all(x[axis]-2*off_a==y[axis] for x,y in zip(left,right))
        elif c['kind'] in {'gatehouse_warp','guarded_connection'}:
            assert c['route'] in route_ids and set(c['requires_flags'])<=conditions
        else:
            raise AssertionError(('unknown connection kind',c['kind']))
    from check_boundary_walking_links import validate as validate_boundary_walking_links
    walking_link_errors=validate_boundary_walking_links(world,ROOT)
    assert not walking_link_errors,walking_link_errors
    # The mine's later floors have their own registered dossiers and indexed return links.
    needed={m for q in story['quests'] for m in q['maps'] if m not in maps and m not in route_ids}
    for location in needed:
        trail=set()
        while location not in maps:
            assert location not in trail,('cyclic parent chain',location)
            trail.add(location)
            room=room_contracts[location]
            assert read(ROOT/room['source_file'])['id']==location
            assert any(pair['parent']==room['parent'] for pair in room['paired_bindings'])
            parent[location]=room['parent']
            location=room['parent']
        maps.update(trail)
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
            a,b=[endpoint_hubs[e['place']] for e in r['endpoints']]
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
            if town['id'] in ('MnBemidji','MnStCloud'):
                from check_city_binding_plan import check_realigned_fronts
                check_realigned_fronts(read(ROOT/town['authoring_file']),pack['data'])
            if town.get('recomposition_contract_file'):
                from check_city_binding_plan import current_city as validate_recomposition
                validate_recomposition(ROOT,town,read(ROOT/town['authoring_file']),registry,current_world['connections'],pack)
            elif town.get('expansion_contract_file'):
                validate_expansion(ROOT,town,read(ROOT/town['authoring_file']),registry,current_world['connections'],pack)
        print('The three current major city exteriors and all indexed expansion-room entries and returns resolve in the supplied pack.')
        from audit_region_encounters import active_roster
        active=active_roster()
        for special in current_world.get('special_zones',[]):
            zone=read(ROOT/special['file'])
            expected=safari_tables(ROOT/special['file'],zone,ROOT.parents[1]/current_world['region_ecology_revision']['source_file'])
            safari=pack['data']['safari_zone']
            assert safari['id']==zone['id']
            assert {h['id'] for h in safari['habitats']}==expected.keys()
            for habitat in safari['habitats']:
                for kind in ('grass','water'):
                    assert habitat[kind]==expected[habitat['id']][kind],(habitat['id'],kind)
                    assert all(slot['species'] in species and slot['species'] in active for slot in habitat[kind])
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
    if args.render: gallery(routes)


if __name__=='__main__':main()
