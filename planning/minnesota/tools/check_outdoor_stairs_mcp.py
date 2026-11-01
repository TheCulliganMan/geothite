#!/usr/bin/env python3
"""Check native outdoor stairs through ordinary production MCP input.

Requires exact-pack fixture saves authored before any input. No coordinates,
flags, encounters or runtime state are rewritten during a case. Retain failures.
"""
import argparse
import concurrent.futures
import hashlib
import json
from pathlib import Path
import subprocess

from check_mina_battle_mcp import LiveRecorded


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run_case(args, flight):
    name = flight['id']; folder = args.output/name; folder.mkdir(exist_ok=False)
    source = args.fixtures/(name+'.crystalsave'); original = sha(source)
    stderr_path = folder/'stderr.log'; records = []; process = None; error = None
    checks = []
    try:
        with stderr_path.open('x') as stderr:
            process = subprocess.Popen(
                [str(args.client),'mcp',str(args.pack),'--load',str(source),
                 '--save',str(folder/'actual-autosave.crystalsave')],
                stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=stderr,
                text=True,bufsize=1)
            client = LiveRecorded(process, records)
            client.rpc('initialize',dict(protocolVersion='2025-06-18',capabilities={},
                       clientInfo=dict(name='outdoor-native-stair-check',version='1')))
            def at(point):
                status = client.status()
                assert status['map']==flight['map'] and status['position']==dict(zip(['x','y'],point)),status
                assert status['mode']=='overworld' and not status['dialogue'],status
                assert status['money']==10000,status
                return status
            before = at(flight['upper'])
            if flight.get('kind')=='walk-course':
                current = flight['upper']
                for command in flight['commands']:
                    direction = command['direction']
                    assert direction in ['up', 'down', 'left', 'right']
                    if command['action']=='walk':
                        assert 0 < command['steps'] <= 8
                        client.move(direction, command['steps'])
                        current = command['to']
                        after = at(current)
                        checks.append('walked-'+direction)
                    else:
                        assert command['action']=='deny'
                        for _ in range(3):
                            client.press(direction)
                            after = at(current)
                        checks.append('denied-three-presses-'+direction)
                assert current == flight['upper'], 'course must return to its initial point'
            elif flight.get('kind')=='cliff-corner':
                for direction in [flight['outward'],'down']:
                    client.call('press',button=direction,times=3);at(flight['upper'])
                    checks.append('denied-'+direction)
                client.move(flight['inward'],1);at(flight['inside'])
                client.move(flight['outward'],1);after=at(flight['upper'])
                checks.append('crossed-inside-rim-both-directions')
            elif flight.get('kind')=='setback-boulder':
                for point in [flight['upper'],flight['right']]:
                    if point==flight['right']:
                        client.move('right',1);at(point)
                    for direction in ['up','down']:
                        client.call('press',button=direction,times=3);at(point)
                        checks.append('denied-'+direction)
                client.move('left',1);after=at(flight['upper'])
                checks.append('crossed-cleared-rim-both-directions')
            else:
                north = (flight.get('orientation')=='north'
                         or flight.get('kind')=='retained native north-facing ramp')
                outward, inward = ('up','down') if north else ('down','up')
                client.move(outward,1);at(flight['step'])
                for direction in (['left'] if north else ['left','right']):
                    client.call('press',button=direction,times=3);at(flight['step'])
                    checks.append('denied-'+direction)
                client.move(outward,1);at(flight['lower']);checks.append('reached-outside-landing')
                client.move(inward,2);after=at(flight['upper']);checks.append('returned-to-inside-landing')
            for key in ['party','bag']:
                if key in before:assert before[key]==after[key],key
            process.stdin.close();process.wait(timeout=30)
            assert process.returncode==0,process.returncode
            if flight.get('verify_continue'):
                reached = folder/'actual-autosave.crystalsave'
                reached_hash = sha(reached)
                process = subprocess.Popen(
                    [str(args.client),'mcp',str(args.pack),'--load',str(reached),
                     '--save',str(folder/'continued-autosave.crystalsave')],
                    stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=stderr,
                    text=True,bufsize=1)
                client = LiveRecorded(process, records)
                client.rpc('initialize',dict(protocolVersion='2025-06-18',capabilities={},
                           clientInfo=dict(name='outdoor-reached-save-continue',version='1')))
                at(flight['upper'])
                client.press('start')
                view = client.observe()['structuredContent']
                assert any(line['text'].strip(' >').upper()=='SAVE'
                           for line in view['menu']), view
                client.press('b')
                at(flight['upper'])
                process.stdin.close();process.wait(timeout=30)
                assert process.returncode==0 and sha(reached)==reached_hash
                checks.append('actual-reached-save-continue-and-production-start')
    except BaseException as exc:
        error = repr(exc)
    finally:
        if process is not None and process.poll() is None:
            process.terminate();process.wait(timeout=30)
    stderr = stderr_path.read_text()
    if stderr and error is None:error='native stderr was not empty'
    if sha(source)!=original:error='initial fixture was modified'
    result = dict(status='PASS' if error is None else 'FAIL',flight=flight,error=error,
                  fixture_sha256=original,checks=checks,calls=records,stderr=stderr,
                  exit_code=None if process is None else process.returncode)
    (folder/'receipt.json').write_text(json.dumps(result,indent=2)+'\n')
    print(name,result['status'],error or '',flush=True)
    return result


def main():
    cli=argparse.ArgumentParser(description=__doc__)
    for key in ['pack','inventory','fixtures','client','output']:
        cli.add_argument('--'+key,type=Path,required=True)
    cli.add_argument('--expected-cases',type=int,default=26)
    args=cli.parse_args();args.client=args.client.resolve();args.output.mkdir(exist_ok=False,parents=True)
    inputs={str(p):sha(p) for p in [args.pack,args.inventory,args.client,Path(__file__)]}
    flights=json.loads(args.inventory.read_text())
    assert args.expected_cases>0 and len(flights)==args.expected_cases
    assert len({f['id'] for f in flights})==args.expected_cases
    with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
        results=list(pool.map(lambda f:run_case(args,f),flights))
    unchanged=all(sha(Path(p))==h for p,h in inputs.items())
    report=dict(status='PASS' if unchanged and all(r['status']=='PASS' for r in results) else 'FAIL',
                scope='Explicit local position/party/Repel fixtures; all subsequent actions through production VisibleShellController-backed stdio MCP. No earned access claim.',
                inputs=inputs,inputs_unchanged=unchanged,cases=[dict(id=r['flight']['id'],status=r['status'],error=r['error']) for r in results],
                calls=sum(len(r['calls']) for r in results))
    (args.output/'receipt.json').write_text(json.dumps(report,indent=2)+'\n')
    assert report['status']=='PASS','Inspect each retained native failure'


if __name__=='__main__':main()
