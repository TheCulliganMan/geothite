#!/usr/bin/env python3
"""Retain all three exact winter source wholes in a separate-panel review board."""
import argparse
import hashlib
import json
from pathlib import Path
import sys

from PIL import Image,ImageDraw,ImageFont
ROOT=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_archive import save_render


def digest(path):return hashlib.sha256(path.read_bytes()).hexdigest()


def main(output):
    base=ROOT/'planning/minnesota/renders/ice';v2=base/'region-v01/final-v02-native-slots';v3=base/'region-v01/final-v03-native-music'
    assert (v2/'MnRainyRiverIcefields.layout.json').read_bytes()==(v3/'MnRainyRiverIcefields.layout.json').read_bytes()
    a=json.loads((v2/'MnRainyRiverIcefields.json').read_text());b=json.loads((v3/'MnRainyRiverIcefields.json').read_text())
    assert [k for k in a if a[k]!=b[k]]==['music']
    rows=[('Rainy River Icefields','Complete winter region | 32 x 24 metatiles',v2/'whole-native-source.png'),
          ('Border Pines / R17','Existing route winter section; stepped southern snowline',base/'sections-v01/review-v03/r17/after/whole-source-scene.png'),
          ('North Shore / R11','Existing upper gorge frost; open Lake Superior retained',base/'sections-v01/review-v03/r11/after/whole-source-scene.png')]
    output.mkdir(parents=True,exist_ok=False);sources=output/'originals';sources.mkdir()
    images=[];receipts=[]
    for i,(title,caption,path) in enumerate(rows):
        image=Image.open(path).convert('RGB');save_render(image,sources/f'whole-{i+1}.png');images.append(image)
        receipts.append({'title':title,'source':str(path.relative_to(ROOT)),'sha256':digest(path),'size':list(image.size)})
    font_path=Path('/System/Library/Fonts/Supplemental/Arial.ttf')
    font=lambda size:ImageFont.truetype(str(font_path),size) if font_path.exists() else ImageFont.load_default()
    pad=32;header=180;board=Image.new('RGB',(sum(im.width for im in images)+pad*4,max(im.height for im in images)+header+60),(20,28,37));d=ImageDraw.Draw(board)
    d.text((pad,20),'MINNESOTA WINTER — SOURCE PREVIEW',font=font(48),fill='white')
    d.text((pad,82),'Native gameplay verification pending. Separate review panels; this is not connected geography.',font=font(28),fill='#e1c991')
    d.text((pad,122),'Whole maps retain their native 32px metatile scale. Source player-camera insets are labelled separately.',font=font(25),fill='#bacbd7')
    x=pad
    for (title,caption,_),im in zip(rows,images):
        d.text((x,header),title,font=font(36),fill='white');d.text((x,header+44),caption,font=font(20),fill='#bacbd7');board.paste(im,(x,header+80));x+=im.width+pad
    y=header+80+images[0].height+70
    for label,name in [('Weakest ordinary south bank / source camera 4x','WEAKEST_SOUTH_BANK-4x.png'),('Central stopping tongue / source camera 4x','MID_CHANNEL-4x.png')]:
        d.text((pad,y),label,font=font(24),fill='white');im=Image.open(v2/name).convert('RGB');save_render(im,sources/name);board.paste(im,(pad,y+38));y+=im.height+90
    save_render(board,output/'board-full-resolution.png')
    save_render(board.resize((board.width//2,board.height//2),Image.Resampling.NEAREST),output/'board-display.png')
    (output/'proof.json').write_text(json.dumps({'status':'SOURCE_BOARD_ONLY','native_gameplay_verification':'pending','whole_map_scale':'one source pixel per board pixel, no resizing',
       'rainy_v03_layout_exact_v02':True,'rainy_v03_only_dossier_difference':'music','originals':receipts,
       'rainy_v03_dossier_sha256':digest(v3/'MnRainyRiverIcefields.json'),'rainy_v03_layout_sha256':digest(v3/'MnRainyRiverIcefields.layout.json'),
       'board_sha256':digest(output/'board-full-resolution.png')},indent=2)+'\n')


if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('output',type=Path);args=p.parse_args();main(args.output)
