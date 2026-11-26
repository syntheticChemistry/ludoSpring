"""
kenshi_board_bridge.py — Bridge Kenshi nerve events to the scratch board.

Connects to gaia-nerve.dll on :7741, reads Kenshi events (select, hit, ko, 
death, faction), and writes them to the scratch board. Also reads the
Kenshi save to write world state snapshots.

This is one arm of the triad bridge:
  Kenshi → scratch board → Satisfactory/Outer

Usage:
    python kenshi_board_bridge.py          # one-shot: read save, write to board
    python kenshi_board_bridge.py live     # live: connect to nerve + poll saves
"""

import sys, os, json, time, socket
from datetime import datetime
from pathlib import Path

BOARD_PATH = os.path.join(os.environ.get("LOCALAPPDATA", ""), "Gaia", "scratch-board.jsonl")
KENSHI_DIR = os.path.join(os.environ.get("LOCALAPPDATA", ""), "Gaia", "kenshi")
NERVE_PORT = 7741
INBOX = os.path.join(KENSHI_DIR, "inbox.txt")


def write_event(event_type, data):
    """Write a Kenshi event to the scratch board."""
    entry = {
        "ts": datetime.now().isoformat(),
        "source": "kenshi",
        "type": event_type,
        "data": data,
    }
    os.makedirs(os.path.dirname(BOARD_PATH), exist_ok=True)
    with open(BOARD_PATH, "a") as f:
        f.write(json.dumps(entry) + "\n")


def read_minds():
    """Read Kenshi NPC minds."""
    minds_path = os.path.join(KENSHI_DIR, "minds.json")
    if os.path.exists(minds_path):
        with open(minds_path, "r") as f:
            return json.load(f)
    return {}


def read_latest_save():
    """Find and summarize the latest Kenshi save."""
    save_dirs = [
        os.path.join(os.environ.get("LOCALAPPDATA", ""), "Kenshi", "save"),
        "M:\\SteamLibrary\\steamapps\\common\\Kenshi\\save",
    ]
    for sd in save_dirs:
        if os.path.exists(sd):
            saves = []
            for d in os.listdir(sd):
                fp = os.path.join(sd, d)
                if os.path.isdir(fp):
                    quick = os.path.join(fp, "quick.save")
                    plat = os.path.join(fp, "platoon.save")
                    mod_time = max(
                        os.path.getmtime(quick) if os.path.exists(quick) else 0,
                        os.path.getmtime(plat) if os.path.exists(plat) else 0,
                    )
                    if mod_time > 0:
                        saves.append((mod_time, d, fp))
            if saves:
                saves.sort(reverse=True)
                return saves[0]
    return None


def snapshot():
    """Take a snapshot of current Kenshi state and write to board."""
    minds = read_minds()
    save = read_latest_save()

    state = {
        "npcs": list(minds.keys()) if minds else [],
        "npc_count": len(minds),
        "latest_save": save[1] if save else None,
        "save_time": datetime.fromtimestamp(save[0]).isoformat() if save else None,
    }

    # Read trust levels
    if minds:
        for name, mind in minds.items():
            state[f"npc_{name}_trust"] = mind.get("trust", 0)
            state[f"npc_{name}_mood"] = mind.get("mood", "?")

    write_event("world_snapshot", state)
    print(f"[kenshi→board] snapshot: {len(minds)} NPCs, save: {save[1] if save else 'none'}")
    return state


def read_inbox():
    """Read Kenshi inbox (file-based transport) and forward to board."""
    if not os.path.exists(INBOX):
        return []
    events = []
    with open(INBOX, "r") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            parts = line.split("\t")
            event_type = parts[0] if parts else "raw"
            who = parts[1] if len(parts) > 1 else ""
            detail = parts[2] if len(parts) > 2 else ""
            ts = parts[-1] if len(parts) > 1 else ""
            events.append({
                "type": event_type,
                "who": who,
                "detail": detail,
                "game_ts": ts,
            })
    return events


def live_mode():
    """Live bridge: connect to nerve + poll inbox + periodic snapshots."""
    print(f"[kenshi→board] LIVE MODE — bridging Kenshi to scratch board")
    print(f"  Nerve port: {NERVE_PORT}")
    print(f"  Board path: {BOARD_PATH}")
    print(f"  Inbox: {INBOX}")

    last_snapshot = 0
    last_inbox_size = 0

    while True:
        now = time.time()

        # Periodic snapshot (every 60s)
        if now - last_snapshot > 60:
            try:
                snapshot()
            except Exception as e:
                print(f"[kenshi→board] snapshot error: {e}")
            last_snapshot = now

        # Check inbox for new events
        if os.path.exists(INBOX):
            size = os.path.getsize(INBOX)
            if size != last_inbox_size:
                events = read_inbox()
                for e in events:
                    write_event(e["type"], e)
                    print(f"  → {e['type']}: {e.get('who', '')} {e.get('detail', '')}")
                last_inbox_size = size

        # Try nerve connection (non-blocking check)
        try:
            sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            sock.settimeout(1)
            sock.connect(("127.0.0.1", NERVE_PORT))
            buf = b""
            while True:
                try:
                    data = sock.recv(4096)
                    if not data:
                        break
                    buf += data
                    while b"\n" in buf:
                        line, buf = buf.split(b"\n", 1)
                        text = line.decode("utf-8", errors="replace").strip()
                        if text:
                            parts = text.split("\t")
                            write_event(parts[0] if parts else "nerve", {
                                "raw": text,
                                "parts": parts,
                            })
                            print(f"  nerve→ {text[:80]}")
                except socket.timeout:
                    break
            sock.close()
        except (ConnectionRefusedError, socket.timeout):
            pass
        except Exception as e:
            print(f"[kenshi→board] nerve error: {e}")

        time.sleep(5)


if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "snapshot"

    if cmd == "live":
        live_mode()
    else:
        snapshot()
