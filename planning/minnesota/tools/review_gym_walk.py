#!/usr/bin/env python3
"""Diagram an actual controller walk over the published room's native pixels.

The external proof is written only after the literal control regression wins
the badge, checks Continue and returns through both city-bound carpets.
"""
import argparse
import json
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

from render_archive import retain_render_file, save_render
from render_stages import render_stage


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--proof', type=Path, required=True)
    parser.add_argument('--room', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--font', type=Path, required=True)
    args = parser.parse_args()
    proof = json.loads(args.proof.read_text())
    assert proof['map'] == 'MnRochesterGym'
    assert proof['badge_earned'] and proof['both_city_returns'] and proof['tm_quantity'] == 1
    retain_render_file(args.room)
    room = render_stage(Image.open(args.room), 'convert', args.output.parent,
                        'recorded-room-rgb', 'RGB')
    assert room.size == (224, 320)
    scale, left, top = 2, 24, 82
    result = Image.new('RGB', (780, 784), '#17242b')
    draw = ImageDraw.Draw(result)
    title = ImageFont.truetype(str(args.font), 27)
    body = ImageFont.truetype(str(args.font), 17)
    small = ImageFont.truetype(str(args.font), 15)
    draw.text((24, 18), 'ROCHESTER / THE RECORDED SOLUTION', fill='white', font=title)
    draw.text((24, 51), 'Native room with the actual walked route overlaid', fill='#c3ced3', font=small)
    enlarged = render_stage(room, 'resize', args.output.parent, 'recorded-room-2x',
                            (448, 640), Image.Resampling.NEAREST)
    result.paste(enlarged, (left, top))
    colors = ['#37e9dd', '#ffd04d', '#8c59ef']
    for phase, color in zip(proof['phases'][:3], colors):
        points = [(left + (x * 16 + 8) * scale,
                   top + (y * 16 + 8) * scale) for x, y in phase['points']]
        for a, b in zip(phase['points'], phase['points'][1:]):
            assert abs(a[0]-b[0]) + abs(a[1]-b[1]) == 1, 'one actual tile per step'
        draw.line(points, fill='#17242b', width=8)
        draw.line(points, fill=color, width=4)
        # Small arrows show travel direction without changing the map pixels.
        for i in range(1, len(points), 3):
            x, y = points[i]
            px, py = points[i-1]
            dx, dy = (x-px)//32, (y-py)//32
            draw.polygon([(x+dx*5, y+dy*5), (x-dx*4-dy*4, y-dy*4+dx*4),
                          (x-dx*4+dy*4, y-dy*4-dx*4)], fill=color)
    for number, phase, color in zip(range(1, 4), proof['phases'], colors):
        x, y = phase['points'][-1]
        x, y = left+(x*16+8)*scale, top+(y*16+8)*scale
        draw.ellipse((x-12, y-12, x+12, y+12), fill='#17242b', outline=color, width=3)
        draw.text((x-5, y-10), str(number), fill='white', font=body)
    text_x = 494
    for y, color, heading, lines in [
        (100, colors[0], '1 / TAVEL', ['From the carpet,', 'go up to the first trainer.']),
        (216, colors[1], '2 / ELVIK', ['Go around the right end,', 'then the left end.', 'Battle the second trainer.']),
        (360, colors[2], '3 / VAREL', ['Use the top right gap.', 'Turn left into the court.', 'Both trainer wins required.']),
    ]:
        draw.text((text_x, y), heading, fill=color, font=body)
        for offset, line in enumerate(lines):
            draw.text((text_x, y+34+offset*25), line, fill='white', font=small)
    for offset, line in enumerate(['Played with Totodile 16', 'and Hoothoot 14.', '',
                                   'Parcel supplied at setup.', 'No badge or trainer wins', 'supplied at setup.', '',
                                   'Badge, one TM, Continue', 'and both exits verified.']):
        draw.text((text_x, 500+offset*24), line, fill='#c3ced3', font=small)
    draw.text((24, 746), 'Three alternating planter openings: right, left, right.', fill='white', font=body)
    print(save_render(result, args.output))


if __name__ == '__main__':
    main()
