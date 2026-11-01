#!/usr/bin/env python3
"""Compose actual native maps, optionally labeling a verified controller fixture.

A passing journey log is accepted only with the exact pack-bound itinerary and
both named production tests passing. Evidence links/hashes remain beside the
image. Existing images are archived before any replacement.
"""
import argparse
import hashlib
import json
from pathlib import Path
from PIL import Image, ImageDraw
from render_progress_atlas import font
from render_archive import save_render


def journey_evidence(args, report):
    """Return exact fixture evidence or None; never infer earned progression."""
    if not args.journey_log and not args.journey_receipt:
        return None
    sha = lambda raw: hashlib.sha256(raw).hexdigest()
    if args.journey_receipt:
        evidence = json.loads(args.journey_receipt.read_bytes())
        assert evidence['status'] == 'NATIVE_CONTROLLER_PASS_EXPLICIT_LOCAL_FIXTURE'
        assert evidence['pack_sha256'] == report['pack_sha256']
        log_path = Path(evidence['log_path'])
        plan_path = Path(evidence['plan_path'])
        assert sha(log_path.read_bytes()) == evidence['log_sha256']
        assert sha(plan_path.read_bytes()) == evidence['plan_sha256']
        if args.journey_log: assert args.journey_log == log_path
    else:
        log_path = args.journey_log
        plan_path = args.journey_plan or args.audit_directory/'all-bridges-input-plan.json'
    plan = json.loads(plan_path.read_bytes())
    assert plan['pack_sha256'] == report['pack_sha256']
    assert len(plan['steps']) == 1269 and len(plan['bridge_apertures']) == 7
    log = log_path.read_text()
    tests = ['mississippi_bridge_keeps_native_foot_road_and_upper_save',
             'mississippi_continuous_surf_winona_to_st_cloud_and_bridge_continue']
    assert all(test+' ... ok' in log for test in tests)
    assert '2 passed; 0 failed; 0 ignored;' in log and 'FAILED' not in log
    return dict(schema_version=1,status='NATIVE_CONTROLLER_PASS_EXPLICIT_LOCAL_FIXTURE',
                pack_sha256=report['pack_sha256'],plan_path=str(plan_path),
                plan_sha256=sha(plan_path.read_bytes()),log_path=str(log_path),
                log_sha256=sha(log_path.read_bytes()),input_count=1269,
                bridge_count=7,passed_tests=tests,
                fixture='Declared local Surf/Waterfall party and badges; no earned regional progression claimed.')


ap=argparse.ArgumentParser(description=__doc__)
ap.add_argument('--audit-directory',type=Path,required=True)
ap.add_argument('--manifest',type=Path,required=True)
ap.add_argument('--journey-log',type=Path)
ap.add_argument('--journey-receipt',type=Path)
ap.add_argument('--journey-plan',type=Path)
args=ap.parse_args()
out=args.audit_directory
report=json.loads((out/'native-connection-audit.json').read_bytes())
manifest=json.loads(args.manifest.read_bytes())
evidence=journey_evidence(args,report)
names={row['id'] for row in manifest['new_maps']} | {'MnMinneapolis','MnStPaul','MnR04C00R00','MnR03C00R00','MnWinona','MnStCloud','MnR05C00R00','MnR05C00R01','MnR05C00R02'}
rows=[row for row in report['placements'] if row['map'] in names]
assert len(rows)==len(names)==26 and len({r['component'] for r in rows})==1
x0=min(r['origin_metatiles'][0] for r in rows);y0=min(r['origin_metatiles'][1] for r in rows)
x1=max(r['origin_metatiles'][0]+r['dimensions_metatiles'][0] for r in rows)
y1=max(r['origin_metatiles'][1]+r['dimensions_metatiles'][1] for r in rows)
scale=12
im=Image.new('RGB',((x1-x0)*scale,(y1-y0)*scale+104),'#121f2b');draw=ImageDraw.Draw(im)
draw.text((14,12),'WINONA → ST CLOUD · ACTUAL CONNECTED NATIVE MAPS',font=font(24,True),fill='white')
if evidence:
    caption=f"Pack {report['pack_sha256'][:12]} · PASS: 1,269 native route inputs · all 7 bridge apertures · explicit local fixture."
    limits='Declared Surf/Waterfall party + badges; image shows initial map gates. No earned regional progression claimed.'
else:
    caption=f"Published pack {report['pack_sha256'][:12]} · 26 native exteriors at one scale · no invented terrain. Controller journey pending."
    limits='Original badge gate shown closed; native Surf/Waterfall prerequisites and bridge traversal require production inputs.'
draw.text((14,49),caption,font=font(15),fill='#e8c38c')
draw.text((14,73),limits,font=font(15),fill='#b9d0cf')
for row in rows:
    x,y=row['origin_metatiles'];w,h=row['dimensions_metatiles']
    scene=Image.open(out/'maps'/row['map']/'scene-rgb.png')
    assert scene.size==(w*32,h*32)
    im.paste(scene.resize((w*scale,h*scale),Image.Resampling.NEAREST),((x-x0)*scale,(y-y0)*scale+104))
save_render(im,out/'winona-st-cloud-native-connected.png')
if evidence:
    (out/'native-journey-evidence.json').write_text(json.dumps(evidence,indent=2)+'\n')
print('Native corridor:',im.size,'controller fixture passed:',bool(evidence))
