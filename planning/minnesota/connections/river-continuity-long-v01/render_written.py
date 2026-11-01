#!/usr/bin/env python3
"""Render exact written native art for whole-corridor/reference sight review.

This does not execute a GPU renderer or controller; terrain is never modified.
"""
import argparse
import hashlib
import json
from pathlib import Path
import sys
from PIL import Image, ImageDraw
ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
sys.path.insert(0,str(ROOT/'planning/minnesota/connections/river-continuity-v01'))
sys.path.insert(0,str(Path(__file__).resolve().parent))
from render_progress_atlas import font
from render_reference import load_pack, room_sheet
from review_first_bend import composite
from review_long_river import region_camera, full_region


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack',type=Path,required=True)
    parser.add_argument('--expected-sha256',required=True)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    assert not args.output.exists()
    args.output.mkdir(parents=True)
    assert hashlib.sha256(args.pack.read_bytes()).hexdigest()==args.expected_sha256
    pack=load_pack(args.pack)
    manifest=json.loads((ROOT/'planning/minnesota/connections/mississippi-v02/manifest.json').read_text())
    placements=json.loads((ROOT/'planning/minnesota/renders/ice/native-geography-v03/atlas/native-connection-audit.json').read_text())['placements']
    names={r['id']for r in manifest['new_maps']}|{'MnMinneapolis','MnStPaul','MnR04C00R00','MnR03C00R00','MnWinona','MnStCloud','MnR05C00R00','MnR05C00R01','MnR05C00R02'}
    image=composite(pack,placements,names,12,args.output/'raw-compositor-original.png','WRITTEN LONG MISSISSIPPI')
    # Only the annotation band is relabeled; every native map pixel is retained.
    draw=ImageDraw.Draw(image)
    draw.rectangle((0,0,image.width-1,87),fill='#121f2b')
    draw.text((12,9),'WRITTEN LONG MISSISSIPPI / '+args.expected_sha256[:8],font=font(20,True),fill='white')
    draw.text((12,38),'Native source art from the actual written pack. Actual positions; no invented joins.',font=font(14),fill='#c5d9d7')
    draw.text((12,59),'Whole-map art reference, not a single GPU frame. Controller and visual results have separate receipts.',font=font(14),fill='#e6c289')
    image.save(args.output/'whole-corridor-native-art-original.png')
    image.resize((1400,round(image.height*1400/image.width)),Image.Resampling.NEAREST).save(args.output/'whole-corridor-native-art-display.png')
    rows={prefix:full_region(pack,manifest,prefix)[0]for prefix in ['MnMississippiUpstreamWoods','MnMississippiLowerWoods']}
    rows['R05']=[dict(id=f'MnR05C00R{i:02}',source_origin_metatiles=[0,i*24],dimensions_metatiles=[32,pack['data']['maps'][f'MnR05C00R{i:02}']['attributes']['height']])for i in range(3)]
    cameras=[('upper-keeper','MnMississippiUpstreamWoods',(68,23),'east'),('upper-fisher','MnMississippiUpstreamWoods',(32,131),'east'),('lower-fisher','MnMississippiLowerWoods',(20,61),'east'),('lower-keeper','MnMississippiLowerWoods',(102,83),'east'),('r05-lookout','R05',(54,21),'west'),('r05-fisher','R05',(56,89),'east'),('r05-reward','R05',(42,81),'west'),('weak-lower-pool','MnMississippiLowerWoods',(68,34),'east'),('ordinary-lower-descent','MnMississippiLowerWoods',(44,115),'south'),('upper-quiet-pool','MnMississippiUpstreamWoods',(112,82),'north')]
    contact=[]
    for label,prefix,point,facing in cameras:
        im=region_camera(pack,rows[prefix],point,facing)
        im.save(args.output/f'{label}-native160.png')
        im.resize((640,576),Image.Resampling.NEAREST).save(args.output/f'{label}-4x.png')
        contact.append((label+' written art reference',im.convert('RGB')))
    room_sheet(contact,columns=4).save(args.output/'banks-coves-and-ordinary-native-art.png')
    receipt=dict(status='WRITTEN_NATIVE_ART_REFERENCE_NOT_GPU_OR_REACHED_GAMEPLAY',pack_sha256=args.expected_sha256,identity=pack['identity']['content_hash'],map_count=len(pack['data']['maps']),source_tool_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),views=[dict(label=l,region=p,regional_core=list(c),facing=f)for l,p,c,f in cameras],files={p.name:hashlib.sha256(p.read_bytes()).hexdigest()for p in args.output.glob('*.png')})
    (args.output/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')


if __name__=='__main__':
    main()
