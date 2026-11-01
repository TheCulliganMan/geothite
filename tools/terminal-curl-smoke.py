#!/usr/bin/env python3
"""Exercise the published curl installer and Rust client outside the checkout."""

import argparse
import fcntl
import json
import os
from pathlib import Path
import pty
import re
import select
import signal
import struct
import subprocess
import tempfile
import termios
import time


def stop(process):
    if process.poll() is None:
        os.killpg(process.pid, signal.SIGTERM)
    try:
        process.wait(timeout=5)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        process.wait(timeout=5)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("url", nargs="?", default="https://geothite.ryanculligan.com")
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix="geothite-curl-smoke-") as temporary:
        root = Path(temporary)
        scratch = root / "scratch"
        scratch.mkdir()
        home = root / "installed"
        env = {
            **os.environ,
            "GEOTHITE_HOME": str(home),
            "GEOTHITE_DOWNLOAD_URL": args.url.rstrip("/"),
            "TMPDIR": str(scratch),
            "TERM": "xterm-256color",
        }
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))

        def attach_terminal():
            os.setsid()
            fcntl.ioctl(slave, termios.TIOCSCTTY, 0)

        game = subprocess.Popen(
            ["sh", "-c", 'curl -fsSL "$GEOTHITE_DOWNLOAD_URL/install.sh" | sh'],
            cwd=root, env=env, stdin=slave, stdout=slave, stderr=subprocess.PIPE,
            preexec_fn=attach_terminal,
        )
        os.close(slave)
        output = bytearray()

        def wait_for(text, offset=0, timeout=120):
            deadline = time.monotonic() + timeout
            while time.monotonic() < deadline:
                rendered = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", output[offset:].decode("utf-8", "replace"))
                if text in rendered:
                    return
                if game.poll() is not None:
                    break
                if select.select([master], [], [], 0.1)[0]:
                    try:
                        output.extend(os.read(master, 65536))
                    except OSError:
                        break
            error = game.stderr.read() if game.poll() is not None else b""
            raise AssertionError(f"Downloaded client did not show {text!r}: stderr={error!r}; output={output[-1200:]!r}")

        try:
            wait_for("PlayersHouse2F")
            assert output.startswith(b"\x1b[?1049h"), "Installer printed before fullscreen startup"
            offset = len(output)
            os.write(master, b"\r")
            wait_for("SAVE", offset, timeout=15)
            os.write(master, b"x")
            time.sleep(0.2)
            os.write(master, b"\x1b[15~")  # Production F5 save.
            deadline = time.monotonic() + 15
            while time.monotonic() < deadline and not list((home / "saves").glob("*.crystalsave")):
                time.sleep(0.1)
            saves = list((home / "saves").glob("*.crystalsave"))
            assert len(saves) == 1, "Downloaded client did not create its real save"
            os.write(master, b":q!\r")
            deadline = time.monotonic() + 10
            while game.poll() is None and time.monotonic() < deadline:
                if select.select([master], [], [], 0.1)[0]:
                    try:
                        os.read(master, 65536)
                    except OSError:
                        break
            game.wait(timeout=2)
            assert game.returncode == 0
            assert game.stderr.read() == b"", "Fullscreen client wrote to stderr"
        finally:
            stop(game)
            os.close(master)

        saved = saves[0].read_bytes()
        upgrade = subprocess.run(
            ["sh", "-c", 'curl -fsSL "$GEOTHITE_DOWNLOAD_URL/install.sh" | sh -s -- --no-play'],
            cwd=root, env=env, capture_output=True, check=True,
        )
        assert upgrade.stdout == upgrade.stderr == b"", "Reinstall should be silent"
        assert saves[0].read_bytes() == saved, "Reinstall changed the player save"

        mcp = subprocess.Popen(
            [str(home / "bin/geothite"), "mcp"], cwd=root, env=env,
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            text=True, start_new_session=True,
        )
        request_id = 0

        def call(name, arguments=None):
            nonlocal request_id
            request_id += 1
            request = {"jsonrpc": "2.0", "id": request_id, "method": "tools/call",
                       "params": {"name": name, "arguments": arguments or {}}}
            mcp.stdin.write(json.dumps(request) + "\n")
            mcp.stdin.flush()
            assert select.select([mcp.stdout], [], [], 30)[0], f"MCP timeout: {name}"
            response = json.loads(mcp.stdout.readline())
            assert response.get("id") == request_id and not response.get("error"), response
            result = response["result"]
            assert not result.get("isError"), result
            return result["structuredContent"]

        try:
            moved = call("move", {"direction": "right", "steps": 1, "run": True})
            assert moved["position"] == {"x": 4, "y": 3}, moved
            call("press", {"button": "start"})
            observed = call("observe", {"detail": "compact"})
            assert any("SAVE" in line["text"] for line in observed["menu"]), observed
            mcp.stdin.close()
            mcp.wait(timeout=10)
            assert mcp.returncode == 0 and mcp.stderr.read() == ""
        finally:
            stop(mcp)
        print("Curl install, fullscreen Start/save, save-preserving reinstall and authoritative MCP movement/Start passed.")


if __name__ == "__main__":
    main()
