#!/usr/bin/env python3
"""Replay both real Rock Cut mouths and the short drop through production stdio.

Starts from controller-created native checkpoints. Never rewrites game state.
Controller tests separately establish healing, Repel consumption and Continue.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT/'planning/minnesota/tools'))
from check_mina_battle_mcp import LiveRecorded
from check_north_shore_story_mcp import finish
from check_winona_mcp import walk
from render_reference import load_pack

ROOM = 'MnR05RockCut'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['pack', 'client', 'saves', 'inspector', 'output']:
        parser.add_argument('--'+name, type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    pack = load_pack(args.pack)
    idfile = args.output/'modpack-id.txt'
    idfile.write_text(pack['identity']['runtime_modpack_id'])
    results = []
    for case in ['north-clear', 'repel-continued']:
        folder = args.output/case
        folder.mkdir()
        source, save = args.saves/(case+'.crystalsave'), folder/'actual-autosave.crystalsave'
        bindings = {str(p):sha(p) for p in [args.pack,args.client,args.inspector,source]}
        records, error, process, inspection = [], None, None, None
        try:
            with (folder/'stderr.log').open('x') as err:
                process = subprocess.Popen([str(args.client.resolve()),'mcp',str(args.pack.resolve()),
                                            '--load',str(source.resolve()),'--save',str(save.resolve())],
                                           stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=err,text=True,bufsize=1)
                c = LiveRecorded(process,records)
                c.rpc('initialize',dict(protocolVersion='2025-06-18',capabilities={},clientInfo=dict(name='rockcut-shelf',version='1')))
                c.at(ROOM,*([11,4] if case=='north-clear' else [6,5]))
                money = c.status()['money']
                walk(c,pack,(11,4))
                c.move('down');c.at('MnR05C00R00',30,28)
                c.move('up');c.at(ROOM,11,5)
                c.move('up');c.at(ROOM,11,4)
                for x in [6,7]:
                    walk(c,pack,(x,5))
                    c.move('down');c.at(ROOM,x,6)
                    c.move('down');c.at(ROOM,x,8)
                    c.move('up');c.at(ROOM,x,8)
                    shutil.copy2(save,folder/f'actual-drop-{x}.crystalsave')
                    for at in [(6,8),(7,8),(8,8),(9,8),(9,7),(9,6),(9,5),(8,5),(7,5),(6,5)]:
                        walk(c,pack,at)
                if case=='north-clear':
                    walk(c,pack,(6,13));c.press('left');c.press('a')
                    pages=finish(c)
                    assert 'Rest a while.' in pages and 'same old hill.' in pages,pages
                walk(c,pack,(5,16))
                c.move('down');c.at('MnR05C00R01',16,30)
                c.move('up');c.at(ROOM,5,17)
                c.move('up');c.at(ROOM,5,16)
                assert c.status()['money']==money
                c.press('start');assert any('PACK' in row['text'] for row in c.observe()['structuredContent']['menu']);c.press('b')
                shutil.copy2(save,folder/'actual-course-complete.crystalsave')
                checked=subprocess.run([str(args.inspector.resolve()),str(idfile),sha(args.pack),pack['identity']['content_hash'],str(save)],capture_output=True,text=True)
                assert checked.returncode==0,checked.stderr
                assert 'PARTY_SURF_MOVES=0' in checked.stdout and 'MODE=Normal' in checked.stdout
                inspection=checked.stdout
                process.stdin.close();assert process.wait(timeout=30)==0
        except BaseException as exc:
            error=repr(exc)
        finally:
            if process is not None and process.poll() is None:
                process.terminate();process.wait(timeout=30)
        stderr=(folder/'stderr.log').read_text()
        unchanged=all(sha(Path(p))==value for p,value in bindings.items())
        receipt=dict(status='PASS' if error is None and not stderr and unchanged else 'FAIL',error=error,bindings=bindings,
                     inputs_unchanged=unchanged,native_save_inspection=inspection,stderr=stderr,calls=records,scope=__doc__)
        (folder/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
        results.append(dict(case=case,status=receipt['status'],rpc_calls=len(records)))
        (args.output/'receipt.json').write_text(json.dumps(results,indent=2)+'\n')
        assert receipt['status']=='PASS',error
        print(case,'PASS',len(records),'RPC calls',flush=True)


if __name__=='__main__':
    main()
