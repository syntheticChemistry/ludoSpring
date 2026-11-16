#!/usr/bin/env python3
"""KenshiReborn: read Kenshi's autosaves, think as the world, narrate.

Reads only. Never writes under the Kenshi save folder. In watch mode it also
answers the GaiaMind KenshiLua mailbox (inbox.txt -> outbox.txt) so a selected
squad member speaks a line thought up by the tissue model. Each character keeps a
voice, trust and memory in minds.json, so they stay themselves between taps.

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
VOICE_MODEL = os.environ.get("KENSHI_EYES_VOICE_MODEL", "gaia-tissue-forge")
PLAYER_FACTION = "Nameless"
SAVE_POLL_S = 20
INBOX_POLL_S = 1
SPEAK_COOLDOWN_S = 20
NAME_KEY = struct.pack("<i", 4) + b"name"
MINDS = OUT / "minds.json"
MEMORY_KEEP = 8
TRUST_PER_TAP = 0.2
NEEDS = ("survival", "safety", "belonging", "esteem", "self_actualization")
TRUST_VOICE = {
    0: "guarded; you barely know the player",
    1: "civil, a little warmer",
    2: "you confide small frustrations",
    3: "you trust the player and say what you really think",
    4: "loyal; you share fears and hopes",
    5: "you would die for the player and speak like it",
}


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


def generate(prompt, num_predict, model=MODEL, temperature=0.8):
    body = json.dumps({"model": model, "prompt": prompt, "stream": False, "options": {"num_predict": num_predict, "temperature": temperature}}).encode()
    req = urllib.request.Request(f"{OLLAMA}/api/generate", data=body, headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=120) as r:
        return json.loads(r.read())["response"].strip()


QUOTES = str.maketrans({"\u201c": '"', "\u201d": '"', "\u2018": "'", "\u2019": "'"})
NARRATION = re.compile(r"^(\*|\(|i (glance|look|smile|nod|turn|shrug|sigh|whistle|grin|pause|lean|gaze|stare)s?\b)", re.I)


ECHO = re.compile(r"\b(tapped|the player|your shoulder|you said)\b", re.I)
SPEECH_TAG = re.compile(r'[,.]?"?\s*\bI (say|said|mutter|murmur|whisper|add|reply|sigh)\b[^"]*"?\s*', re.I)


def sentences(text):
    return [s.strip() for s in re.split(r"(?<=[.!?])\s+", text) if s.strip()]


def spoken(raw, said):
    """Keep only fresh speech: drop sentences already said, prompt echoes, stage directions, non-English."""
    line = next((l for l in raw.splitlines() if l.strip()), "").translate(QUOTES).replace("\t", " ")
    line = SPEECH_TAG.sub(". ", line).replace('"', "").strip(" .,")
    line = line + "." if line and line[-1] not in ".!?" else line
    fresh = [s for s in sentences(line) if s.lower() not in said and not ECHO.search(s) and not NARRATION.match(s)]
    line = " ".join(fresh[:2])
    if not line or sum(c.isascii() for c in line) / len(line) < 0.9:
        return ""
    return line


def load_minds():
    try:
        return json.loads(MINDS.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return {}


def save_minds(minds):
    OUT.mkdir(parents=True, exist_ok=True)
    tmp = MINDS.with_suffix(".tmp")
    tmp.write_text(json.dumps(minds, indent=1), encoding="utf-8")
    tmp.replace(MINDS)


def remember(mind, text):
    mind["memory"] = (mind.get("memory", []) + [text])[-MEMORY_KEEP:]


def birth(name):
    """First tap: give the character a lasting voice (RPGPT_NPC_PERSONALITY_SPEC, trimmed to what a line needs)."""
    need = NEEDS[sum(name.encode()) % len(NEEDS)]
    prompt = (
        f"Invent a Kenshi character named {name}, a member of the player's squad, the Nameless. "
        f"Their deepest need is {need.replace('_', '-')}. "
        'Reply with JSON only: {"temperament": "<3-6 words>", "want": "<what they want, under 10 words>", '
        '"habit": "<a verbal habit, under 8 words>"}'
    )
    try:
        raw = generate(prompt, 96, VOICE_MODEL)
        voice = json.loads(raw[raw.index("{") : raw.rindex("}") + 1])
    except (ValueError, KeyError):
        voice = {}
    return {
        "voice": {
            "temperament": str(voice.get("temperament") or "weathered and wary")[:60],
            "need": need,
            "want": str(voice.get("want") or "to see tomorrow")[:80],
            "habit": str(voice.get("habit") or "answers in few words")[:60],
        },
        "trust": 0.0,
        "taps": 0,
        "memory": [],
    }


def think_as(name, snap, minds):
    squad = next((ms for ms in (snap or {}).get("squads", {}).values() if name in ms), [])
    mates = ", ".join(m for m in squad if m != name) or "no one"
    mind = minds.get(name) or birth(name)
    v = mind["voice"]
    trust = min(5, int(mind["trust"]))
    recent = "\n".join(f"- {m}" for m in mind["memory"][-5:])
    recent = f"Lately (do not mention this list):\n{recent}\n" if recent else ""
    prompt = (
        f"You are {name}, one of the Nameless, in Kenshi's harsh sword-punk desert.\n"
        f"Temperament: {v['temperament']}. Deepest need right now: {v['need']}. You want {v['want']}. "
        f"Verbal habit: {v['habit']}.\n"
        f"Trust in the player ({trust}/5): {TRUST_VOICE[trust]}.\n"
        f"Squadmates: {mates}.\n{recent}"
        "The player turns to you. Say one short line of spoken dialogue in English, first person, "
        "under 18 words, about something new, no actions or narration. Kenshi is the land, not a person; "
        "never say the words trust, need or temperament. "
        "Only the spoken words, no quotes, no name."
    )
    said = {s.lower() for m in mind["memory"] if m.startswith("you said: ") for s in sentences(m[len("you said: "):])}
    for temperature in (0.8, 1.0, 1.2):
        line = spoken(generate(prompt, 48, VOICE_MODEL, temperature), said)
        if line:
            break
    else:
        return ""
    mind["trust"] = min(5.0, mind["trust"] + TRUST_PER_TAP)
    mind["taps"] += 1
    remember(mind, f"you said: {line}")
    minds[name] = mind
    return line


def witness(minds, change):
    """Squad changes between saves become memories for everyone who saw them."""
    joined, lost = change.get("joined", []), change.get("lost", [])
    if not (joined or lost):
        return
    for name, mind in minds.items():
        for j in joined:
            if j != name:
                remember(mind, f"{j} joined the squad")
        for l in lost:
            if l != name:
                remember(mind, f"{l} is gone from the squad")


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
        minds = load_minds()
        try:
            words = think_as(name, snap, minds)
        except OSError as e:
            print(f"ollama unreachable for {name}: {e}", flush=True)
            continue
        save_minds(minds)
        if not words:
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
            if watch:
                minds = load_minds()
                witness(minds, change)
                save_minds(minds)
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
