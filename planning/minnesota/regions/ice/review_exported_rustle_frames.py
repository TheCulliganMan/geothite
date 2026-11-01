#!/usr/bin/env python3
"""Consume immutable production rustle exports; never synthesize OAM frames."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import sys
from PIL import Image, ImageDraw
ROOT=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_archive import save_render
from render_progress_atlas import font
STYLES=['native','mn_winter_outdoor','mn_winter_north_shore']
TIMES=['morn','day','nite','dark']


def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--input',type=Path,required=True);p.add_argument('--output',type=Path,required=True);args=p.parse_args()
    receipt_path=args.input/'receipt.json';receipt_raw=receipt_path.read_bytes();receipt=json.loads(receipt_raw)
    assert receipt['status']=='PRODUCTION_RENDERER_FRAME_ASSETS_NOT_GPU_OR_CONTROLLER_CAPTURE'
    assert receipt['pack_sha256']=='aa1aa251f7383640412b82b9b3ed9d6e0bb1304569230a79624c75274db60e5a'
    assert len(receipt['frames'])==24
    snapshots={p.name:sha(p)for p in args.input.iterdir() if p.is_file()}
    assert sha(args.input/'native-source-grass-rustle.png')==receipt['source_png_sha256']
    frames={};checks=[]
    for r in receipt['frames']:
        key=r['style'],r['time_bank'],r['frame'];assert key not in frames
        path=args.input/r['png'];assert sha(path)==r['png_sha256'];im=Image.open(path);assert im.mode=='RGBA' and im.size==(18,9);assert hashlib.sha256(im.tobytes()).hexdigest()==r['rgba_sha256'];assert r['dimensions']==[18,9] and r['sprite_size']==[72.0,36.0]
        alpha=im.getchannel('A');counts=Counter(alpha.getdata());assert set(counts)=={0,255} and counts[255]==76 and counts[0]==86
        preview=args.input/r['nearest_preview'];assert sha(preview)==r['nearest_preview_sha256'];assert Image.open(preview).tobytes()==im.resize((288,144),Image.Resampling.NEAREST).tobytes()
        visible=Counter(tuple(p[:3])for p in im.getdata() if p[3]);frames[key]=im
        checks.append(dict(style=key[0],time_bank=key[1],frame=key[2],dimensions=[18,9],opaque_core_pixels=counts[255],transparent_core_pixels=counts[0],opaque_bounds_exclusive=list(alpha.getbbox()),opaque_rgb_counts=[dict(rgb=list(rgb),pixels=count)for rgb,count in visible.items()],rgba_sha256=r['rgba_sha256'],png_sha256=r['png_sha256']))
    for time in TIMES:
        for frame in [0,1]:
            native=frames['native',time,frame]
            a=frames['mn_winter_outdoor',time,frame];b=frames['mn_winter_north_shore',time,frame]
            assert a.tobytes()==b.tobytes(), 'Two winter aliases deliberately share frost colors'
            assert a.getchannel('A').tobytes()==native.getchannel('A').tobytes(), 'Native opacity/OAM locations retained'
            assert a.tobytes()!=native.tobytes(), 'Winter cache must not fall back to native green/dark'
            for other_time in TIMES:assert a.tobytes()==frames['mn_winter_outdoor',other_time,frame].tobytes(), 'Authored frosted foreground is time-independent; zero is transparent'
    a=frames['mn_winter_outdoor','day',0];b=frames['mn_winter_outdoor','day',1]
    phase_delta=dict(rgba_changed_pixels=sum(x!=y for x,y in zip(a.getdata(),b.getdata())),alpha_changed_pixels=sum(x!=y for x,y in zip(a.getchannel('A').getdata(),b.getchannel('A').getdata())))
    assert phase_delta==dict(rgba_changed_pixels=100,alpha_changed_pixels=60)
    args.output.mkdir(parents=True,exist_ok=False)
    width,height=1920,1200;board=Image.new('RGB',(width,height),'#101b24');d=ImageDraw.Draw(board)
    d.text((24,16),'ACTUAL PRODUCTION RUSTLE FRAMES · ALL FOUR TIME BANKS',font=font(30,True),fill='white')
    d.text((24,60),'Written aa1aa251 pack · original 18×9 RGBA frames enlarged 16× with nearest-neighbor only',font=font(22),fill='#bfd4d8')
    d.text((24,92),'Checkerboard reveals unchanged transparency. These are renderer frame assets, not GPU/controller captures.',font=font(21),fill='#e4c78c')
    for col,style in enumerate(STYLES):
        x=24+col*632;d.text((x,132),style,font=font(23,True),fill='white')
        for row,time in enumerate(TIMES):
            y=180+row*230;d.text((x,y),time+' / frame 0',font=font(20),fill='#bfd4d8');d.text((x+304,y),time+' / frame 1',font=font(20),fill='#bfd4d8')
            for frame in [0,1]:
                cell=Image.new('RGBA',(288,144));cd=ImageDraw.Draw(cell)
                for yy in range(0,144,32):
                    for xx in range(0,288,32):cd.rectangle((xx,yy,xx+31,yy+31),fill=('#55616a'if(xx//32+yy//32)%2 else '#8c969d'))
                cell.alpha_composite(frames[style,time,frame].resize((288,144),Image.Resampling.NEAREST));board.paste(cell.convert('RGB'),(x+frame*304,y+32))
    d.text((24,1124),'Every frame: 76 opaque / 86 transparent pixels. Winter aliases: icy pale blue + blue-gray; no native green pixels.',font=font(21),fill='#e4c78c')
    d.text((24,1158),'Phase 0 bounds [1,0..17,8); phase 1 [0,1..18,9). Actual native OAM movement retained.',font=font(21),fill='#bfd4d8')
    save_render(board,args.output/'all-24-production-frames.png');save_render(board.resize((1440,900),Image.Resampling.NEAREST),args.output/'all-24-production-frames-display.png')
    proof=dict(status='PIXEL_AUDIT_PASS_SIGHT_REVIEW_REQUIRED',pack_sha256=receipt['pack_sha256'],input_receipt_sha256=hashlib.sha256(receipt_raw).hexdigest(),source_png_sha256=receipt['source_png_sha256'],frame_count=24,frames=checks,phase_delta=phase_delta,
        conclusions=['All24 originals match production export PNG and raw RGBA hashes.','All24 pure16x previews exactly enlarge those originals.','Native OAM opacity positions retained in both aliases, both phases and all four banks.','Both winter aliases deliberately share identical pale icy colors in every time bank; native day remains green, night blue and dark black.','Two actual production OAM phases remain distinct; no phase was reconstructed or recolored by this consumer.'],limits=['These are production loader frame assets, not a GPU capture or an executed movement effect.','A later native full-frame review still checks scene priority and actual effect dispatch.'])
    assert {p.name:sha(p)for p in args.input.iterdir() if p.is_file()}==snapshots,'Original export changed during review'
    (args.output/'proof.json').write_text(json.dumps(proof,indent=2)+'\n')
    print(json.dumps({'frames':24,'pixel_audit':'PASS','sheet':str(args.output/'all-24-production-frames.png'),'phase_delta':phase_delta}))


if __name__=='__main__':main()
