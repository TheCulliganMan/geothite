#!/usr/bin/env python3
"""Capture actual reached dark/lit saves; compare full illuminated native frames.

The controller course produces each checkpoint after actual Flash and clue input.
Lit cameras face south with the unclaimed cache visible. Source NITE is the
native DARK+Flash palette; no captured GPU image or pack is changed. Dark frames
are retained for visual inspection separately from seven full-frame comparisons.
"""
import argparse
from copy import deepcopy
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

from PIL import Image

ROOT=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_reference import load_pack,SourceArt,room_sheet
from render_archive import retain_render_file,save_render
from review_cave_quality_native import compare


def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    p=argparse.ArgumentParser(description=__doc__)
    for n in ['pack','binary','saves','output']:p.add_argument('--'+n,type=Path,required=True)
    args=p.parse_args();args.output.mkdir(parents=True,exist_ok=False)
    pack_path=args.pack.resolve();private=Path('.local')/f'gallery-capture-{os.getpid()}';shutil.copy2(args.binary,private)
    source_path=Path(__file__).parent/'gallery.json';source=json.loads(source_path.read_text());pack=load_pack(pack_path);art=SourceArt(pack)
    report=dict(pack=str(pack_path),pack_sha256=sha(pack_path),binary_sha256=sha(private),source_sha256=sha(source_path),scope=__doc__,captures=[]);sheet=[]
    env=dict(os.environ,CRYSTAL_RENDER_TEST_HEADLESS='1',CRYSTAL_RENDER_TEST_CAPTURE_TARGET='image',CRYSTAL_RENDER_TEST_WINDOW_SIZE='640x576',CRYSTAL_PREVIEW_MUTE='1')
    for key in ['CRYSTAL_RENDER_TEST_PARTY_SPECIES','CRYSTAL_RENDER_TEST_PARTY_SIZE','CRYSTAL_RENDER_TEST_SECOND_VIEW']:env.pop(key,None)
    views=[('dark-entry',[18,17]),('dark-fossil',[5,4])]+[('lit-'+v['id'],v['tile'])for v in source['review_contract']['viewpoints']]
    try:
        for name,at in views:
            folder=args.output/name;folder.mkdir();save=args.saves/(name+'.crystalsave');png=folder/'production.png';cmd=[str(private.resolve()),'--pack',str(pack_path),'--save',str(save.resolve()),'--view','2d','--screenshot',str(png.resolve())]
            frozen=sha(save)
            with(folder/'capture.log').open('w')as log:r=subprocess.run(cmd,env=env,stdout=log,stderr=subprocess.STDOUT,timeout=120)
            assert r.returncode==0,(name,r.returncode);assert sha(save)==frozen;retain_render_file(png);actual=Image.open(png).convert('RGB');row=dict(id=name,save=str(save),save_sha256=frozen,production_sha256=sha(png),command=cmd,core=at)
            if name.startswith('lit-'):
                module=deepcopy(pack['data']['maps']['MnDriftlessLampGallery']);module['attributes']['time_of_day']='nite';row.update(compare(pack,art,module,at,actual,folder))
            report['captures'].append(row);(args.output/'capture-progress.json').write_text(json.dumps(report,indent=2)+'\n');sheet.append((name,actual));print(name,row.get('pixel_mismatches','dark visual review'),flush=True)
        save_render(room_sheet(sheet,3),args.output/'actual-dark-and-lit-cameras.png');report['status']='PASS'if all(r.get('pixel_mismatches',0)==0 for r in report['captures'])else'REVIEW_DIFFERENCES'
    finally:
        private.unlink();report['private_binary_removed']=True;assert sha(pack_path)==report['pack_sha256'];(args.output/'capture-progress.json').write_text(json.dumps(report,indent=2)+'\n')
    assert report['status']=='PASS'


if __name__=='__main__':main()
