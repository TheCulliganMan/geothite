#!/usr/bin/env python3
"""Serialize graphical QA across chats sharing this checkout (macOS/Linux)."""
import fcntl
import os
from pathlib import Path
import signal
import subprocess
import sys


def main():
    command = sys.argv[1:]
    if command[:1] == ["--"]:
        command = command[1:]
    if not command:
        sys.exit("usage: python3 tools/with-graphics-lock.py -- COMMAND [ARG ...]")
    root = Path(__file__).resolve().parent.parent
    directory = root / ".local"
    directory.mkdir(exist_ok=True)
    # Never unlink this inode: waiters must all lock the same file. Kernel locks
    # release on exit, so stale owner text cannot strand later test runs.
    with (directory / "graphics-qa.lock").open("a+") as lease:
        print("Waiting for graphics QA lease...", file=sys.stderr, flush=True)
        fcntl.flock(lease, fcntl.LOCK_EX)
        lease.seek(0)
        lease.truncate()
        lease.write(f"pid={os.getpid()} command={command!r}\n")
        lease.flush()
        print("Graphics QA lease acquired.", file=sys.stderr, flush=True)
        # The child retains the lease even if this wrapper is killed. It is
        # released when the graphical process itself exits.
        child = subprocess.Popen(command, pass_fds=(lease.fileno(),))
        previous = {}
        for signum in (signal.SIGINT, signal.SIGTERM):
            previous[signum] = signal.signal(
                signum, lambda received, _frame: child.send_signal(received)
            )
        try:
            status = child.wait()
        finally:
            for signum, handler in previous.items():
                signal.signal(signum, handler)
        return status if status >= 0 else 128 - status


if __name__ == "__main__":
    sys.exit(main())
