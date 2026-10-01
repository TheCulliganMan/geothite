#!/usr/bin/env python3
"""Temporary pinned human GLB conversion; removed before the verified push.

prepare replays reviewed UTF-8 source edits, generates the exact reviewed GLB,
and removes explicitly retired assets. finish requires successful validation,
removes this bootstrap and its workflow, checks the full Git tree, and may push.
No authentication material or repository policy is read or changed here.
"""
from pathlib import Path
import argparse
import hashlib
import json
import os
import re
import subprocess
import sys

REPOSITORY = 'TheCulliganMan/geothite'
BRANCH = 'feat/connected-johto-3d'
PR = 5
WORKFLOW = '.github/workflows/materialize-human-glb.yml'
BOOTSTRAP = 'tools/human-glb-migration'
BOOTSTRAP_FILES = [f'{BOOTSTRAP}/{name}' for name in ('run.py', 'plan.json', 'integration.patch', 'README.md')]
PLAN_SHA256 = 'f92d52637bf852d5f5ed211f512171faf252f6fd654dae35daddce938e5ac974'
ROOT = Path(__file__).resolve().parents[2]


def require(condition, message):
    if not condition:
        raise ValueError(message)


def run(*arguments, capture=True, env=None):
    return subprocess.run(arguments, cwd=ROOT, check=True, env=env,
                          stdout=subprocess.PIPE if capture else None,
                          stderr=subprocess.PIPE if capture else None).stdout


def git(*arguments):
    return run('git', *arguments).decode().strip()


def sha(data):
    return hashlib.sha256(data).hexdigest()


def paths_between(before, after):
    return set(run('git', 'diff', '--no-renames', '--name-only', '-z', before, after).decode().split('\0')) - {''}


def changed_paths():
    result = set()
    for entry in run('git', 'status', '--porcelain=v1', '-z', '--untracked-files=all').decode().split('\0'):
        if not entry:
            continue
        require(len(entry) > 3 and entry[2] == ' ', 'malformed worktree status')
        require(not any(c in entry[:2] for c in 'RCU'), 'renames, copies or conflicts are not permitted')
        result.add(entry[3:])
    return result


def remote_tip():
    rows = git('ls-remote', '--exit-code', 'origin', f'refs/heads/{BRANCH}').splitlines()
    require(len(rows) == 1, 'remote branch is absent or ambiguous')
    tip, ref = rows[0].split('\t')
    require(ref == f'refs/heads/{BRANCH}', 'unexpected remote reference')
    return tip


def checked_file(record):
    path = ROOT / record['path']
    require(path.is_file() and not path.is_symlink(), f"missing or symbolic file: {record['path']}")
    data = path.read_bytes()
    require(len(data) == record['bytes'] and sha(data) == record['sha256'], f"file identity differs: {record['path']}")
    require(hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest() == record['git_blob_sha'], 'Git blob identity differs')
    return data


def load_plan():
    raw = (ROOT / BOOTSTRAP / 'plan.json').read_bytes()
    require(sha(raw) == PLAN_SHA256, 'reviewed conversion plan differs')
    plan = json.loads(raw)
    require(plan['repository'] == REPOSITORY and plan['branch'] == BRANCH and plan['pr'] == PR, 'plan target differs')
    for name in [*plan['changed_paths'], *(r['path'] for r in plan['retired']), plan['catalog']['path']]:
        path = Path(name)
        require(not path.is_absolute() and '..' not in path.parts, 'unsafe plan path')
    require(sha((ROOT / BOOTSTRAP / 'integration.patch').read_bytes()) == plan['patch_sha256'], 'reviewed source patch differs')
    return plan


def guard(args, plan):
    require(re.fullmatch('[0-9a-f]{40}', args.expected_m) is not None, 'bootstrap SHA must be exact')
    event = json.loads(args.event.read_text())
    pr = event['pull_request']
    require(event['action'] == 'synchronize' and event['number'] == PR, 'unexpected PR event')
    require(event['repository']['full_name'] == REPOSITORY, 'unexpected event repository')
    require(pr['head']['repo']['full_name'] == REPOSITORY and pr['base']['repo']['full_name'] == REPOSITORY, 'fork or different repository')
    require(pr['head']['ref'] == BRANCH and pr['base']['ref'] == 'main', 'unexpected PR branch')
    require(pr['state'] == 'open' and pr['draft'] is True, 'PR must remain open and draft')
    w = pr['head']['sha']
    require(re.fullmatch('[0-9a-f]{40}', w) is not None, 'invalid workflow SHA')
    require(git('rev-parse', 'HEAD') == w, 'checkout differs from event head')
    require(git('show', '-s', '--format=%P', w).split() == [args.expected_m], 'workflow must be a single-parent child of reviewed bootstrap')
    require(paths_between(args.expected_m, w) == {WORKFLOW}, 'workflow commit changed other paths')
    require(git('diff', '--diff-filter=A', '--name-only', args.expected_m, w) == WORKFLOW, 'workflow must be newly added')
    require(git('show', '-s', '--format=%P', args.expected_m).split() == [plan['base_commit']], 'bootstrap parent differs from frozen remote base')
    require(git('rev-parse', plan['base_commit'] + '^{tree}') == plan['base_tree'], 'base tree differs')
    require(paths_between(plan['base_commit'], args.expected_m) == set(BOOTSTRAP_FILES), 'bootstrap changed existing game files')
    require(set(git('diff', '--diff-filter=A', '--name-only', plan['base_commit'], args.expected_m).splitlines()) == set(BOOTSTRAP_FILES), 'bootstrap must only add its temporary files')
    require(remote_tip() == w, 'feature branch advanced; refusing stale conversion')
    return w


def verify_intended_tree(plan, state_directory):
    # A separate temporary index gives a full-tree comparison before tests,
    # without disturbing the checkout index or requiring a commit.
    index = state_directory / 'verification.index'
    if index.exists():
        index.unlink()
    env = dict(os.environ, GIT_INDEX_FILE=str(index))
    run('git', 'read-tree', 'HEAD', env=env)
    run('git', 'add', '--all', '--', *plan['changed_paths'], env=env)
    run('git', 'update-index', '--force-remove', '--', WORKFLOW, *BOOTSTRAP_FILES, env=env)
    tree = run('git', 'write-tree', env=env).decode().strip()
    require(tree == plan['final_tree'], f'prepared full tree differs: {tree}')
    index.unlink()
    return tree


def prepare(args, plan, w):
    require(not changed_paths(), 'conversion checkout must start clean')
    for item in plan['retired']:
        checked_file(item)
    patch = ROOT / BOOTSTRAP / 'integration.patch'
    patch.read_text(encoding='utf-8')
    patch_paths = set()
    for row in run('git', 'apply', '--numstat', '-z', str(patch)).decode().split('\0'):
        if row:
            added, deleted, path = row.split('\t', 2)
            require(added != '-' and deleted != '-', 'source patch must be UTF-8 text only')
            patch_paths.add(path)
    require(patch_paths == set(plan['source_paths']), 'source patch paths differ')
    run('git', 'apply', '--check', str(patch))
    run('git', 'apply', str(patch))
    sys.path.insert(0, str(ROOT / 'tools'))
    from animated_glb import export_directory, read_catalog
    from johto_character_geometry import f32_contract_digest, read_models
    human_directory = ROOT / plan['human_directory']
    before = read_models(human_directory)
    require(len(before) == 75, 'expected exactly 75 original human rigs')
    destination = ROOT / plan['catalog']['path']
    export_directory(human_directory, destination)
    checked_file(plan['catalog'])
    after = read_catalog(destination)
    identities_before = {model['name']: f32_contract_digest(model) for model in before.values()}
    identities_after = {name: f32_contract_digest(model) for name, model in after.items()}
    require(identities_before == identities_after, 'human geometry, material, joint, bind or ordering changed')
    require(len(identities_after) == 75, 'generated catalog must contain exactly 75 human rigs')
    for item in plan['retired']:
        checked_file(item)
        (ROOT / item['path']).unlink()
    require(changed_paths() == set(plan['changed_paths']), 'conversion changed unexpected paths')
    tree = verify_intended_tree(plan, args.state.parent)
    state = {'m': args.expected_m, 'w': w, 'plan_sha256': PLAN_SHA256,
             'tree': tree, 'human_f32_identities': identities_after}
    args.state.write_text(json.dumps(state, indent=2) + '\n')
    print(f'Prepared {tree}: 75 exact human rigs, 150 authored clips; retired {len(plan["retired"])} old files')


def finish(args, plan, w):
    state = json.loads(args.state.read_text())
    require(state['m'] == args.expected_m and state['w'] == w and state['plan_sha256'] == PLAN_SHA256, 'preparation receipt differs')
    require(state['tree'] == plan['final_tree'], 'preparation tree differs')
    require(args.validation_stamp.read_text().strip() == w, 'successful validation stamp missing')
    checked_file(plan['catalog'])
    require(changed_paths() == set(plan['changed_paths']), 'tests or generation changed unexpected paths')
    verify_intended_tree(plan, args.state.parent)
    for path in [WORKFLOW, *BOOTSTRAP_FILES]:
        (ROOT / path).unlink()
    (ROOT / BOOTSTRAP).rmdir()
    actual = changed_paths()
    require(actual == set(plan['changed_paths']) | set(BOOTSTRAP_FILES) | {WORKFLOW}, 'cleanup changed unexpected paths')
    run('git', 'add', '--all', '--', *sorted(actual))
    staged = set(run('git', 'diff', '--cached', '--name-only', '-z').decode().split('\0')) - {''}
    require(staged == actual, 'staged paths differ')
    require(git('write-tree') == plan['final_tree'], 'final full tree differs')
    run('git', 'diff', '--cached', '--check')
    require(not (ROOT / WORKFLOW).exists() and not (ROOT / BOOTSTRAP).exists(), 'temporary tools remain')
    if not args.publish:
        print('Verified and staged exact conversion; no commit or push requested')
        return
    require(os.environ.get('GITHUB_ACTIONS') == 'true' and os.environ.get('GITHUB_REPOSITORY') == REPOSITORY and os.environ.get('GITHUB_EVENT_NAME') == 'pull_request', 'publication requires the scoped GitHub Actions PR job')
    require(git('remote', 'get-url', 'origin') in (f'https://github.com/{REPOSITORY}', f'https://github.com/{REPOSITORY}.git'), 'unexpected publication origin')
    require(remote_tip() == w, 'branch advanced before commit')
    run('git', '-c', 'user.name=github-actions[bot]', '-c', 'user.email=41898282+github-actions[bot]@users.noreply.github.com', 'commit', '-m', 'Adopt shared GLB human rigs and authored locomotion clips')
    commit = git('rev-parse', 'HEAD')
    require(git('show', '-s', '--format=%P', commit) == w and git('rev-parse', commit + '^{tree}') == plan['final_tree'], 'conversion commit identity differs')
    require(remote_tip() == w, 'branch advanced before push')
    run('git', 'push', 'origin', f'HEAD:refs/heads/{BRANCH}')
    require(remote_tip() == commit, 'published commit verification failed')
    print(f'Published exact verified human GLB conversion: {commit}')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['prepare', 'finish'])
    parser.add_argument('--expected-m', required=True)
    parser.add_argument('--event', type=Path, required=True)
    parser.add_argument('--state', type=Path, required=True)
    parser.add_argument('--validation-stamp', type=Path)
    parser.add_argument('--publish', action='store_true')
    args = parser.parse_args()
    require(args.state.is_absolute() and not args.state.resolve().is_relative_to(ROOT), 'receipt must be outside checkout')
    require(not args.publish or args.action == 'finish', '--publish is only valid for finish')
    args.state.parent.mkdir(parents=True, exist_ok=True)
    if args.action == 'finish':
        require(args.validation_stamp is not None, 'finish requires validation stamp')
    plan = load_plan()
    w = guard(args, plan)
    (prepare if args.action == 'prepare' else finish)(args, plan, w)


if __name__ == '__main__':
    main()
