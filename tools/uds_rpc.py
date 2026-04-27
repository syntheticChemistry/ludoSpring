#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# uds_rpc.py — socat replacement for JSON-RPC over Unix domain sockets
# Usage: echo '{"jsonrpc":"2.0",...}' | python3 uds_rpc.py /path/to/socket
import socket, sys, os

if len(sys.argv) < 2:
    print("Usage: echo JSON | python3 uds_rpc.py /path/to.sock", file=sys.stderr)
    sys.exit(1)

sock_path = sys.argv[1]
payload = sys.stdin.read().strip()
if not payload:
    sys.exit(0)

try:
    s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    s.settimeout(5)
    s.connect(sock_path)
    s.sendall((payload + "\n").encode())
    chunks = []
    while True:
        try:
            data = s.recv(8192)
            if not data:
                break
            chunks.append(data)
            if b"\n" in data:
                break
        except socket.timeout:
            break
    s.close()
    sys.stdout.write(b"".join(chunks).decode())
except Exception as e:
    print(f'{{"error":"{e}"}}', file=sys.stderr)
    sys.exit(1)
