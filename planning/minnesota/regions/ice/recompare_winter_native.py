#!/usr/bin/env python3
"""Recompare retained original GPU frames without launching or changing binaries."""
import argparse
import hashlib
import json
from pathlib import Path
import sys
from PIL import Image
ROOT=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
sys.path.insert(0,str(Path(__file__).parent))
from render_archive import save_render
from render_reference import SourceArt,load_pack,room_sheet
import review_winter_native as winter


def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--input',type=Path,required=True);p.add_argument('--output',type=Path,required=True);args=p.parse_args()
    prior_path=args.input/'capture-progress.json';prior_raw=prior_path.read_bytes();prior=json.loads(prior_raw)
    pack_path=Path(prior['pack']);assert sha(pack_path)==prior['pack_sha256'];pack=load_pack(pack_path);art=SourceArt(pack);winter.configure()
    args.output.mkdir(parents=True,exist_ok=False)
    report=dict(pack=str(pack_path),pack_sha256=prior['pack_sha256'],binary_sha256=prior['binary_sha256'],views_sha256=prior['views_sha256'],
        mode='RECOMPARE_ORIGINAL_GPU_IMAGES_WITHOUT_RECAPTURE',prior_capture_report=str(prior_path),prior_capture_report_sha256=hashlib.sha256(prior_raw).hexdigest(),
        comparator_sha256=sha(Path(winter.__file__)),unchanged_shared_reviewer_sha256=sha(winter.SHARED),
        source_rule='Native two-hop active/depth2/depth1 rectangle precedence; active-map modulo border only outside all real rectangles; camera clamp from immediate neighbors only.',
        source_algorithm_file='crates/crystal-bevy/src/bevy_shell/battle_entry_progress.rs',source_algorithm_sha256=sha(ROOT/'crates/crystal-bevy/src/bevy_shell/battle_entry_progress.rs'),fixture=prior['fixture'],captures=[])
    sheet=[]
    for old in prior['captures']:
        png=args.input/old['id']/'production.png';assert sha(png)==old['production_sha256'];folder=args.output/old['id'];folder.mkdir()
        actual=Image.open(png).convert('RGB');assert actual.size==(640,576)
        save_render(Image.open(png),folder/'production-original.png')
        view={k:old[k]for k in ['id','map','core']}
        result=winter.compare(pack,art,view,actual,folder)
        raw=[]
        for rel in old['raw_gpu_descriptors']:
            path=args.input/rel;raw.append(dict(path=str(path),sha256=sha(path)))
        row=dict(view,production_original=str(png),production_sha256=old['production_sha256'],original_command=old['command'],raw_gpu_descriptors=raw,
            prior_pixel_mismatches=old['pixel_mismatches'],**result)
        report['captures'].append(row);sheet.append((old['id'],actual))
        print(old['id'],'old',old['pixel_mismatches'],'new',result['pixel_mismatches'],flush=True)
        assert sha(png)==old['production_sha256']
    report['status']='PASS'if all(r['pixel_mismatches']==0 for r in report['captures'])else'REVIEW_DIFFERENCES'
    report['pack_unchanged']=sha(pack_path)==prior['pack_sha256'];report['prior_report_unchanged']=prior_path.read_bytes()==prior_raw
    assert report['pack_unchanged']and report['prior_report_unchanged']
    save_render(room_sheet(sheet,3),args.output/'production-decisions.png')
    (args.output/'review_winter_native.py.snapshot').write_bytes(Path(winter.__file__).read_bytes())
    (args.output/'review_connected_route_native.py.snapshot').write_bytes(winter.SHARED.read_bytes())
    (args.output/'recomparison.json').write_text(json.dumps(report,indent=2)+'\n')
    print(report['status'],len(report['captures']),'original frames',flush=True)
    assert report['status']=='PASS','Inspect retained full-frame differences'


if __name__=='__main__':main()
