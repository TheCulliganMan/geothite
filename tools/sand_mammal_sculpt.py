#!/usr/bin/env python3
"""Original plated armadillo and swept-quill digging mammal anatomy.

Geometry authoring only; canonical skin integration is a separate build step.
Coordinates are +Y up, +Z front, with complete planted neutral feet.
"""
import math
from sculpt_geometry import ellipsoid, crease_normals, fitted_triangle_contour, part
from bovine_sculpt import curved_tube, oval
from cubone_sculpt import skin_shading
from skin_glb import COORDINATES

SPECIES = ('sandshrew', 'sandslash')
HEIGHTS = {'sandshrew': 1.02, 'sandslash': 1.18}
GOLD = (.88, .69, .31)
OCHRE = (.77, .56, .26)
CREAM = (.98, .92, .74)
IVORY = (.98, .97, .90)
SEAM = (.53, .37, .20)
INK = (.075, .09, .10)


def fitted(name, color, outline, parent, thickness=.004):
    return skin_shading(crease_normals(fitted_triangle_contour(
        name, color, outline, parent, thickness, precision=7), 40.), parent)


def raw_parts(species):
    if species not in SPECIES:
        raise ValueError('unknown armored digging mammal')
    slash = species == 'sandslash'
    body = ellipsoid('Body / broad upright digging torso', GOLD,
                     (0., .43, 0.), (.265, .32, .225), 32, 20)
    head = ellipsoid('Head / tapered armored mammal face', GOLD,
                     (0., .775, .055), (.218, .218, .205), 32, 20)
    muzzle = ellipsoid('Muzzle / low cream digging snout', CREAM,
                       (0., .704, .216), (.138, .078, .112), 28, 18)
    p = [body, head, muzzle]
    p.append(fitted('Belly / fitted broad cream underside', CREAM,
                    oval(0., .415, .194, .251, 48), body, .008))
    p.append(fitted('Nose / small dark snout tip', INK,
                    oval(0., .732, .024, .015, 24), muzzle, .004))
    if not slash:
        for side in (-1, 1):
            x = side*.047
            p.append(fitted('Forehead plate / fitted polygonal armor '+str(side),
                            GOLD, [(x-.042,.901),(x+.042,.901),
                            (x+.036,.960),(x-.036,.960)], head, .006))
        p.append(fitted('Bridge plate / fitted central face armor', GOLD,
                        [(-.035,.883),(.035,.883),(.035,.791),(-.035,.791)], head, .005))
    for side in (-1, 1):
        ear = curved_tube('Ear / rooted tapered pinna ' + str(side), GOLD,
                         [(side*.135, .905, .015), (side*.177, 1.013, .012),
                          (side*.183, 1.080 if slash else 1.050, .018)],
                         [.063, .040, .004], 20)
        p.append(ear)
        p.append(fitted('Ear inset / fitted warm pinna ' + str(side), OCHRE,
                        oval(side*.169, .989, .018, .034, 24), ear, .003))
        xy = ([(side*x, y) for x, y in ((.075, .843), (.171, .875),
               (.163, .793), (.097, .779))] if slash else
              oval(side*.116, .826, .042, .058, 32))
        eye = fitted('Eye / fitted ivory lens ' + str(side), IVORY, xy, head)
        p.append(eye)
        p.append(fitted('Pupil / fitted dark vertical gaze ' + str(side), INK,
                        oval(side*.124, .823, .015, .026 if slash else .038, 28), eye, .003))
        p.append(curved_tube('Arm / continuous bent digging forelimb ' + str(side),
                            GOLD, [(side*.145, .55, .005),
                            (side*.31, .43, .055), (side*.35, .34, .15)],
                            [.085, .077, .064], 20))
        p.append(ellipsoid('Hand / broad digging palm ' + str(side), GOLD,
                           (side*.35, .31, .16), (.088, .086, .079), 24, 16))
        for j, dx in enumerate((-.035,.035) if slash else (-.052,0.,.052)):
            x = side*.35 + dx
            p.append(curved_tube(f'Claw / rooted curved ivory digger {side} {j}',
                                IVORY, [(x, .297, .183), (x, .251, .245),
                                (x, .145 if slash else .227, .300 if slash else .285)],
                                [.031 if slash else .026, .022 if slash else .020, .002], 16))
        p.append(ellipsoid('Haunch / rooted supporting hindquarter ' + str(side),
                           GOLD, (side*.18, .215, -.01), (.12, .165, .125), 28, 18))
        p.append(curved_tube('Leg / supported bent ankle ' + str(side), GOLD,
                            [(side*.18, .19, .01), (side*.21, .10, .04),
                             (side*.21, .055, .115)], [.080, .065, .055], 20))
        p.append(ellipsoid('Foot / complete planted mammal pad ' + str(side), GOLD,
                           (side*.21, .043, .105), (.105, .043, .14), 28, 18))
        for j, dx in enumerate((-.057, 0., .057)):
            p.append(ellipsoid(f'Toe / planted ivory foot claw {side} {j}', IVORY,
                               (side*.21+dx, .025, .213), (.027, .025, .075), 20, 14))
    p.append(curved_tube('Tail / continuous tapering armored tail', GOLD,
                        [(0., .245, -.12), (0., .16, -.32),
                         (.06, .14, -.49), (.14, .18, -.62)],
                        [.090, .075, .043, .003], 24))
    shell = ellipsoid('Shell / rooted dorsal armor mantle', SEAM if not slash else OCHRE,
                      (0., .48, -.100), (.266, .35, .205), 32, 20)
    p.append(shell)
    # Eighteen independently rooted tiles/quills; no detached floating spikes.
    for row, y in enumerate((.300, .475, .650)):
        for col in range(6):
            angle = (col-2.5)*.48
            fraction = max(.1, 1.-((y-.48)/.35)**2)**.5
            x = .266*fraction*math.sin(angle)
            z = -.100-.205*fraction*math.cos(angle)
            if slash:
                p.append(curved_tube(f'Quill / rooted swept dorsal lance {row} {col}',
                                    OCHRE, [(x*.88, y, z+.040),
                                    (x*1.12, y+.080, z-.070),
                                    (x*1.38, y+.195, z-.190)],
                                    [.071, .054, .002], 18))
            else:
                # Broad shallow rounded tiles leave narrow dark mortar seams.
                outline = [(a*fraction,b) for a,b in
                           ((-.043,-.087),(.043,-.087),(.057,-.071),(.057,.071),
                            (.043,.087),(-.043,.087),(-.057,.071),(-.057,-.071))]
                vertices = [(a*scale,b*scale,depth) for depth,scale in
                            ((-.032,.85),(-.020,1.),(.020,1.),(.032,.85))
                            for a,b in outline]
                faces = [tuple(reversed(range(8))),tuple(range(24,32))]
                for ring in range(3):
                    for i in range(8):
                        a = ring*8+i;b = ring*8+(i+1)%8
                        faces.append((a,b,b+8,a+8))
                tile = crease_normals(part(f'Plate / segmented dorsal armor tile {row} {col}',
                                           GOLD, vertices, faces, True), 35.)
                c, s = math.cos(angle), math.sin(angle)
                for field in ('positions', 'normals'):
                    values = []
                    for i in range(0, len(tile[field]), 3):
                        a, b, d = tile[field][i:i+3]
                        values.extend((c*a-s*d+(x if field=='positions' else 0.),
                                       b+(y if field=='positions' else 0.),
                                       s*a+c*d+(z if field=='positions' else 0.)))
                    tile[field] = values
                p.append(tile)
    return p


def sculpture(species, normalize=True):
    parts = raw_parts(species)
    if normalize:
        lo = min(v for p in parts for v in p['positions'][1::3])
        hi = max(v for p in parts for v in p['positions'][1::3])
        scale = HEIGHTS[species]/(hi-lo)
        for p in parts:
            p['positions'] = [round((v-(lo if i%3==1 else 0.))*scale, 7)
                              for i, v in enumerate(p['positions'])]
            p['normals'] = [round(v, 7) for v in p['normals']]
    return {'name': species, 'version': 1, 'coordinate_system': COORDINATES,
            'primitives': parts}
