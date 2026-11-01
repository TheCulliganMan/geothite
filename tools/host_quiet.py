"""Optional macOS preconditions for timing captures on a shared workstation."""
from collections import deque
from datetime import datetime, timezone
from pathlib import Path
import os
import plistlib
import subprocess
import sys
import threading
import time


CONTENDERS = {'rustc', 'clippy-driver', 'xcodebuild', 'chrome-headless-shell',
              'render_at_location', 'render_saved_session', 'new_bark_3d', 'performance_audit',
              'baseline-v7', 'candidate-v7'}


def processes():
    output = subprocess.check_output(['ps','-axo','pid=,ppid=,pcpu=,comm='],text=True,timeout=3)
    result = []
    for line in output.splitlines():
        pid, parent, cpu, command = line.strip().split(None,3)
        result.append(dict(pid=int(pid),parent_pid=int(parent),
                           cpu_percent=float(cpu),name=Path(command).name))
    return result


def contenders(rows, own_pids=()):
    # Browser renderers and GPU helpers belong to the captured Node process.
    # Exclude its descendants without ignoring an unrelated browser or build.
    owned = set(own_pids)
    while True:
        descendants = {row['pid'] for row in rows if row.get('parent_pid') in owned}
        if descendants <= owned:
            break
        owned.update(descendants)
    return [row for row in rows if row['pid'] not in owned and (
        row['name'] in CONTENDERS or row['name'].startswith((
            'crystal_bevy-', 'crystal_voxel_view-', 'candidate-progressive-',
            'baseline-progressive-')))]


def gpu_utilization(tree):
    """Missing driver telemetry is unavailable, rather than an idle GPU."""
    if isinstance(tree,dict):
        statistics=tree.get('PerformanceStatistics')
        if isinstance(statistics,dict):
            value=statistics.get('Device Utilization %')
            if isinstance(value,(int,float)) and 0<=value<=100:
                return value
        values=tree.values()
    elif isinstance(tree,list):
        values=tree
    else:
        return None
    samples=[value for item in values if (value:=gpu_utilization(item)) is not None]
    return max(samples) if samples else None


def snapshot():
    if sys.platform!='darwin':
        raise ValueError('--quiet-host currently requires macOS AGX driver telemetry')
    rows=processes()
    tree=plistlib.loads(subprocess.check_output(['ioreg','-r','-c','AGXAccelerator','-a'],timeout=3))
    gpu=gpu_utilization(tree)
    if gpu is None:
        raise ValueError('GPU idle telemetry unavailable; cannot certify --quiet-host preconditions')
    return dict(at_utc=datetime.now(timezone.utc).isoformat(),gpu_percent=gpu,
                cpu_percent_estimate=sum(row['cpu_percent'] for row in rows)/(os.cpu_count() or 1),
                contenders=contenders(rows))


class CaptureHost:
    def __init__(self, enabled=False, stable_seconds=10, max_gpu_percent=15, max_cpu_percent=30):
        self.policy=dict(stable_seconds=stable_seconds,max_gpu_percent=max_gpu_percent,
                         max_cpu_percent_estimate=max_cpu_percent) if enabled else None
        self.before=[]
        self.events=[]
        self.own_pids={os.getpid()}
        self.done=threading.Event()
        self.thread=None

    def own_pid(self,pid):
        self.own_pids.add(pid)
        # Register the real child before sampling, avoiding a launch-time race.
        if self.policy is not None and self.thread is None:
            self.thread=threading.Thread(target=self._watch,daemon=True)
            self.thread.start()

    def wait(self):
        if self.policy is None:
            return
        recent=deque(maxlen=20)
        stable_since=None
        announced=None
        while True:
            sample=snapshot();recent.append(sample)
            quiet=(not sample['contenders'] and sample['gpu_percent']<=self.policy['max_gpu_percent']
                   and sample['cpu_percent_estimate']<=self.policy['max_cpu_percent_estimate'])
            if quiet:
                if stable_since is None:
                    stable_since=time.monotonic()
                if time.monotonic()-stable_since>=self.policy['stable_seconds']:
                    self.before=list(recent)
                    return
            else:
                stable_since=None
            state=(quiet,tuple(sorted({row['name'] for row in sample['contenders']})))
            if state!=announced:
                print(f"Host precondition: GPU {sample['gpu_percent']}%, estimated CPU {sample['cpu_percent_estimate']:.1f}%, "
                      f"other jobs {', '.join(state[1]) or 'none'}; {'settling' if quiet else 'waiting'}",flush=True)
                announced=state
            time.sleep(1)

    def _watch(self):
        seen=set()
        while not self.done.wait(.5):
            try:
                for row in contenders(processes(),self.own_pids):
                    if row['pid'] not in seen:
                        self.events.append(dict(at_utc=datetime.now(timezone.utc).isoformat(),**row))
                        seen.add(row['pid'])
            except (OSError,ValueError,subprocess.SubprocessError) as error:
                self.events.append(dict(error=str(error)))
                return

    def __enter__(self):
        return self

    def __exit__(self,*unused):
        self.done.set()
        if self.thread:
            self.thread.join(timeout=5)

    def report(self):
        return dict(policy=self.policy,before=self.before,contenders_during_capture=self.events)
