#!/usr/bin/env python3
"""Capture actual reached saves with no coordinate, time or story overrides."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

from PIL import Image, ImageDraw

ROOT=Path(__file__).resolve().parents[4]
PREFIXES=['compact-gunflint-rooms','compact-gunflint-camp-return','compact-gunflint-frost-return',
          'compact-cedar-ledge-landing','compact-superior-surf-seam','marais-free-inward-care',
          'marais-MnGrandMaraisFieldStation']


def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack',type=Path,required=True)
    parser.add_argument('--expected-pack-sha256',required=True)
    parser.add_argument('--content-identity',required=True)
    parser.add_argument('--renderer',type=Path,required=True)
    parser.add_argument('--saves',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    assert sha(args.pack)==args.expected_pack_sha256
    assert not args.output.exists(),'fresh retained capture epoch required'
    encoded=args.content_identity.encode();binary=bytes.fromhex(args.content_identity)
    compatible=[p for p in args.saves.glob('*.crystalsave') if encoded in p.read_bytes() or binary in p.read_bytes()]
    selected=[]
    for prefix in PREFIXES:
        matches=[p for p in compatible if p.name.startswith(prefix+'-')]
        assert matches,('missing final-identity reached save',prefix)
        selected.append((prefix,max(matches,key=lambda p:int(p.stem.rsplit('-',1)[-1]))))
    args.output.mkdir(parents=True)
    env=dict(os.environ,CRYSTAL_RENDER_TEST_HEADLESS='1',CRYSTAL_RENDER_TEST_CAPTURE_TARGET='image',
             CRYSTAL_RENDER_TEST_WINDOW_SIZE='640x576',RUST_MIN_STACK='33554432')
    receipt=dict(schema_version=1,pack_sha256=args.expected_pack_sha256,content_identity=args.content_identity,
        renderer=str(args.renderer),renderer_sha256=sha(args.renderer),compatible_save_count=len(compatible),
        scope='Actual final-pack production reached saves. No map/position/hour/party/badge/flag overrides. Headless2D presentation captures, not independent earned campaign progression.',
        overrides=[],captures=[])
    for prefix,original in selected:
        folder=args.output/prefix;folder.mkdir();save=folder/original.name;shutil.copyfile(original,save)
        screenshot=folder/'native-2d-original.png'
        command=[str(args.renderer.resolve()),'--pack',str(args.pack.resolve()),'--save',str(save.resolve()),
                 '--view','2d','--screenshot',str(screenshot.resolve())]
        assert not any(v in command for v in ['--map','--x','--y','--hour','--walk','--preview-flag','--preview-doll'])
        with(folder/'renderer.log').open('w')as log:
            result=subprocess.run(command,cwd=ROOT,env=env,stdout=log,stderr=subprocess.STDOUT)
        entry=dict(name=prefix,original_save=str(original),retained_save=str(save),save_sha256=sha(save),
                   command=command,returncode=result.returncode,screenshot=str(screenshot),
                   screenshot_sha256=sha(screenshot)if screenshot.exists()else None,overrides=[])
        receipt['captures'].append(entry)
        (args.output/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
        print(prefix,result.returncode,entry['screenshot_sha256'],flush=True)
        assert result.returncode==0 and screenshot.exists(),'inspect retained renderer.log'
    contact=Image.new('RGB',(2*640,4*608),'#17222b');draw=ImageDraw.Draw(contact)
    for i,entry in enumerate(receipt['captures']):
        x=i%2*640;y=i//2*608
        draw.text((x+8,y+8),entry['name'],fill='white')
        contact.paste(Image.open(entry['screenshot']).convert('RGB'),(x,y+32))
    contact.save(args.output/'reached-contact-original.png')
    contact.resize((640,1216)).save(args.output/'reached-contact-review.png')
    shutil.copyfile(Path(__file__),args.output/Path(__file__).name)
    print(args.output,flush=True)


if __name__=='__main__':main()
