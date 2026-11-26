"""
scratch_board.py — The 1D Scratch-and-Catch Board.

The shared event bus that connects the game triad:
  - Kenshi (ground) writes combat, faction, selection events
  - Satisfactory (board) writes factory, conveyor, production events
  - Space Engineers / KSP (outer) writes physics, orbital, particle events

All three games read from and write to ONE bus:
  %LOCALAPPDATA%/Gaia/scratch-board.jsonl

The board is a 1D interface — a flat sequence of events that any game
can hook into. Factory logic compiles into RPG behavior. Physics
refines the world. Every event from any game feeds the others.

Usage:
    python scratch_board.py status     # show board state
    python scratch_board.py tail       # tail the board (live)
    python scratch_board.py stats      # event statistics
    python scratch_board.py bridge     # run the bridge (connect all games)
"""

import sys, os, json, time, socket, threading
from datetime import datetime
from collections import Counter
from pathlib import Path

BOARD_PATH = os.path.join(os.environ.get("LOCALAPPDATA", ""), "Gaia", "scratch-board.jsonl")
GAIA_DIR = os.path.join(os.environ.get("LOCALAPPDATA", ""), "Gaia")

# Kenshi nerve port
KENSHI_PORT = 7741
# Satisfactory nerve port (future)
SATISFACTORY_PORT = 7742
# Space/KSP nerve port (future)
OUTER_PORT = 7743
# Board bus port
BOARD_PORT = 7740


def read_board():
    """Read all events from the scratch board."""
    if not os.path.exists(BOARD_PATH):
        return []
    events = []
    with open(BOARD_PATH, "r") as f:
        for line in f:
            line = line.strip()
            if line:
                try:
                    events.append(json.loads(line))
                except:
                    pass
    return events


def write_event(source, event_type, data):
    """Write an event to the scratch board."""
    entry = {
        "ts": datetime.now().isoformat(),
        "source": source,
        "type": event_type,
        "data": data,
    }
    os.makedirs(os.path.dirname(BOARD_PATH), exist_ok=True)
    with open(BOARD_PATH, "a") as f:
        f.write(json.dumps(entry) + "\n")
    return entry


def status():
    """Show board status."""
    events = read_board()
    print(f"\n{'='*60}")
    print(f"  SCRATCH BOARD — 1D Game Triad Bus")
    print(f"{'='*60}")
    print(f"  Path: {BOARD_PATH}")
    print(f"  Events: {len(events)}")

    if events:
        sources = Counter(e.get("source", "?") for e in events)
        types = Counter(e.get("type", "?") for e in events)
        print(f"\n  SOURCES:")
        for src, count in sources.most_common():
            print(f"    {count:5d}  {src}")
        print(f"\n  EVENT TYPES:")
        for t, count in types.most_common():
            print(f"    {count:5d}  {t}")
        print(f"\n  LATEST 5:")
        for e in events[-5:]:
            ts = e.get("ts", "?")[:19]
            src = e.get("source", "?")
            typ = e.get("type", "?")
            print(f"    {ts}  [{src}] {typ}")

    # Check nerve connections
    print(f"\n  NERVE STATUS:")
    for name, port in [("kenshi", KENSHI_PORT), ("satisfactory", SATISFACTORY_PORT), ("outer", OUTER_PORT)]:
        try:
            sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            sock.settimeout(0.5)
            result = sock.connect_ex(("127.0.0.1", port))
            sock.close()
            status = "LIVE" if result == 0 else "DOWN"
        except:
            status = "DOWN"
        print(f"    {name:15s}  :{port}  {status}")


def tail():
    """Tail the board (live updates)."""
    print(f"[board] tailing {BOARD_PATH} (Ctrl+C to stop)")
    last_size = 0
    if os.path.exists(BOARD_PATH):
        last_size = os.path.getsize(BOARD_PATH)
        # Print last 5 events
        events = read_board()
        for e in events[-5:]:
            ts = e.get("ts", "?")[:19]
            src = e.get("source", "?")
            typ = e.get("type", "?")
            print(f"  {ts}  [{src}] {typ}")

    while True:
        if os.path.exists(BOARD_PATH):
            size = os.path.getsize(BOARD_PATH)
            if size > last_size:
                with open(BOARD_PATH, "r") as f:
                    f.seek(last_size)
                    new_data = f.read()
                last_size = size
                for line in new_data.strip().split("\n"):
                    if line:
                        try:
                            e = json.loads(line)
                            ts = e.get("ts", "?")[:19]
                            src = e.get("source", "?")
                            typ = e.get("type", "?")
                            print(f"  {ts}  [{src}] {typ}")
                        except:
                            print(f"  {line[:80]}")
        time.sleep(1)


def stats():
    """Show event statistics."""
    events = read_board()
    if not events:
        print("[board] no events yet")
        return

    sources = Counter(e.get("source", "?") for e in events)
    types = Counter(e.get("type", "?") for e in events)

    print(f"\n  BOARD STATISTICS — {len(events)} events")
    print(f"\n  By source:")
    for src, count in sources.most_common():
        pct = round(100 * count / len(events), 1)
        bar = "█" * int(pct / 2)
        print(f"    {src:15s}  {count:5d}  {pct:5.1f}%  {bar}")

    print(f"\n  By type:")
    for t, count in types.most_common(20):
        print(f"    {t:25s}  {count:5d}")

    if len(events) > 1:
        first = events[0].get("ts", "")
        last = events[-1].get("ts", "")
        print(f"\n  Time range: {first[:19]} → {last[:19]}")


def bridge():
    """Run the bridge — connect all three game nerves to the board."""
    print(f"[bridge] Starting triad bridge on :{BOARD_PORT}")
    print(f"  Kenshi nerve:       :{KENSHI_PORT}")
    print(f"  Satisfactory nerve: :{SATISFACTORY_PORT}")
    print(f"  Outer nerve:        :{OUTER_PORT}")
    print(f"  Board bus:          :{BOARD_PORT}")

    def kenshi_listener():
        """Listen for Kenshi nerve events and write to board."""
        while True:
            try:
                sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
                sock.settimeout(2)
                sock.connect(("127.0.0.1", KENSHI_PORT))
                print("[bridge] Connected to Kenshi nerve!")
                buf = b""
                while True:
                    data = sock.recv(4096)
                    if not data:
                        break
                    buf += data
                    while b"\n" in buf:
                        line, buf = buf.split(b"\n", 1)
                        text = line.decode("utf-8", errors="replace").strip()
                        if text:
                            parts = text.split("\t")
                            event_type = parts[0] if parts else "raw"
                            write_event("kenshi", event_type, {
                                "raw": text,
                                "parts": parts,
                            })
            except socket.timeout:
                pass
            except ConnectionRefusedError:
                pass
            except Exception as e:
                print(f"[bridge] kenshi error: {e}")
            time.sleep(5)

    # Start listeners in threads
    t1 = threading.Thread(target=kenshi_listener, daemon=True)
    t1.start()
    print("[bridge] Kenshi listener started")

    # Main loop — periodic state snapshots
    while True:
        write_event("bridge", "heartbeat", {
            "kenshi_nerve": check_port(KENSHI_PORT),
            "satisfactory_nerve": check_port(SATISFACTORY_PORT),
            "outer_nerve": check_port(OUTER_PORT),
        })
        time.sleep(60)


def check_port(port):
    """Check if a port is listening."""
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(0.5)
        result = sock.connect_ex(("127.0.0.1", port))
        sock.close()
        return "LIVE" if result == 0 else "DOWN"
    except:
        return "DOWN"


if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "status"

    if cmd == "tail":
        tail()
    elif cmd == "stats":
        stats()
    elif cmd == "bridge":
        bridge()
    else:
        status()
