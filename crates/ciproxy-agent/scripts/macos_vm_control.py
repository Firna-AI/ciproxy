#!/usr/bin/env python3
"""Send one control request to a macOS VM helper, as the agent does.

Usage: macos_vm_control.py SOCKET state|stop|save|wait [SECONDS]|connect PORT

For connect, the helper passes the guest vsock stream fd by SCM_RIGHTS.
The script then reads one guest line and prints only its message type,
so no runner registration or other guest data reaches the output.
"""

import json
import socket
import sys

VERSION = "CIPROXY-VM 1"


def request(path, line, want_fd=False, timeout=None):
    sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    sock.settimeout(timeout)
    sock.connect(path)
    sock.sendall(f"{VERSION} {line}\n".encode())
    data, fds = b"", []
    while not data.endswith(b"\n"):
        chunk, received, _, _ = socket.recv_fds(sock, 128, 1)
        if not chunk:
            break
        data += chunk
        fds.extend(received)
    sock.close()
    if fds and not want_fd:
        raise SystemExit("unexpected fd")
    return data.decode().strip(), fds


def main():
    path, command = sys.argv[1], sys.argv[2]
    if command == "connect":
        answer, fds = request(path, f"CONNECT {sys.argv[3]}", True, 30)
        print(answer)
        if answer == "OK":
            if len(fds) != 1:
                raise SystemExit("OK without exactly one fd")
            guest = socket.socket(fileno=fds[0])
            guest.settimeout(30)
            line = b""
            while not line.endswith(b"\n"):
                chunk = guest.recv(4096)
                if not chunk:
                    break
                line += chunk
            print("guest message type:", json.loads(line).get("type", "unknown"))
        return
    timeout = 300
    if command == "wait":
        timeout = float(sys.argv[3]) if len(sys.argv) > 3 else None
    answer, _ = request(path, command.upper(), timeout=timeout)
    print(answer)


if __name__ == "__main__":
    main()
