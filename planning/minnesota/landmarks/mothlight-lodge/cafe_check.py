#!/usr/bin/env python3
"""Disposable café journey using real production MCP inputs; see lodge runbook."""
import json, os, subprocess, sys
from pathlib import Path
import play_check as game

frames=[]
def observe():
    o=game.rpc('observe', detail='compact');frames.append(o);return o
def words(o): return ' '.join(r['text'] for k in ['dialogue','menu','prompt','info'] for r in o.get(k,[]))
def finish_text():
    pages=[]
    for _ in range(100):
        o=observe();pages.append(words(o))
        assert 'Battle' not in o['status_line'], 'unexpected battle during text-only interaction'
        if not o.get('dialogue') and not o.get('menu'): return ' '.join(pages)
        game.press('a')
    raise AssertionError(('text fence',o))
def offer():
    game.walk((2,18));game.press('up');game.press('a')
    game.rpc('save')
    capture=Path(game.save).with_suffix('.offer.crystalsave')
    if not capture.exists():capture.write_bytes(Path(game.save).read_bytes())
    pages=[]
    for _ in range(60):
        o=observe();pages.append(words(o))
        if 'Care for a battle?' in words(o) and any('YES' in r['text'] for r in o.get('prompt',[])):
            assert 'BATTLE CAFE' in ' '.join(pages),pages
            game.rpc('save')
            capture=Path(game.save).with_suffix('.question.crystalsave')
            if not capture.exists():capture.write_bytes(Path(game.save).read_bytes())
            return
        game.press('a')
    raise AssertionError(('no offer',o))
def battle():
    game.press('a');seen=False;turns=0;pages=[]
    for _ in range(400):
        o=observe();text=words(o);pages.append(text)
        if 'Battle' in o['status_line']:seen=True
        if seen and 'Overworld' in o['status_line']:
            pages.append(finish_text());break
        menu=[r['text'] for r in o.get('menu',[])]
        # Kindlit's first move is selected only when it is a damaging move.
        if '--lose' not in sys.argv and any('FLAMETHROWER' in t for t in menu):
            for _ in range(4):
                selected=next((r['text'] for r in observe()['menu'] if r.get('kind') == 'selected'), '')
                if 'FLAMETHROWER' in selected:break
                game.press('down')
            turns+=1
        game.press('a')
    else: raise AssertionError(('battle failed to finish',o))
    assert seen,'never entered actual battle'
    print('BATTLE',turns,'turn menus',flush=True)
    return ' '.join(pages)
def restart():
    game.rpc('save');game.client.stdin.close();assert game.client.wait(timeout=20)==0
    assert not game.client.stderr.read();game.start()
def inspect():
    game.rpc('save');return game.inspect()
def choose(label):
    for _ in range(30):
        o=observe()
        selected=next((r['text'] for r in o.get('menu',[]) if r['kind']=='selected'),'')
        if label in selected:
            game.press('a');return
        game.press('down')
    raise AssertionError(('cannot choose',label,o))
def control():
    before=game.pos(game.state());game.move('down');assert game.pos(game.state())!=before
    game.press('start');assert game.state()['menu'];game.press('b')

try:
    game.start();game.walk((30,10));assert game.move('up')['map']=='MnMothlightLodge'
    if '--lose' not in sys.argv:
        game.walk((10,10));game.talk((10,9))
    offer();game.press('down');game.press('a');assert 'cozy' in finish_text()
    assert not inspect()['cafe_today'];control()
    before_battle=inspect()
    offer();text=battle();result=inspect();print('RESULT',result,flush=True)
    if '--lose' in sys.argv:
        assert not result['cafe_today'] and not result['cafe_pending'] and result['juice']==0,result
        assert game.state()['map']=='MnGrandMaraisPokecenter',game.state()
        door=next(w for w in game.maps['MnGrandMaraisPokecenter']['events']['warps'] if w['target_map_constant']=='MN_GRAND_MARAIS')
        game.walk((door['x'],door['y']-1));game.cross('down','MnGrandMarais')
        game.walk((18,3))
        # Retired road guards remain in source rows; cross their open tiles with actual inputs.
        for _ in range(3):game.move('up')
        assert game.pos(game.state())==(18,0)
        game.cross('up','MnMaraisCedarApproach')
        game.walk((30,10));game.move('up');offer();game.press('down');game.press('a');finish_text();control()
        assert not inspect()['cafe_today']
        print('PASS loss, clinic recovery, real walk back and same-day retry offer',flush=True)
    else:
        assert result['cafe_today'],result
        assert sum(m['current_pp'] for m in result['moves']) < sum(m['current_pp'] for m in before_battle['moves']), 'battle must consume real PP'
        if game.full:
            assert result['cafe_pending'] and result['juice']==0,result
            restart();game.walk((2,18));assert 'PACK is full' in game.talk((2,17))
            assert inspect()['cafe_pending'];control()
            if '--midnight-full' in sys.argv:
                game.rpc('save');game.client.stdin.close();assert game.client.wait(timeout=20)==0
                assert not game.client.stderr.read()
                subprocess.check_call([os.environ['MOTHLIGHT_FIXTURE_BIN'],game.pack,game.save,'--next-day'])
                game.start();assert inspect()['cafe_pending'] and not inspect()['cafe_today']
            game.press('start');choose('PACK');choose('POTION');choose('TOSS')
            game.press('a');game.press('a')
            for _ in range(6):game.press('b')
            game.walk((2,18));assert 'BERRY JUICE' in game.talk((2,17))
            claimed=inspect();assert claimed['juice']==1 and not claimed['cafe_pending'],claimed
            if '--midnight-full' in sys.argv:
                offer();game.press('down');game.press('a');finish_text()
            else:
                assert 'tomorrow' in game.talk((2,17))
            assert inspect()['juice']==1;control()
            print('PASS full Pack retains prize across Continue, frees space through Pack, claims exactly once',flush=True)
        else:
            assert not result['cafe_pending'] and result['juice']==1,result
            control();restart();game.walk((2,18));assert 'tomorrow' in game.talk((2,17))
            assert inspect()['juice']==1;control()
            game.rpc('save');game.client.stdin.close();assert game.client.wait(timeout=20)==0
            assert not game.client.stderr.read()
            subprocess.check_call([os.environ['MOTHLIGHT_FIXTURE_BIN'],game.pack,game.save,'--next-day'])
            game.start();offer();battle();assert inspect()['juice']==2;control()
            game.walk((20,9));assert game.move('up')['map']=='MnMothlightLoft';restart()
            game.walk((13,8));game.cross('left','MnMothlightLodge')
            game.walk((13,28));game.cross('down','MnMaraisCedarApproach')
            game.walk((26,23));assert game.move('down')['map']=='MnGrandMarais'
            print('PASS decline, victory, one prize, same-day lock, Continue, simulated daily reset, second victory, loft and return',flush=True)
finally:
    Path(game.save).with_suffix('.cafe-transcript.json').write_text(json.dumps(frames,indent=2))
    if game.client and game.client.poll() is None:
        game.client.stdin.close();assert game.client.wait(timeout=20)==0
        assert not game.client.stderr.read()
