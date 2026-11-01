"""Attach Instruments after validated warmup, before any measured game frames."""
import ctypes
import json
import os
import select
import signal
import subprocess
import sys
import time
import uuid

from instruments_scratch import RecorderScratch


class TracingStarted:
    """Receive xctrace's documented Darwin start notification on a private FD."""
    def __init__(self):
        self.name = 'com.geothite.performance-audit.'+uuid.uuid4().hex
        self.library = ctypes.CDLL('/usr/lib/libSystem.B.dylib')
        pointer = ctypes.POINTER(ctypes.c_int)
        self.library.notify_register_file_descriptor.argtypes = [ctypes.c_char_p, pointer, ctypes.c_int, pointer]
        self.library.notify_register_file_descriptor.restype = ctypes.c_uint32
        self.library.notify_cancel.argtypes = [ctypes.c_int]
        self.library.notify_cancel.restype = ctypes.c_uint32
        self.fd, self.token = ctypes.c_int(-1), ctypes.c_int(-1)

    def __enter__(self):
        status = self.library.notify_register_file_descriptor(
            self.name.encode(), ctypes.byref(self.fd), 0, ctypes.byref(self.token))
        if status:
            raise ValueError(f'Cannot register native-profiler start notification: status {status}')
        return self

    def wait(self, timeout):
        if not select.select([self.fd.value], [], [], timeout)[0]:
            return False
        if len(os.read(self.fd.value, 4)) != 4:
            raise ValueError('Truncated native-profiler start notification')
        return True

    def __exit__(self, *unused):
        # notify_cancel closes the FD once its sole registration is released.
        self.library.notify_cancel(self.token.value)


def stop(process, graceful_signal):
    if process is None or process.poll() is not None:
        return
    process.send_signal(graceful_signal)
    try:
        process.wait(timeout=20)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait()


def wait_for_saved_trace(recorder, scratch, timeout):
    """Observe conversion intermediates after gameplay exits, until save ends."""
    deadline=time.monotonic()+timeout
    while recorder.poll() is None:
        scratch.observe(recorder.pid)
        remaining=deadline-time.monotonic()
        if remaining<=0:
            raise subprocess.TimeoutExpired(recorder.args,timeout)
        try:
            recorder.wait(timeout=min(.25,remaining))
        except subprocess.TimeoutExpired:
            pass


def record(command, env, directory, native_trace, client_log, recorder_log, log_path, timeout, own_pid=None,
           cpu_sampling=False):
    """Return the actual client exit status; profiler failures raise separately."""
    app = recorder = None
    scratch = RecorderScratch()
    deadline = time.monotonic()+timeout
    command = command+['--wait-for-metal-capture']
    try:
        with client_log.open('w') as output:
            app = subprocess.Popen(command, env=env, stdout=output, stderr=subprocess.STDOUT)
            if own_pid is not None:
                own_pid(app.pid)
            while not (directory/'native-ready').exists():
                if app.poll() is not None:
                    raise ValueError(f'Client exited {app.returncode} before native-profiler readiness')
                if time.monotonic() >= deadline:
                    raise subprocess.TimeoutExpired(command, timeout)
                time.sleep(.05)
            metadata = json.loads((directory/'metadata.json').read_text())
            if metadata['process_id'] != app.pid:
                raise ValueError('Native profiler PID differs from the actual client process')
            with TracingStarted() as notification:
                executable=subprocess.check_output(['xcrun','--find','xctrace'],text=True,timeout=10).strip()
                native = [executable, 'record', '--instrument', 'Metal Application', '--instrument', 'GPU']
                if cpu_sampling:
                    native += ['--instrument', 'Time Profiler']
                native += ['--output', str(native_trace), '--no-prompt', '--time-limit', f'{timeout}s',
                          '--notify-tracing-started', notification.name, '--attach', str(app.pid)]
                recorder = subprocess.Popen(native, stdout=recorder_log, stderr=subprocess.STDOUT)
                started_deadline = min(deadline, time.monotonic()+45)
                while not notification.wait(.1):
                    if recorder.poll() is not None:
                        raise ValueError(f'Native profiler exited {recorder.returncode} before recording started')
                    if app.poll() is not None:
                        raise ValueError(f'Client exited {app.returncode} while attaching native profiler')
                    if time.monotonic() >= started_deadline:
                        raise ValueError('Native profiler did not signal recording start within its deadline')
                (directory/'native-start').write_text('recording started')
            app.wait(timeout=max(.01, deadline-time.monotonic()))
            if recorder.poll() is None:
                scratch.observe(recorder.pid)
            # Attach recordings normally stop when their target exits. If an
            # Instruments version keeps recording, request its ordinary stop.
            stopping_deadline = time.monotonic()+3
            while recorder.poll() is None and time.monotonic() < stopping_deadline:
                if any(marker in log_path.read_text() for marker in (
                        'Target app exited', 'Recording completed', 'Saving output file')):
                    break
                time.sleep(.1)
            else:
                if recorder.poll() is None:
                    recorder.send_signal(signal.SIGINT)
            wait_for_saved_trace(recorder,scratch,timeout+120)
            if recorder.returncode:
                raise ValueError(f'Native profiler exited {recorder.returncode}; see {log_path.name}')
            return subprocess.CompletedProcess(command, app.returncode)
    finally:
        if recorder is not None and recorder.poll() is None:
            scratch.observe(recorder.pid)
        stop(app, signal.SIGTERM)
        stop(recorder, signal.SIGINT)
        report=scratch.retire(recorder is None or recorder.poll() is not None)
        report['recorder_pid']=recorder.pid if recorder is not None else None
        (directory/'native-scratch.json').write_text(json.dumps(report,indent=2)+'\n')
        if report['errors'] and sys.exc_info()[0] is None:
            raise ValueError('Native recorder scratch cleanup failed; see native-scratch.json')
