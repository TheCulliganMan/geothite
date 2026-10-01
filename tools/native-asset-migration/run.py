#!/usr/bin/env python3
"""Temporary, pinned bootstrap for one exact same-repository PR conversion.

prepare applies the reviewed patch and reconstructs assets in a disposable checkout.
finish is called only after the workflow's Python, native and Wasm checks pass.
It removes the temporary bootstrap/workflow, stages an exact allowlist, and only
publishes when --publish is explicitly present. No token or auth config is read.
"""
from pathlib import Path
import argparse
import hashlib
import json
import re
import subprocess
import sys

REPOSITORY = 'TheCulliganMan/geothite'
BRANCH = 'feat/connected-johto-3d'
PULL_REQUEST = 5
WORKFLOW = '.github/workflows/materialize-native-art.yml'
BOOTSTRAP = 'tools/native-asset-migration'
BOOTSTRAP_FILES = [f'{BOOTSTRAP}/run.py', f'{BOOTSTRAP}/integration.patch', f'{BOOTSTRAP}/README.md']
PATCH_SHA256 = '5f3d2e034fee5b212319eab2ba92106979c1141f7ce282dd0fb09809a5b8c9ea'
ROOT = Path(__file__).resolve().parents[2]

def run(*args, capture=True):
    return subprocess.run(args, cwd=ROOT, check=True, stdout=subprocess.PIPE if capture else None,
                          stderr=subprocess.PIPE if capture else None).stdout

def git(*args):
    return run('git', *args).decode().strip()

def paths_between(before, after):
    return set(run('git', 'diff', '--name-only', '-z', before, after).decode().rstrip('\0').split('\0')) - {''}

def require(condition, explanation):
    if not condition:
        raise ValueError(explanation)

def remote_tip():
    lines=git('ls-remote','--exit-code','origin',f'refs/heads/{BRANCH}').splitlines()
    require(len(lines)==1,'remote branch is absent or ambiguous')
    sha,ref=lines[0].split('\t')
    require(ref==f'refs/heads/{BRANCH}','remote branch identity differs')
    return sha

def guard(expected_m, event_path):
    require(re.fullmatch('[0-9a-f]{40}',expected_m) is not None,'reviewed bootstrap SHA must be exact')
    event=json.loads(event_path.read_text());pr=event['pull_request']
    require(event['action']=='synchronize' and event['number']==PULL_REQUEST,'unexpected pull request event')
    require(event['repository']['full_name']==REPOSITORY,'unexpected event repository')
    require(pr['head']['repo']['full_name']==REPOSITORY and pr['base']['repo']['full_name']==REPOSITORY,'fork or different repository')
    require(pr['head']['ref']==BRANCH,'unexpected feature branch')
    w=pr['head']['sha']
    require(re.fullmatch('[0-9a-f]{40}',w) is not None,'invalid event head SHA')
    require(git('rev-parse','HEAD')==w,'checkout is not the exact event head')
    parents=git('show','-s','--format=%P',w).split()
    require(parents==[expected_m],'workflow commit must be the single-parent child of reviewed M')
    require(paths_between(expected_m,w)=={WORKFLOW},'W changed files other than its one temporary workflow')
    require(git('diff','--diff-filter=A','--name-only',expected_m,w)==WORKFLOW,'temporary workflow must be newly added')
    m_parents=git('show','-s','--format=%P',expected_m).split()
    require(len(m_parents)==1,'bootstrap M must have exactly one parent')
    require(paths_between(m_parents[0],expected_m)==set(BOOTSTRAP_FILES),'M changed the existing game or files outside bootstrap')
    require(set(git('diff','--diff-filter=A','--name-only',m_parents[0],expected_m).splitlines())==set(BOOTSTRAP_FILES),'bootstrap M must only add new files')
    require(hashlib.sha256((ROOT/BOOTSTRAP/'integration.patch').read_bytes()).hexdigest()==PATCH_SHA256,'reviewed UTF-8 integration patch differs')
    require(remote_tip()==w,'feature branch has advanced; refusing stale conversion')
    return w

def migration_report(destination, check):
    arguments=[sys.executable,'tools/migrate_asset_storage.py','--report',str(destination)]
    if check:arguments.append('--check')
    run(*arguments)
    return json.loads(destination.read_text())

def identities_equal(before,after):
    require(before['source_identities']==after['source_identities'],'editable source identities changed')
    require(before['model_identities']==after['model_identities'],'runtime model identities changed')

def changed_paths():
    entries=run('git','status','--porcelain=v1','-z','--untracked-files=all').decode().split('\0')
    result=set()
    for entry in entries:
        if not entry:continue
        require(len(entry)>3 and entry[2]==' ','malformed worktree status')
        require(not any(c in entry[:2] for c in 'RCU'),'renames, copies and conflicts are not permitted')
        result.add(entry[3:])
    return result

def prepare(args,w):
    require(not changed_paths(),'bootstrap checkout must start clean')
    patch=ROOT/BOOTSTRAP/'integration.patch'
    patch.read_text(encoding='utf-8')
    numstat=run('git','apply','--numstat','-z',str(patch)).decode().split('\0')
    integration=set()
    for entry in numstat:
        if not entry:continue
        fields=entry.split('\t',2)
        require(len(fields)==3 and fields[0]!='-' and fields[1]!='-','integration patch must contain UTF-8 source changes only')
        relative=fields[2]
        require(not Path(relative).is_absolute() and '..' not in Path(relative).parts,'unsafe integration path')
        integration.add(relative)
    run('git','apply','--check',str(patch))
    run('git','apply',str(patch))
    before=migration_report(args.state.parent/'before.json',True)
    after=migration_report(args.state.parent/'after.json',False)
    identities_equal(before,after)
    run(sys.executable,'tools/build_art_index.py')
    allowed=set(before['changed_paths'])|integration|set(BOOTSTRAP_FILES)|{WORKFLOW}
    require(changed_paths()<=allowed,'preparation changed an unexpected path')
    state={'m':args.expected_m,'w':w,'patch_sha256':PATCH_SHA256,'allowed':sorted(allowed),'before':before}
    args.state.write_text(json.dumps(state,indent=2)+'\n')
    print(f'Prepared {after["sources"]} native Blender sources and {after["models"]} ordinary gzip models; every identity preserved')

def finish(args,w):
    state=json.loads(args.state.read_text())
    require(state['m']==args.expected_m and state['w']==w and state['patch_sha256']==PATCH_SHA256,'preparation receipt differs')
    require(args.validation_stamp.read_text().strip()==w,'successful workflow validation stamp is missing')
    after=migration_report(args.state.parent/'final.json',True)
    identities_equal(state['before'],after)
    allowed=set(state['allowed'])
    require(changed_paths()<=allowed,'tests or generation changed an unexpected path')
    for relative in [WORKFLOW,*BOOTSTRAP_FILES]:
        (ROOT/relative).unlink()
    (ROOT/BOOTSTRAP).rmdir()
    actual=changed_paths()
    require(actual<=allowed,'unexpected path before staging')
    run('git','add','--all','--',*sorted(actual))
    staged=set(run('git','diff','--cached','--name-only','-z').decode().rstrip('\0').split('\0'))
    require(staged==actual,'staged paths differ from verified worktree changes')
    require(not (ROOT/WORKFLOW).exists() and not (ROOT/BOOTSTRAP).exists(),'temporary migration tools were not removed')
    run('git','diff','--cached','--check')
    if not args.publish:
        print('Validated and staged complete conversion; --publish was absent, so no commit or push occurred')
        return
    require(remote_tip()==w,'feature branch advanced before commit')
    run('git','-c','user.name=github-actions[bot]','-c','user.email=41898282+github-actions[bot]@users.noreply.github.com','commit','-m','Store editable art and runtime models as normal binary files')
    commit=git('rev-parse','HEAD')
    require(git('show','-s','--format=%P',commit)==w,'conversion commit parent differs')
    require(remote_tip()==w,'feature branch advanced before push')
    run('git','push','origin',f'HEAD:refs/heads/{BRANCH}')
    require(remote_tip()==commit,'remote conversion commit verification failed')
    print(f'Published complete verified conversion: {commit}')

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action',choices=['prepare','finish'])
    parser.add_argument('--expected-m',required=True)
    parser.add_argument('--event',type=Path,required=True)
    parser.add_argument('--state',type=Path,required=True)
    parser.add_argument('--validation-stamp',type=Path)
    parser.add_argument('--publish',action='store_true')
    args=parser.parse_args()
    require(args.state.is_absolute() and not args.state.resolve().is_relative_to(ROOT),'receipt must be outside the tracked checkout')
    args.state.parent.mkdir(parents=True,exist_ok=True)
    if args.action=='finish':require(args.validation_stamp is not None,'finish requires the successful validation stamp')
    w=guard(args.expected_m,args.event)
    if args.action=='prepare':prepare(args,w)
    else:finish(args,w)

if __name__=='__main__':main()
