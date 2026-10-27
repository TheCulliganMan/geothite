"""Inspect authored native ledges; draw movement evidence over source pixels.

This is a read-only review overlay. It never places terrain or alters collision.
"""
from collections import deque

from PIL import ImageDraw


def shortest_walk(start, end, allowed):
    parents = {start: None}
    todo = deque([start])
    while todo:
        x, y = todo.popleft()
        if (x, y) == end:
            path = []
            at = end
            while at is not None:
                path.append(at)
                at = parents[at]
            return path[::-1]
        for point in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]:
            if point in allowed and point not in parents:
                parents[point] = (x, y)
                todo.append(point)
    return None


def inspect_ledges(design, kind, reachable):
    floor = {p for p in reachable if kind(*p) in {'FLOOR', 'TALL_GRASS'}}
    width, height = design['dimensions_metatiles']
    reports = []
    for ledge in design.get('one_way_ledges', []):
        name = ledge['id']
        start, end = tuple(ledge['landing']), tuple(ledge['upper_approach'])
        assert start in floor and end in floor, name + ': occupied/blocked landing or approach'
        path = shortest_walk(start, end, floor)
        minimum = ledge['minimum_return_steps']
        assert minimum >= 9 and path and len(path) - 1 >= minimum, name + ': disconnected or trivial return'
        x0, y0, x1, y1 = ledge['return_passage_core']
        assert 0 <= x0 < x1 <= width * 2 and 0 <= y0 < y1 <= height * 2, name + ': return passage bounds'
        passage = {(x, y) for y in range(y0, y1) for x in range(x0, x1)}
        assert start not in passage and end not in passage, name + ': passage overlaps ledge'
        return_floor = floor
        if ledge.get('return_region_core') is not None:
            left, top, right, bottom = ledge['return_region_core']
            assert 0 <= left < right <= width*2 and 0 <= top < bottom <= height*2, name + ': local region bounds'
            inside = lambda point: left <= point[0] < right and top <= point[1] < bottom
            assert inside(start) and inside(end) and all(map(inside, passage)), name + ': local region excludes endpoint/passage'
            return_floor = {point for point in floor if inside(point)}
            assert shortest_walk(start, end, return_floor), name + ': disconnected local ordinary return'
        assert shortest_walk(start, end, return_floor - passage) is None, name + ': unintended uphill opening'
        reports.append(dict(id=name, return_walk_steps=len(path) - 1,
                            hop_displacement_tiles=3, saved_walk_steps=len(path) - 4,
                            return_path_core=path,
                            local_return_walk_steps=len(shortest_walk(start, end, return_floor))-1))
    return reports


def draw_ledge_paths(scene, design, reports):
    image = scene.convert('RGB').copy()
    draw = ImageDraw.Draw(image)
    center = lambda p: (p[0] * 16 + 8, p[1] * 16 + 8)
    for ledge, report in zip(design.get('one_way_ledges', []), reports):
        points = [center(p) for p in report['return_path_core']]
        draw.line(points, fill='#052e34', width=7)
        draw.line(points, fill='#50e9ea', width=3)
        start, end = center(ledge['upper_approach']), center(ledge['landing'])
        draw.line([start, end], fill='#3a2105', width=9)
        draw.line([start, end], fill='#ffbd42', width=5)
        dx = (end[0] > start[0]) - (end[0] < start[0])
        dy = (end[1] > start[1]) - (end[1] < start[1])
        draw.polygon([end, (end[0] - dx * 11 - dy * 6, end[1] - dy * 11 + dx * 6),
                      (end[0] - dx * 11 + dy * 6, end[1] - dy * 11 - dx * 6)], fill='#ffbd42')
        x, y = center(ledge['reverse_blocked'])
        draw.line([(x - 4, y - 4), (x + 4, y + 4)], fill='#ff5768', width=3)
        draw.line([(x - 4, y + 4), (x + 4, y - 4)], fill='#ff5768', width=3)
        label = f"{report['id']}: {report['return_walk_steps']}-tile walk back"
        x, y = max(4, min(start[0] - 90, image.width - 300)), max(4, start[1] - 30)
        draw.rectangle((x - 3, y - 3, x + 294, y + 15), fill='#14212b')
        draw.text((x, y), label, fill='white')
    return image
