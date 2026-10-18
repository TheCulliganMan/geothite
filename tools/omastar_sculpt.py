#!/usr/bin/env python3
"""Original spiked nautilus adult with a genuinely open four-part beak."""
import argparse
import json
import math
from pathlib import Path

from bovine_sculpt import oval
from cubone_sculpt import skin_shading
from lugia_sculpt import paddle
from omanyte_sculpt import shell_parts, CREAM, BLUE, INK
from raptor_bird_sculpt import paint
from sculpt_geometry import ellipsoid, tube, crease_normals, fitted_contour
from skin_glb import COORDINATES

HEIGHT = 1.08
YELLOW = (.99, .94, .59)
IVORY = (.99, .96, .77)


def anatomy():
    # Reuse the authored nautilus surface construction, with adult dimensions.
    # The mantle, gaze, beak, spines and ten tentacle paths are independently posed.
    parts = shell_parts()
    for part in parts:
        part['part'] = part['part'].replace('Omanyte', 'Omastar')
        part['positions'] = [v * 1.12 + (.0, .13, -.015)[i % 3]
                             for i, v in enumerate(part['positions'])]
    body = ellipsoid('Body / broad blue Omastar mantle', BLUE,
                     (0, .265, .165), (.29, .195, .235), 36, 22)
    parts.append(body)
    for side in (-1, 1):
        eye = ellipsoid(f'Eye / rooted pale yellow Omastar lens {side}', YELLOW,
                        (side * .115, .410, .348), (.067, .070, .041), 28, 18)
        parts.append(eye)
        pupil = skin_shading(paint(f'Pupil / fitted black vertical Omastar slit {side}',
                                  INK, oval(side * .115, .410, .012, .054, 18),
                                  eye, .004, 2), eye)
        parts.append(pupil)
        parts.append(skin_shading(paint(f'Glint / fitted ivory Omastar eye light {side}',
                                       IVORY, oval(side * .115-.002, .441, .003, .007, 12),
                                       pupil, .003, 1), pupil))

        paths = [
            ([(side*.20,.33,.15),(side*.31,.42,.19),(side*.34,.61,.19),(side*.32,.71,.21)],
             [.077,.068,.037,.002], [.054,.046,.025,.0015], 'raised inner'),
            ([(side*.23,.25,.03),(side*.43,.28,.04),(side*.48,.48,.03),(side*.44,.63,.04)],
             [.060,.051,.025,.002], [.045,.034,.019,.0015], 'raised outer'),
            ([(side*.19,.19,.12),(side*.37,.12,.24),(side*.50,.16,.29),(side*.53,.24,.28)],
             [.065,.048,.028,.002], [.044,.034,.020,.0015], 'curling side'),
            ([(side*.19,.22,-.01),(side*.34,.20,-.12),(side*.44,.26,-.09),(side*.43,.35,-.07)],
             [.054,.038,.022,.002], [.040,.031,.015,.0015], 'curling rear'),
            ([(side*.10,.21,.29),(side*.14,.085,.37),(side*.27,.047,.43),(side*.35,.095,.41)],
             [.078,.070,.042,.002], [.054,.046,.025,.0015], 'grounded front'),
        ]
        for points, widths, depths, label in paths:
            parts.append(paddle(f'Tentacle / rooted {label} Omastar arm {side}', BLUE,
                                points, widths, depths, segments=18, steps=4))

    # Four solid ivory quadrants surround an unoccupied cross. Their recessed
    # roots enter the mantle; neither a painted cross nor a filled cream disc
    # stands in for the open beak.
    parts.append(ellipsoid('Mouth / recessed dark Omastar oral well', INK,
                           (0, .257, .400), (.111, .106, .012), 28, 16))
    for sx in (-1, 1):
        for sy in (-1, 1):
            outline = [(.016, .016), (.100, .016)]
            outline += [(.101*math.cos(a), .101*math.sin(a))
                        for a in (math.pi*i/24 for i in range(2, 11))]
            outline += [(.016, .100)]
            xy = [(sx*x, .257+sy*y) for x, y in outline]
            def dome(y, x):
                return .375 + .24*math.sqrt(max(0., .115**2-x*x-(y-.257)**2))
            parts.append(crease_normals(fitted_contour(
                f'Beak / rooted ivory Omastar quadrant {sx} {sy}',
                IVORY, xy, dome, .060, 2), 40))

    # Six real tapering spines follow the shell's dorsal meridian. Roots sit
    # inside the stored shell and extend along its outward radial direction.
    for index, angle in enumerate((.08, .70, 1.33, 1.97, 2.60, 3.21)):
        cy = .49*1.12 + .13
        cz = -.045*1.12 - .015
        root = (0, cy+.32*1.12*.94*math.sin(angle),
                cz+.275*1.12*.94*math.cos(angle))
        tip = (.018 if index % 2 else -.018,
               root[1]+.14*math.sin(angle), root[2]+.14*math.cos(angle))
        parts.append(crease_normals(tube(f'Spine / rooted cream Omastar shell thorn {index}',
                                         CREAM, [root, tip], [.055, .0015], 16), 40))
    return parts


def sculpture(normalize=True):
    parts = anatomy()
    low = min(v for part in parts for v in part['positions'][1::3])
    high = max(v for part in parts for v in part['positions'][1::3])
    scale = HEIGHT/(high-low)
    for part in parts:
        if normalize:
            part['positions'] = [round((v-(low if i % 3 == 1 else 0))*scale, 7)
                                 for i, v in enumerate(part['positions'])]
        part['normals'] = [round(v, 7) for v in part['normals']]
        for key in ('positions', 'normals'):
            part[key] = [0. if v == 0 else v for v in part[key]]
    return {'name': 'omastar', 'version': 1, 'coordinate_system': COORDINATES,
            'primitives': parts}


def write_canonical(output):
    from nautilus_glb import export_nautilus
    (output/'omastar.glb').write_bytes(export_nautilus('omastar'))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    write_canonical(args.output)


if __name__ == '__main__':
    main()
