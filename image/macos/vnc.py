# SPDX-License-Identifier: Apache-2.0
"""A small RFB 3.8 client: screenshots, keys, and clicks for the Setup Assistant."""

import socket
import struct
import subprocess
import time
import zlib

SHIFTED = '~!@#$%^&*()_+{}|:"<>?'
KEYS = {"return": 0xFF0D, "tab": 0xFF09, "escape": 0xFF1B, "space": 0x0020,
        "backspace": 0xFF08, "left": 0xFF51, "up": 0xFF52, "right": 0xFF53,
        "down": 0xFF54, "shift": 0xFFE1, "control": 0xFFE3, "option": 0xFFE9,
        "command": 0xFFE7, "f5": 0xFFC2}
RAW, DESKTOP_SIZE = 0, -223


class VncError(RuntimeError):
    pass


def vnc_response(password, challenge):
    """Encrypt the challenge with DES, as VNC authentication asks.

    The key is the password, padded to 8 bytes, with the bits of each byte in
    reverse order. The system openssl does the DES step.
    """
    key = password.encode("latin-1")[:8].ljust(8, b"\x00")
    key = bytes(int(f"{byte:08b}"[::-1], 2) for byte in key)
    result = subprocess.run(
        ["/usr/bin/openssl", "enc", "-des-ecb", "-nopad", "-K", key.hex()],
        input=challenge, capture_output=True, check=True)
    return result.stdout


class Vnc:
    def __init__(self, host, port, password=None, timeout=30):
        self.sock = socket.create_connection((host, port), timeout=timeout)
        version = self.read(12)
        if not version.startswith(b"RFB 003."):
            raise VncError(f"not an RFB server: {version!r}")
        self.sock.sendall(b"RFB 003.008\n")
        count = self.read(1)[0]
        if count == 0:
            raise VncError(self.read(struct.unpack(">I", self.read(4))[0]).decode())
        types = self.read(count)
        if password and 2 in types:
            self.sock.sendall(b"\x02")
            self.sock.sendall(vnc_response(password, self.read(16)))
        elif not password and 1 in types:
            self.sock.sendall(b"\x01")
        else:
            raise VncError(f"no matching security type in {list(types)}")
        if struct.unpack(">I", self.read(4))[0] != 0:
            raise VncError("the server refused the connection")
        self.sock.sendall(b"\x01")
        self.width, self.height = struct.unpack(">HH", self.read(4))
        self.read(16)
        self.read(struct.unpack(">I", self.read(4))[0])
        pixel_format = struct.pack(">BBBBHHHBBB3x", 32, 24, 0, 1, 255, 255, 255, 16, 8, 0)
        self.sock.sendall(b"\x00\x00\x00\x00" + pixel_format)
        encodings = [RAW, DESKTOP_SIZE]
        self.sock.sendall(struct.pack(">BxH", 2, len(encodings))
                          + b"".join(struct.pack(">i", value) for value in encodings))
        self.frame = bytearray(self.width * self.height * 4)

    def read(self, size):
        data = bytearray()
        while len(data) < size:
            chunk = self.sock.recv(size - len(data))
            if not chunk:
                raise VncError("the server closed the connection")
            data += chunk
        return bytes(data)

    def refresh(self):
        """Request a full frame and apply every rectangle of the answer."""
        self.sock.sendall(struct.pack(">BBHHHH", 3, 0, 0, 0, self.width, self.height))
        while True:
            kind = self.read(1)[0]
            if kind == 0:
                self.read(1)
                for _ in range(struct.unpack(">H", self.read(2))[0]):
                    self.rectangle()
                return
            if kind == 1:
                self.read(3)
                self.read(6 * struct.unpack(">H", self.read(2))[0])
            elif kind == 2:
                continue
            elif kind == 3:
                self.read(3)
                self.read(struct.unpack(">I", self.read(4))[0])
            else:
                raise VncError(f"unknown server message {kind}")

    def rectangle(self):
        x, y, width, height, encoding = struct.unpack(">HHHHi", self.read(12))
        if encoding == DESKTOP_SIZE:
            self.width, self.height = width, height
            self.frame = bytearray(width * height * 4)
            return
        if encoding != RAW:
            raise VncError(f"unexpected encoding {encoding}")
        data = self.read(width * height * 4)
        for row in range(height):
            start = ((y + row) * self.width + x) * 4
            self.frame[start:start + width * 4] = data[row * width * 4:(row + 1) * width * 4]

    def png(self, path):
        """Write the current frame as an RGB PNG file."""
        rows = bytearray()
        rgb = bytearray(self.width * 3)
        for row in range(self.height):
            line = self.frame[row * self.width * 4:(row + 1) * self.width * 4]
            rgb[0::3], rgb[1::3], rgb[2::3] = line[2::4], line[1::4], line[0::4]
            rows += b"\x00" + rgb

        def chunk(kind, body):
            return (struct.pack(">I", len(body)) + kind + body
                    + struct.pack(">I", zlib.crc32(kind + body) & 0xFFFFFFFF))
        header = struct.pack(">IIBBBBB", self.width, self.height, 8, 2, 0, 0, 0)
        with open(path, "wb") as output:
            output.write(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header)
                         + chunk(b"IDAT", zlib.compress(bytes(rows), 6)) + chunk(b"IEND", b""))

    def key(self, keysym, down):
        self.sock.sendall(struct.pack(">BBxxI", 4, int(down), keysym))

    def press(self, name, modifiers=()):
        for modifier in modifiers:
            self.key(KEYS[modifier], True)
        keysym = KEYS[name] if name in KEYS else ord(name)
        self.key(keysym, True)
        self.key(keysym, False)
        for modifier in reversed(modifiers):
            self.key(KEYS[modifier], False)
        time.sleep(0.05)

    def type(self, text):
        for character in text:
            if character == "\n":
                self.press("return")
            elif character.isupper() or character in SHIFTED:
                self.press(character, ("shift",))
            else:
                self.press(character)

    def click(self, x, y):
        x, y = int(x), int(y)
        self.sock.sendall(struct.pack(">BBHH", 5, 0, x, y))
        time.sleep(0.1)
        self.sock.sendall(struct.pack(">BBHH", 5, 1, x, y))
        time.sleep(0.1)
        self.sock.sendall(struct.pack(">BBHH", 5, 0, x, y))
        time.sleep(0.2)

    def close(self):
        self.sock.close()
