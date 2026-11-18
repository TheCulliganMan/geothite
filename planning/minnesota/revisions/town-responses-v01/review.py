#!/usr/bin/env python3
"""Static native-asset storyboard; no gameplay, pack writing or save mutation."""
import argparse
import hashlib
import io
import json
from pathlib import Path
import sys

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from render_reference import SourceArt, load_pack
from review_grand_marais import sprite

RESIDENTS = [
    ('MnStCloud', 'MN_ST_CLOUD_STONECUTTER', 'MnTownStonecutterAfterText'),
    ('MnStCloud', 'MN_ST_CLOUD_RIVER_WATCHER', 'MnTownRiverWatcherAfterText'),
    ('MnDuluth', 'MN_DULUTH_X_DOCK_HAND', 'MnTownDockHandAfterText'),
    ('MnDuluth', 'MnDuluth_MN_DULUTH_ISLAND_ARTIST', 'MnTownPainterAfterText'),
]


def pages(rows):
    result = []
    for row in rows:
        op = row['command']
        if op in ('text', 'para'):
            result.append([])
        if op in ('text', 'para', 'line'):
            result[-1].append(row['args'][0].strip('"'))
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('pack', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--installed', action='store_true', help='Require return text in written pack')
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    pack = load_pack(args.pack)
    source = json.loads(Path(__file__).with_name('dialogue.json').read_text())
    maps = pack['data']['maps']
    release = maps['MnStCloud']['scripts']['MnArcSCScene']
    assert release[-2] == {'command':'setevent','args':['EVENT_MN_REED_WORKER_RELEASED']}
    assert any(r == {'command':'iffalse','args':['MnArcSCDeclined']} for r in release)
    restored = maps['MnBoundaryIntake']['scripts']['MnVerdigrimBypass']
    assert restored[-2] == {'command':'setevent','args':['EVENT_MN_SURVEY_RESTORED']}
    assert restored.index({'command':'setevent','args':['EVENT_MN_KETTLURK_RELEASED']}) < len(restored)-2
    files = pack['runtime_files']
    font = Image.open(io.BytesIO(bytes(files['gfx/font/font.png']))).convert('RGB')
    frame = Image.open(io.BytesIO(bytes(files['gfx/frames/1.png']))).convert('RGB')
    assert font.size == (128, 64) and frame.size == (24, 16)
    glyphs = {c: 0x80+i for i,c in enumerate('ABCDEFGHIJKLMNOPQRSTUVWXYZ')}
    glyphs.update({c: 0xa0+i for i,c in enumerate('abcdefghijklmnopqrstuvwxyz')})
    glyphs.update({':':0x9c, "'":0xe0, '-':0xe3, '?':0xe6, '!':0xe7, '.':0xe8, ',':0xf4})
    art = SourceArt(pack)
    all_sheets = []
    count = 0
    for map_id, script, label in RESIDENTS:
        module = pack['data']['maps'][map_id]
        actor = next(o for o in module['objects'] if o['script'] == script)
        a = module['attributes']
        scene = art.render(a['tileset_name'], a['width'], a['height'], module['blocks']).convert('RGBA')
        # Only the unchanged resident is staged in this static camera, not a
        # fabricated player session or a guessed callback visibility state.
        scene.alpha_composite(sprite(pack, actor['sprite']), (actor['x']*16, actor['y']*16-4))
        camera = scene.crop((actor['x']*16-72, actor['y']*16-40,
                             actor['x']*16+88, actor['y']*16+104)).convert('RGB')
        before = source[map_id][script+'Text']
        after = source[map_id][label]
        if args.installed:
            assert pages(module['scripts'][label]) == after, label
            assert pages(module['scripts'][script+'Text']) == before, script
            check, flag = {
                'MN_ST_CLOUD_STONECUTTER': ('checkevent', 'EVENT_MN_REED_WORKER_RELEASED'),
                'MN_ST_CLOUD_RIVER_WATCHER': ('checkevent', 'EVENT_MN_SURVEY_RESTORED'),
                'MN_DULUTH_X_DOCK_HAND': ('checkevent', 'EVENT_MN_SURVEY_RESTORED'),
                'MnDuluth_MN_DULUTH_ISLAND_ARTIST': ('checkevent', 'EVENT_MN_SURVEY_RESTORED'),
            }[script]
            assert module['scripts'][script] == [
                {'command':'faceplayer','args':[]},
                {'command':check,'args':[flag]},
                {'command':'iftrue','args':[script+'After']},
                {'command':'jumptext','args':[script+'Text']},
            ]
            assert module['scripts'][script+'After'] == [{'command':'jumptext','args':[label]}]
            assert {'source_script':script, 'command_index':1, 'command':check,
                    'flag_id':flag} in module['script_flag_commands']
            assert any(row['source_script']==script and row['command_index']==2
                       and row['command']=='iftrue' and row['resolved_target_script']==script+'After'
                       for row in module['script_control_commands'])

        sheet = Image.new('RGB', (max(len(before),len(after))*320, 640), '#25313a')
        draw = ImageDraw.Draw(sheet)
        for state, conversation in enumerate([before, after]):
            draw.text((8, state*320+4), f'{script}: '+('FIRST VISIT' if state == 0 else 'RETURN'), fill='white')
            for page_index, lines in enumerate(conversation):
                assert 1 <= len(lines) <= 2
                view = camera.copy()
                # Production field box: 20x6 tiles at (0,12), baselines 14,16.
                view.paste('white', (0,96,160,144))
                for y in range(6):
                    for x in range(20):
                        if y == 0: tile = (0,0) if x==0 else (16,0) if x==19 else (8,0)
                        elif y == 5: tile = (8,8) if x==0 else (16,8) if x==19 else (8,0)
                        elif x in (0,19): tile = (0,8)
                        else: continue
                        sx,sy=tile
                        view.paste(frame.crop((sx,sy,sx+8,sy+8)), (x*8,96+y*8))
                for row,line in enumerate(lines):
                    assert line and line.isascii() and len(line)<=18
                    for col,c in enumerate(line):
                        if c==' ': continue
                        index=glyphs[c]-0x80
                        sx,sy=(index%16)*8,(index//16)*8
                        view.paste(font.crop((sx,sy,sx+8,sy+8)), (8+col*8,112+row*16))
                view.save(args.output/f'{script}-{state}-{page_index+1}.png')
                sheet.paste(view.resize((320,288),Image.Resampling.NEAREST),(page_index*320,state*320+24))
                count+=1
        sheet.save(args.output/f'{script}.png')
        all_sheets.append(sheet)
    combined=Image.new('RGB',(max(s.width for s in all_sheets),sum(s.height for s in all_sheets)),'#25313a')
    y=0
    for sheet in all_sheets: combined.paste(sheet,(0,y));y+=sheet.height
    combined.save(args.output/'all-conversations.png')
    receipt={'pack_sha256':hashlib.sha256(args.pack.read_bytes()).hexdigest(), 'pages':count,
             'installed_text_checked':args.installed, 'gameplay_inputs':0,
             'method':'Static native pack art/font/frame storyboard; not client screenshots'}
    (args.output/'review.json').write_text(json.dumps(receipt,indent=2)+'\n')
    print(json.dumps(receipt))


if __name__ == '__main__': main()
