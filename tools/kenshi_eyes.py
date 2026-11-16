#!/usr/bin/env python3
"""KenshiReborn: read Kenshi's autosaves, think as the world, narrate.

Reads only. Never writes under the Kenshi save folder. In watch mode it also
answers the GaiaMind KenshiLua mailbox (inbox.txt -> outbox.txt) so a selected
squad member speaks a line thought up by the tissue model.

    python kenshi_eyes.py once      # snapshot + one thought from the newest save
    python kenshi_eyes.py watch     # think on every new save, answer the mailbox
"""
import json
import os
import re
import struct
import sys
import time
import urllib.request
from collections import Counter
from datetime import datetime, timezone
from html import escape
from pathlib import Path

SAVES = Path(os.environ.get("LOCALAPPDATA", "")) / "kenshi" / "save"
OUT = Path(os.environ.get("LOCALAPPDATA", "")) / "Gaia" / "kenshi"
OLLAMA = os.environ.get("KENSHI_EYES_OLLAMA", "http://127.0.0.1:11434")
MODEL = os.environ.get("KENSHI_EYES_MODEL", "gaia-tissue-heart")
PLAYER_FACTION = "Nameless"
SAVE_POLL_S = 20
INBOX_POLL_S = 1
SPEAK_COOLDOWN_S = 20
NAME_KEY = struct.pack("<i", 4) + b"name"


def names_in(blob):
    names, i = [], blob.find(NAME_KEY)
    while i != -1:
        j = i + len(NAME_KEY)
        if j + 4 <= len(blob):
            (n,) = struct.unpack_from("<i", blob, j)
            if 0 < n < 64:
                raw = blob[j + 4 : j + 4 + n]
                if all(32 <= c < 127 for c in raw):
                    names.append(raw.decode("ascii"))
        i = blob.find(NAME_KEY, i + 1)
    return names


def newest_save():
    dirs = [d for d in SAVES.iterdir() if (d / "platoon").is_dir()] if SAVES.is_dir() else []
    return max(dirs, key=lambda d: (d / "quick.save").stat().st_mtime if (d / "quick.save").exists() else 0, default=None)


def snapshot(save_dir):
    squads, factions = {}, Counter()
    for f in (save_dir / "platoon").glob("*.platoon"):
        faction = re.sub(r"_\d+$", "", f.stem)
        factions[faction] += 1
        if faction == PLAYER_FACTION:
            members = names_in(f.read_bytes())
            if members:
                squads[f.stem] = members
    return {
        "save": save_dir.name,
        "saved_at": datetime.fromtimestamp((save_dir / "quick.save").stat().st_mtime, timezone.utc).isoformat(timespec="seconds"),
        "squads": squads,
        "factions": dict(factions.most_common()),
        "zones_loaded": sum(1 for _ in (save_dir / "zone").glob("*")) if (save_dir / "zone").is_dir() else 0,
    }


def diff(prev, now):
    if not prev:
        return {"first_sight": True}
    before = {m for ms in prev["squads"].values() for m in ms}
    after = {m for ms in now["squads"].values() for m in ms}
    moved = {}
    for fac in set(prev["factions"]) | set(now["factions"]):
        d = now["factions"].get(fac, 0) - prev["factions"].get(fac, 0)
        if d:
            moved[fac] = d
    return {"joined": sorted(after - before), "lost": sorted(before - after), "faction_shift": moved}


def generate(prompt, num_predict):
    body = json.dumps({"model": MODEL, "prompt": prompt, "stream": False, "options": {"num_predict": num_predict, "temperature": 0.8}}).encode()
    req = urllib.request.Request(f"{OLLAMA}/api/generate", data=body, headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=120) as r:
        return json.loads(r.read())["response"].strip()


def think_as(name, snap):
    squad = next((ms for ms in (snap or {}).get("squads", {}).values() if name in ms), [])
    mates = ", ".join(m for m in squad if m != name) or "no one"
    prompt = (
        f"You are {name}, one of the Nameless, in Kenshi's harsh sword-punk desert. "
        f"Your squadmates: {mates}. The player just tapped your shoulder. "
        "Say one short line in character, under 18 words. Only the line, no quotes, no name."
    )
    line = generate(prompt, 48).splitlines()[0].strip().lstrip('"')
    return line.split('"')[0].strip().replace("\t", " ")


def answer_inbox(state, snap):
    inbox = OUT / "inbox.txt"
    if not inbox.exists():
        return
    with open(inbox, "r", encoding="utf-8", errors="replace") as fh:
        fh.seek(state.get("offset", 0))
        lines = fh.readlines()
        state["offset"] = fh.tell()
    for raw in lines:
        parts = raw.rstrip("\n").split("\t")
        if len(parts) < 2 or parts[0] != "select":
            continue
        name, now = parts[1], time.time()
        if now - state["spoke"].get(name, 0) < SPEAK_COOLDOWN_S:
            continue
        state["spoke"][name] = now
        try:
            words = think_as(name, snap)
        except OSError as e:
            print(f"ollama unreachable for {name}: {e}", flush=True)
            continue
        with open(OUT / "outbox.txt", "a", encoding="utf-8") as fh:
            fh.write(f"{name}\t{words}\n")
        print(f"{name}: {words}", flush=True)


def think(snap, change):
    roster = "; ".join(f"{k}: {', '.join(v)}" for k, v in snap["squads"].items())
    near = ", ".join(f"{k} x{v}" for k, v in list(snap["factions"].items())[:12])
    prompt = (
        "You are the world of Kenshi, a harsh sword-punk desert, watching the player's faction, the Nameless.\n"
        f"Squads: {roster}\nFactions tracked in the world: {near}\n"
        f"What changed since the last save: {json.dumps(change)}\n"
        "Write three short lines, each spoken by a different NPC or faction reacting to this. "
        "Format each line as 'Speaker: words'. Plain text, no preamble."
    )
    return generate(prompt, 160)


def narrate(snap, change, thought):
    OUT.mkdir(parents=True, exist_ok=True)
    rec = {"at": datetime.now(timezone.utc).isoformat(timespec="seconds"), "model": MODEL, "save": snap["save"], "change": change, "thought": thought}
    with open(OUT / "thoughts.jsonl", "a", encoding="utf-8") as fh:
        fh.write(json.dumps(rec) + "\n")
    roster = "".join(f"<li><b>{escape(k)}</b> — {escape(', '.join(v))}</li>" for k, v in snap["squads"].items())
    lines = "".join(f"<p>{escape(l)}</p>" for l in thought.splitlines() if l.strip())
    page = f"""<!doctype html><meta charset="utf-8"><meta http-equiv="refresh" content="15">
<title>Kenshi — the world speaks</title>
<style>body{{background:#14110d;color:#d8c9a8;font:15px/1.5 Georgia,serif;margin:24px}}
h1{{font-size:18px;color:#c9a24a;letter-spacing:.1em}}p{{margin:.6em 0}}small{{color:#7d705a}}ul{{padding-left:18px}}</style>
<h1>THE WORLD SPEAKS</h1>{lines}<hr><small>{escape(snap['save'])} · saved {escape(snap['saved_at'])} · {escape(MODEL)}</small>
<h1>THE NAMELESS</h1><ul>{roster}</ul>"""
    (OUT / "narrator.html").write_text(page, encoding="utf-8")
    (OUT / "snapshot.json").write_text(json.dumps(snap, indent=1), encoding="utf-8")
    return rec


def run(watch):
    prev_path = OUT / "snapshot.json"
    prev = json.loads(prev_path.read_text(encoding="utf-8")) if prev_path.exists() else None
    seen, next_save_check = None, 0.0
    inbox = OUT / "inbox.txt"
    mailbox = {"offset": inbox.stat().st_size if inbox.exists() else 0, "spoke": {}}
    while True:
        if watch:
            answer_inbox(mailbox, prev)
        if time.time() < next_save_check:
            time.sleep(INBOX_POLL_S)
            continue
        next_save_check = time.time() + SAVE_POLL_S
        save = newest_save()
        stamp = save and (save.name, (save / "quick.save").stat().st_mtime)
        if save and stamp != seen:
            seen = stamp
            snap = snapshot(save)
            change = diff(prev, snap)
            try:
                rec = narrate(snap, change, think(snap, change))
                print(f"[{rec['at']}] {snap['save']}: {sum(map(len, snap['squads'].values()))} Nameless, {len(snap['factions'])} factions")
                print(rec["thought"], flush=True)
            except OSError as e:
                print(f"ollama unreachable ({e}); snapshot kept for next save", flush=True)
                (OUT / "snapshot.json").write_text(json.dumps(snap, indent=1), encoding="utf-8")
            prev = snap
        if not watch:
            return


if __name__ == "__main__":
    run(watch=len(sys.argv) > 1 and sys.argv[1] == "watch")
