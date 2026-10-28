#!/usr/bin/env python3
"""Read-only native Surf-edge audit; local collision witnesses, not earned play.

Native bridge axes are respected. Minneapolis gate states are compared using
only the pack's existing gate-open block changes on an in-memory copy. Ordinary
Foot witnesses deliberately omit ledges, Cut and hidden-object state changes.
"""
import argparse
import collections
from copy import deepcopy
import hashlib
import json
from pathlib import Path

from audit_outdoor_connections import OPPOSITE, boundary, collision, project
from render_reference import load_pack

SURF = {'WATER', 'WATERFALL', 'BRIDGE_NS_DECK', 'BRIDGE_NS_RAIL',
        'BRIDGE_EW_DECK', 'BRIDGE_EW_RAIL'}
FOOT = {'FLOOR', 'TALL_GRASS', 'DOOR', 'BRIDGE_NS_DECK', 'BRIDGE_EW_DECK'}
DIRECTIONS = {'north': (0, -1), 'south': (0, 1), 'west': (-1, 0), 'east': (1, 0)}


def permits(kind, dx, dy):
    return kind in SURF and not (kind.startswith('BRIDGE_NS') and dx
                                or kind.startswith('BRIDGE_EW') and dy
                                or kind == 'WATERFALL' and dx)


def inspect(pack, gate_open):
    maps = {n: m for n, m in pack['data']['maps'].items() if n.startswith('Mn')
            and m['attributes']['environment'].lower() in {'town', 'route', 'outdoor'}}
    maps = dict(maps)
    if gate_open:
        module = maps['MnMinneapolis'] = deepcopy(maps['MnMinneapolis'])
        changes = [r for r in module['script_block_changes']
                   if r['source_script'] == 'MnMPUpperPoolBuoyOpen']
        assert len(changes) == 3
        for r in changes:
            module['blocks'][(r['y']//2)*module['attributes']['width']+r['x']//2] = r['block_id']
    kinds = {n: collision(pack, m) for n, m in maps.items()}
    occupied = {n: {(o['x'], o['y']) for o in m['objects']} for n, m in maps.items()}
    connections = {(n, e['direction']): e for n, m in maps.items()
                   for e in m['attributes']['connections'] if e['target_map'] in maps}

    def step(n, p, direction):
        dx, dy = DIRECTIONS[direction]
        q = p[0]+dx, p[1]+dy
        if q in kinds[n]:
            return n, q
        e = connections.get((n, direction))
        if e is None:
            return None
        target = e['target_map']
        q = project(maps[n]['attributes'], maps[target]['attributes'], e, p)
        inverse = connections.get((target, OPPOSITE[direction]))
        if (q not in kinds[target] or inverse is None or inverse['target_map'] != n
                or project(maps[target]['attributes'], maps[n]['attributes'], inverse, q) != p):
            return None
        return target, q

    pending = {(n, p) for n in maps for p, k in kinds[n].items()
               if k in SURF and p not in occupied[n]}
    component_of, components = {}, []
    while pending:
        start = min(pending)
        pending.remove(start)
        queue, cells, banks = collections.deque([start]), [], []
        while queue:
            n, p = queue.popleft()
            cells.append((n, p))
            component_of[n, p] = len(components)
            for direction, (dx, dy) in DIRECTIONS.items():
                target = step(n, p, direction)
                if target is None:
                    continue
                t, q = target
                if (t, q) in pending and permits(kinds[n][p], dx, dy) and permits(kinds[t][q], dx, dy):
                    pending.remove((t, q))
                    queue.append((t, q))
                # Surf mounting must target ordinary Water from an ordinary bank.
                if (n == t and kinds[n][p] == 'WATER' and kinds[t][q] in {'FLOOR', 'TALL_GRASS'}
                        and q not in occupied[t]):
                    banks.append({'map': n, 'floor': list(q), 'water': list(p)})
        components.append({'index': len(components), 'core_cells': len(cells),
                           'maps': sorted({n for n, p in cells}), 'bank_count': len(banks),
                           'bank_examples': banks[:4], '_banks': banks})
    river_component = component_of['MnWinona', (26, 11)]
    exposed, native_shores, real_seams = [], [], 0
    for n, module in maps.items():
        for direction, (dx, dy) in DIRECTIONS.items():
            for p in boundary(module['attributes'], direction):
                if (n, p) not in component_of or not permits(kinds[n][p], dx, dy):
                    continue
                target = step(n, p, direction)
                if target is not None:
                    t, q = target
                    if permits(kinds[t][q], dx, dy):
                        real_seams += 1
                        continue
                    native_shores.append({'map': n, 'core': list(p), 'direction': direction,
                                          'neighbor': t, 'neighbor_core': list(q),
                                          'neighbor_collision': kinds[t][q]})
                    continue
                e = connections.get((n, direction))
                ci = component_of[n, p]
                exposed.append({'map': n, 'core': list(p), 'direction': direction,
                                'component': ci, 'bank_count': components[ci]['bank_count'],
                                'same_component_as_proven_river': ci == river_component,
                                'declared_neighbor': e['target_map'] if e else None})
    # Local witness: actual room door or ordinary native perimeter -> bank. This
    # is not a claim that the room/map is currently progression-accessible.
    relevant = {r['component'] for r in exposed}
    for ci in relevant:
        c = components[ci]
        c['foot_launch_witness'] = None
        for bank in c['_banks']:
            n = bank['map']
            seeds = {(w['x'], w['y']) for w in maps[n]['events']['warps']}
            seeds |= {p for d in DIRECTIONS if (n, d) in connections
                      for p in boundary(maps[n]['attributes'], d)}
            seeds = {p for p in seeds if kinds[n].get(p) in FOOT and p not in occupied[n]}
            parents = {p: None for p in seeds}
            queue = collections.deque(sorted(seeds))
            endpoint = tuple(bank['floor'])
            while queue and endpoint not in parents:
                x, y = queue.popleft()
                for dx, dy in DIRECTIONS.values():
                    p = x+dx, y+dy
                    if p not in parents and kinds[n].get(p) in FOOT and p not in occupied[n]:
                        parents[p] = (x, y)
                        queue.append(p)
            if endpoint in parents:
                path, p = [], endpoint
                while p is not None:
                    path.append(list(p))
                    p = parents[p]
                path.reverse()
                c['foot_launch_witness'] = dict(bank, foot_path=path,
                    seed_kind='authored exterior room warp or native road perimeter; local geometry only')
                break
    for c in components:
        del c['_banks']
    return {'gate_state': 'native_open_changes_geometry_only' if gate_open else 'packed_closed',
            'exterior_count': len(maps), 'river_component': river_component,
            'river_maps': components[river_component]['maps'], 'real_directed_surf_seam_lanes': real_seams,
            'unmatched_edge_records': exposed, 'native_neighbor_shores': native_shores,
            'unmatched_components': [components[i] for i in sorted(relevant)]}


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--pack', type=Path, required=True)
    ap.add_argument('--output', type=Path, required=True)
    args = ap.parse_args()
    raw = args.pack.read_bytes()
    digest = hashlib.sha256(raw).hexdigest()
    pack = load_pack(args.pack)
    report = {'status': 'STATIC_NATIVE_COLLISION_AUDIT_NOT_CONTROLLER_PLAY',
              'pack': str(args.pack), 'pack_sha256': digest,
              'assumptions': ['Surf and Waterfall prerequisites supplied for geometry classification.',
                  'Every current native object treated as occupying its authored tile; hidden/moving actors not simulated.',
                  'Bridge E0..E3 directions respected on entry and exit; no Foot-to-Surf bridge launch.',
                  'Waterfall geometry traversed vertically with HM assumed; no earned progression claim.',
                  'Foot witnesses use only ordinary Floor/TallGrass/door and native deck; ledges/Cut omitted.',
                  'No unconnected edge is alleged traversable off-map; missing visible closure is the issue.'],
              'states': [inspect(pack, False), inspect(pack, True)]}
    assert hashlib.sha256(args.pack.read_bytes()).hexdigest() == digest
    args.output.mkdir(parents=True, exist_ok=True)
    (args.output/'audit.json').write_text(json.dumps(report, indent=2)+'\n')
    for state in report['states']:
        counts = collections.Counter((r['map'], r['direction']) for r in state['unmatched_edge_records'])
        print(state['gate_state'], len(state['unmatched_edge_records']), sorted(counts.items()))
        for c in state['unmatched_components']:
            print(c['index'], c['core_cells'], c['bank_count'], c['foot_launch_witness'])


if __name__ == '__main__':
    main()
