#!/usr/bin/env bash
# Publish a Rust-exported static guide alongside the existing Geothite service.
set -euo pipefail
if [[ $# != 3 ]]; then
  echo 'Usage: tools/deploy-pokedex.sh SSH_HOST RELEASE_DIRECTORY RELEASE_ID' >&2
  exit 2
fi
pokedex_host=$1
pokedex_release=$2
pokedex_id=$3
[[ "$pokedex_id" =~ ^[a-zA-Z0-9_-]+$ ]] || { echo 'Invalid release id' >&2; exit 2; }
[[ -f "$pokedex_release/index.html" && -f "$pokedex_release/pokedex.json" ]] || { echo 'Missing exported guide' >&2; exit 2; }
ssh -o BatchMode=yes "$pokedex_host" "mkdir -p /mnt/homelab/config/geothite/pokedex/releases/$pokedex_id"
rsync -a --checksum "$pokedex_release/" "$pokedex_host:/mnt/homelab/config/geothite/pokedex/releases/$pokedex_id/"
ssh -o BatchMode=yes "$pokedex_host" python3 - "$pokedex_id" <<'PY'
import fcntl, hashlib, json, os, pathlib, shutil, subprocess, sys, time, urllib.request
import yaml
release_id = sys.argv[1]
root = pathlib.Path('/mnt/homelab')
app = root / 'config/geothite/pokedex'
compose = root / 'docker-compose.yml'
mount = '/mnt/homelab/config/geothite/pokedex:/srv/crystal/web/pokedex:ro'

def run(*args):
    return subprocess.check_output(args, cwd=root, stderr=subprocess.PIPE)

def inspect():
    value = json.loads(run('docker', 'inspect', 'homelab-geothite-1'))[0]
    # Compare secrets in memory; never log environment or raw container config.
    return {'image': value['Image'], 'environment': sorted(value['Config']['Env']),
            'ports': value['HostConfig']['PortBindings'], 'networks': sorted(value['NetworkSettings']['Networks']),
            'mounts': sorted((m['Source'], m['Destination'], m['RW']) for m in value['Mounts'] if m['Destination'] != '/srv/crystal/web/pokedex')}

def swap(name, target):
    temporary = app / ('.' + name + '.next')
    temporary.unlink(missing_ok=True)
    temporary.symlink_to(target)
    os.replace(temporary, app / name)

with open(root / '.docker-update.lock', 'a') as lock:
    fcntl.flock(lock, fcntl.LOCK_EX)
    before = inspect()
    container_snapshot = run('docker', 'inspect', 'homelab-geothite-1')
    original = compose.read_text()
    configuration = yaml.safe_load(original)
    service = configuration['services']['geothite']
    old_target = os.readlink(app / 'current') if (app / 'current').is_symlink() else None
    backup_root = root / 'config/geothite/pokedex-backups'
    backup_root.mkdir(mode=0o700, exist_ok=True)
    backup = backup_root / (release_id + '-' + str(int(time.time())))
    backup.mkdir(parents=True, mode=0o700)
    shutil.copyfile(compose, backup / 'docker-compose.yml')
    os.chmod(backup / 'docker-compose.yml', 0o600)
    private_snapshot = backup / 'container-private.json'
    private_snapshot.write_bytes(container_snapshot)
    os.chmod(private_snapshot, 0o600)
    (backup / 'previous-release.txt').write_text(old_target or '')
    need_mount = mount not in service.get('volumes', [])
    if need_mount:
        lines = original.splitlines(keepends=True)
        start = next(i for i, line in enumerate(lines) if line.rstrip() == '  geothite:')
        end = next((i for i in range(start+1, len(lines)) if lines[i].startswith('  ') and not lines[i].startswith('   ') and lines[i].strip()), len(lines))
        volume_line = next(i for i in range(start, end) if lines[i].rstrip() == '    volumes:')
        lines.insert(volume_line+1, '      - ' + mount + '\n')
        changed = ''.join(lines)
        expected = json.loads(json.dumps(configuration))
        expected['services']['geothite']['volumes'].insert(0, mount)
        assert yaml.safe_load(changed) == expected, 'Unexpected Compose change'
        temporary = compose.with_name('.docker-compose.pokedex.next')
        temporary.write_text(changed)
        shutil.copymode(compose, temporary)
        os.replace(temporary, compose)
    try:
        swap('current', 'releases/' + release_id)
        swap('index.html', 'current/index.html')
        swap('species', 'current/species')
        for page in ('moves.html', 'state-fair.html'):
            if (app / 'releases' / release_id / page).exists():
                swap(page, 'current/' + page)
        if need_mount:
            run('docker', 'compose', 'up', '-d', '--no-deps', '--no-build', '--pull', 'never', 'geothite')
        for attempt in range(30):
            try:
                with urllib.request.urlopen('http://100.101.29.133:3003/healthz', timeout=3) as response:
                    assert response.status == 200
                break
            except Exception:
                if attempt == 29: raise
                time.sleep(1)
        after = inspect()
        assert after == before, 'Existing game configuration changed: ' + ', '.join(k for k in before if before[k] != after[k])
        served = urllib.request.urlopen('http://100.101.29.133:3003/pokedex/', timeout=10).read()
        expected_html = (app / 'releases' / release_id / 'index.html').read_bytes()
        assert served == expected_html, 'Release HTML differs from deployed route'
        with urllib.request.urlopen('http://100.101.29.133:3003/', timeout=10) as response:
            assert response.status == 200
    except Exception:
        shutil.copyfile(backup / 'docker-compose.yml', compose)
        if old_target: swap('current', old_target)
        else: (app / 'current').unlink(missing_ok=True)
        if need_mount:
            run('docker', 'compose', 'up', '-d', '--no-deps', '--no-build', '--pull', 'never', 'geothite')
        raise
    print('Published', release_id)
    print('Verified existing image, environment, ports, networks and original mounts unchanged.')
    print('Verified game root, health and exact Pokédex HTML. Private rollback backup:', backup)
PY
