#!/usr/bin/env python3
"""Trigger Mina from her reachable lower bench and execute one visible turn."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import select
import shutil
import subprocess

from check_minneapolis_mcp import Client


def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()


class LiveRecorded(Client):
    def __init__(self, process, records):
        super().__init__(process,timeout=30);self.records=records
    def rpc(self,method,params):
        self.sequence+=1;request=dict(jsonrpc='2.0',id=self.sequence,method=method,params=params)
        row=dict(request=request);self.records.append(row)
        self.process.stdin.write(json.dumps(request)+'\n');self.process.stdin.flush()
        while not select.select([self.process.stdout],[],[],30)[0]:
            assert self.process.poll() is None,'MCP process ended before its response'
            print('Waiting on same live MCP response',self.sequence,flush=True)
        raw=self.process.stdout.readline();assert raw,'native MCP EOF'
        response=json.loads(raw);row['response']=response
        assert response.get('id')==self.sequence and 'error' not in response,response
        result=response['result'];assert not result.get('isError'),result
        return result


def main():
    cli=argparse.ArgumentParser(description=__doc__)
    for key in ['pack','save','client','output']:cli.add_argument('--'+key,type=Path,required=True)
    args=cli.parse_args();args.client=args.client.resolve();args.output.mkdir(exist_ok=False,parents=True)
    inputs={str(p):sha(p) for p in [args.pack,args.save,args.client]};records=[];process=None;error=None;checks=[]
    try:
        with (args.output/'stderr.log').open('x') as stderr:
            autosave=args.output/'actual-autosave.crystalsave'
            process=subprocess.Popen([str(args.client),'mcp',str(args.pack),'--load',str(args.save),'--save',str(autosave)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=stderr,text=True,bufsize=1)
            c=LiveRecorded(process,records);c.rpc('initialize',dict(protocolVersion='2025-06-18',capabilities={},clientInfo=dict(name='mina-lower-bench-check',version='1')))
            c.at('MnR09C00R00',44,36);c.move('down',1);c.at('MnR09C00R00',44,37)
            observed=[]
            def command_menu():
                for _ in range(96):
                    o=c.observe();observed.append(o)
                    view=o['structuredContent'];menu=' '.join(line['text'] for line in view['menu']).upper()
                    if all(word in menu for word in ['FIGHT','PKMN','PACK','RUN']) and not view['dialogue'] and not view['prompt']:return o
                    # A knockout can offer the ordinary optional party switch.
                    # Decline it using input instead of confirming the active
                    # partner repeatedly or rewriting the battle cursor.
                    party = 'CANCEL' in menu and any(name in menu for name in ['BOREALYNX','VOYAGEAR'])
                    c.press('b' if view['prompt'] or party else 'a')
                raise AssertionError('Mina did not yield the visible production battle command menu')
            first=command_menu();checks.append('lower-bench-native-challenge-and-command-menu')
            (args.output/'battle-command-menu.json').write_text(json.dumps(first,indent=2)+'\n')
            c.press('a');before=c.observe();(args.output/'move-menu-before.json').write_text(json.dumps(before,indent=2)+'\n')
            assert before['structuredContent']['menu'],'production move menu missing'
            shutil.copyfile(autosave,args.output/'before-selected-move.crystalsave')
            c.press('a');turn=c.observe();(args.output/'after-selected-move.json').write_text(json.dumps(turn,indent=2)+'\n')
            assert turn['structuredContent']['dialogue'],'actual move must expose battle dialogue'
            shutil.copyfile(autosave,args.output/'after-selected-move.crystalsave')
            command_menu();c.press('a');after=c.observe();(args.output/'move-menu-after.json').write_text(json.dumps(after,indent=2)+'\n')
            pattern=r'PP\s*:?\s*(\d+)\s*/\s*(\d+)'
            old=re.findall(pattern,json.dumps(before),re.IGNORECASE);new=re.findall(pattern,json.dumps(after),re.IGNORECASE)
            if old and new:assert int(new[0][0])<int(old[0][0]),(old,new)
            checks.append('visible-selected-move-and-battle-dialogue')
            (args.output/'pp-display-comparison.json').write_text(json.dumps(dict(before=old,after=new,native_save_verification_required=True),indent=2)+'\n')
            (args.output/'visible-pages.json').write_text(json.dumps(observed,indent=2)+'\n')
            process.stdin.close();process.wait(timeout=30);assert process.returncode==0
    except BaseException as exc:error=repr(exc)
    finally:
        if process is not None and process.poll() is None:process.terminate();process.wait(timeout=30)
    stderr=(args.output/'stderr.log').read_text();unchanged=all(sha(Path(p))==h for p,h in inputs.items())
    report=dict(status='PASS' if error is None and not stderr and unchanged else 'FAIL',error=error,inputs=inputs,inputs_unchanged=unchanged,checks=checks,stderr=stderr,calls=records,scope='Explicit owned healthy level60 Minnesota partners and initial lower-bench placement. All later approach/dialogue/selection/turn inputs through production MCP; no earned campaign claim. Native PP save verification is separate.')
    (args.output/'receipt.json').write_text(json.dumps(report,indent=2)+'\n');assert report['status']=='PASS',error
    print('Mina lower-bench challenge, FIGHT/move menus and selected turn passed; validate native saved PP next.',flush=True)


if __name__=='__main__':main()
