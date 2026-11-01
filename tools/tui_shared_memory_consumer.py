"""Test-only Kitty receiver. Reads actual native RGB bytes; no game/art logic."""
import base64
import ctypes
import hashlib
import json
import mmap
import os
import struct
import zlib


class SharedMemoryConsumer:
    def __init__(self, report_fd=3):
        self.pending = b""
        self.report_fd = report_fd
        self.count = 0
        self.libc = ctypes.CDLL(None, use_errno=True)
        self.libc.shm_open.argtypes = [ctypes.c_char_p, ctypes.c_int, ctypes.c_uint]
        self.libc.shm_open.restype = ctypes.c_int
        self.libc.shm_unlink.argtypes = [ctypes.c_char_p]

    def feed(self, data):
        self.pending += data
        while True:
            start = self.pending.find(b"\x1b_G")
            if start < 0:
                self.pending = self.pending[-3:]
                return
            end = self.pending.find(b"\x1b\\", start)
            if end < 0:
                self.pending = self.pending[start:]
                return
            command = self.pending[start+3:end]
            self.pending = self.pending[end+2:]
            header, _, payload = command.partition(b";")
            fields = dict(part.decode().split("=", 1) for part in header.split(b",") if b"=" in part)
            if fields.get("t") != "s" or fields.get("a") != "T":
                continue
            assert fields["f"] == "24" and fields["q"] == "2" and fields["C"] == "1"
            name = base64.b64decode(payload)
            assert name.startswith(b"/geo-") and b"/" not in name[1:]
            fd = self.libc.shm_open(name, os.O_RDONLY, 0)
            assert fd >= 0, (name, ctypes.get_errno())
            width, height = int(fields["s"]), int(fields["v"])
            try:
                expected = width*height*3
                assert expected <= os.fstat(fd).st_size < expected+mmap.PAGESIZE, "RGB allocation (macOS rounds shm to pages)"
                with mmap.mmap(fd, width*height*3, flags=mmap.MAP_SHARED, prot=mmap.PROT_READ) as pixels:
                    rgb = pixels[:]
            finally:
                os.close(fd)
                assert self.libc.shm_unlink(name) == 0
            report = dict(fields=fields, width=width, height=height, hash=hashlib.sha256(rgb).hexdigest())
            if self.count == 0 or os.environ.get("TUI_CAPTURE_ANIMATION") == "1":
                # Encode a capture of those exact received pixels, not a render.
                def chunk(kind, body):
                    return struct.pack(">I", len(body))+kind+body+struct.pack(">I", zlib.crc32(kind+body))
                scanlines = b"".join(b"\0"+rgb[y*width*3:(y+1)*width*3] for y in range(height))
                png = b"\x89PNG\r\n\x1a\n"+chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0))+chunk(b"IDAT", zlib.compress(scanlines, 1))+chunk(b"IEND", b"")
                report["png"] = base64.b64encode(png).decode()
            os.write(self.report_fd, (json.dumps(report)+"\n").encode())
            self.count += 1
