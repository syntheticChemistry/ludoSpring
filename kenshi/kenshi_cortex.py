#!/usr/bin/env python3
"""
kenshi_cortex.py — The brain that makes Kenshi squads THINK.

Connects to gaia_nerve.dll on 127.0.0.1:7741, receives game events,
routes them through Gaia's tissue models (ollama), and sends dialogue
back so squad members SPEAK with personality.

Flow:
  Kenshi → gaia_mind.lua → gaia_nerve.dll (:7741) → THIS → ollama → back

Tissue routing:
  select   → cortex  (what does this character notice?)
  hit      → gut     (visceral reaction to being attacked)
  ko       → dream   (unconscious mind speaks)
  death    → heart   (grief, final words, squad mourns)
  faction  → oracle  (what do we know about this faction?)

Cascade: when one character speaks, nearby squad members may respond.
"""

import socket
import threading
import json
import time
import random
import urllib.request
from collections import deque
from dataclasses import dataclass, field
from typing import Optional

NERVE_HOST = "127.0.0.1"
NERVE_PORT = 7741
OLLAMA_URL = "http://localhost:11434/api/generate"

# ═══════════════════════════════════════════════════════════
# TISSUE ROUTING — which model responds to which event
# ═══════════════════════════════════════════════════════════

TISSUE_MAP = {
    "select":  "gaia-tissue-cortex",
    "hit":     "gaia-tissue-gut",
    "ko":      "gaia-tissue-dream",
    "death":   "gaia-tissue-heart",
    "faction": "gaia-tissue-oracle",
    "idle":    "gaia-tissue-general",
    "cascade": "gaia-tissue-reason",
}

# ═══════════════════════════════════════════════════════════
# SQUAD MEMORY — each character has personality + history
# ═══════════════════════════════════════════════════════════

@dataclass
class SquadMember:
    name: str
    personality: str = ""
    mood: str = "calm"
    history: deque = field(default_factory=lambda: deque(maxlen=20))
    last_spoke: float = 0
    speak_cooldown: float = 8.0

    def can_speak(self) -> bool:
        return time.time() - self.last_spoke >= self.speak_cooldown

    def spoke(self):
        self.last_spoke = time.time()


SQUAD: dict[str, SquadMember] = {}
SPEAK_QUEUE: deque = deque(maxlen=64)
CASCADE_QUEUE: deque = deque(maxlen=32)

# ═══════════════════════════════════════════════════════════
# PERSONALITY SEEDS — Kenshi archetypes
# ═══════════════════════════════════════════════════════════

ARCHETYPES = [
    "stoic warrior who speaks in short, blunt sentences",
    "curious scavenger who asks questions about everything",
    "bitter veteran who remembers every wound",
    "optimistic dreamer who sees beauty in the wasteland",
    "pragmatic trader who thinks in terms of profit and survival",
    "quiet scholar who quotes ancient texts",
    "fierce protector who threatens anyone near the squad",
    "weary wanderer who just wants to find somewhere safe",
    "dark humorist who jokes about death constantly",
    "spiritual mystic who speaks of the old world's ghosts",
]

def get_member(name: str) -> SquadMember:
    if name not in SQUAD:
        personality = random.choice(ARCHETYPES)
        SQUAD[name] = SquadMember(name=name, personality=personality)
        print(f"  [new] {name}: {personality}")
    return SQUAD[name]

# ═══════════════════════════════════════════════════════════
# OLLAMA — talk to tissue models
# ═══════════════════════════════════════════════════════════

def ask_tissue(model: str, prompt: str, max_tokens: int = 60) -> Optional[str]:
    """Ask a tissue model and get a short response."""
    try:
        payload = json.dumps({
            "model": model,
            "prompt": prompt,
            "stream": False,
            "options": {
                "num_predict": max_tokens,
                "temperature": 0.8,
                "top_p": 0.9,
                "stop": ["\n\n", "---", "###"],
            }
        }).encode()

        req = urllib.request.Request(
            OLLAMA_URL,
            data=payload,
            headers={"Content-Type": "application/json"},
        )
        with urllib.request.urlopen(req, timeout=15) as resp:
            result = json.loads(resp.read().decode())
            text = result.get("response", "").strip()
            # Clean: remove quotes, trim to one sentence max
            text = text.strip('"').strip("'").strip()
            # Take first sentence only
            for end in [". ", "! ", "? "]:
                if end in text:
                    text = text[:text.index(end) + 1]
                    break
            if len(text) > 120:
                text = text[:117] + "..."
            return text if text else None
    except Exception as e:
        print(f"  [tissue error] {model}: {e}")
        return None

# ═══════════════════════════════════════════════════════════
# EVENT HANDLERS — think about what happened
# ═══════════════════════════════════════════════════════════

def handle_select(name: str, _detail: str):
    """Player clicked a character — they notice something."""
    member = get_member(name)
    if not member.can_speak():
        return

    prompt = (
        f"You are {name}, a {member.personality} in the post-apocalyptic world of Kenshi. "
        f"Your mood is {member.mood}. The player just selected you. "
        f"Say ONE short line about what you notice around you. "
        f"Be in character. No quotes. Under 15 words."
    )
    line = ask_tissue(TISSUE_MAP["select"], prompt)
    if line:
        SPEAK_QUEUE.append((name, line))
        member.history.append(f"[said] {line}")
        member.spoke()
        trigger_cascade(name, line, "select")


def handle_hit(name: str, detail: str):
    """Character was hit — visceral gut reaction."""
    member = get_member(name)
    if not member.can_speak():
        return

    member.mood = "angry"
    attacker = detail.split("(")[0].strip() if detail else "someone"

    prompt = (
        f"You are {name}, a {member.personality} in Kenshi. "
        f"You just got HIT by {attacker}! React with ONE short visceral line. "
        f"Pain, anger, defiance, or dark humor. No quotes. Under 12 words."
    )
    line = ask_tissue(TISSUE_MAP["hit"], prompt)
    if line:
        SPEAK_QUEUE.append((name, line))
        member.history.append(f"[hit by {attacker}] {line}")
        member.spoke()
        trigger_cascade(name, line, "hit")


def handle_ko(name: str, _detail: str):
    """Character knocked out — dream state."""
    member = get_member(name)
    member.mood = "dazed"

    prompt = (
        f"You are {name}, a {member.personality} in Kenshi. "
        f"You were just knocked unconscious. You're dreaming. "
        f"Say ONE dreamlike, surreal line. No quotes. Under 15 words."
    )
    line = ask_tissue(TISSUE_MAP["ko"], prompt)
    if line:
        SPEAK_QUEUE.append((name, line))
        member.history.append(f"[ko dream] {line}")
        member.spoke()


def handle_death(name: str, detail: str):
    """Character died — the heart grieves."""
    member = get_member(name)

    prompt = (
        f"{name}, a {member.personality} in Kenshi, has just DIED. "
        f"They belonged to faction: {detail}. "
        f"Write their final thought — one line, poetic, brief. Under 15 words."
    )
    line = ask_tissue(TISSUE_MAP["death"], prompt)
    if line:
        SPEAK_QUEUE.append((name, line))

    # Other squad members mourn
    for other_name, other in SQUAD.items():
        if other_name != name and other.can_speak():
            mourn_prompt = (
                f"You are {other_name}, a {other.personality}. "
                f"Your squadmate {name} just died. React in ONE short line. "
                f"Grief, anger, or resolve. No quotes. Under 12 words."
            )
            mourn_line = ask_tissue(TISSUE_MAP["death"], mourn_prompt)
            if mourn_line:
                SPEAK_QUEUE.append((other_name, mourn_line))
                other.spoke()
            break  # Only one mourner per death to avoid spam


def handle_faction(name: str, faction: str):
    """Encountered a new faction — the oracle assesses."""
    prompt = (
        f"You are a squad in Kenshi. You just encountered the faction: {faction}. "
        f"The squad leader says ONE short assessment line. "
        f"Threat level, reputation, or tactical advice. Under 15 words."
    )
    line = ask_tissue(TISSUE_MAP["faction"], prompt)
    if line:
        # Pick a random squad member to speak
        if SQUAD:
            speaker = random.choice(list(SQUAD.keys()))
            SPEAK_QUEUE.append((speaker, line))
            SQUAD[speaker].spoke()

# ═══════════════════════════════════════════════════════════
# CASCADE — one character's words trigger others
# ═══════════════════════════════════════════════════════════

def trigger_cascade(speaker: str, line: str, context: str):
    """Maybe another squad member responds."""
    if len(SQUAD) < 2:
        return
    # 30% chance of cascade
    if random.random() > 0.3:
        return

    others = [n for n in SQUAD if n != speaker and SQUAD[n].can_speak()]
    if not others:
        return

    responder_name = random.choice(others)
    responder = SQUAD[responder_name]

    CASCADE_QUEUE.append((responder_name, speaker, line, context))


def process_cascades():
    """Process pending cascade responses."""
    while CASCADE_QUEUE:
        responder_name, speaker, trigger_line, context = CASCADE_QUEUE.popleft()
        responder = get_member(responder_name)
        if not responder.can_speak():
            continue

        prompt = (
            f"You are {responder_name}, a {responder.personality} in Kenshi. "
            f"Your squadmate {speaker} just said: '{trigger_line}' "
            f"(context: {context}). "
            f"Respond with ONE short line — agree, disagree, or add to it. "
            f"No quotes. Under 12 words."
        )
        line = ask_tissue(TISSUE_MAP["cascade"], prompt)
        if line:
            SPEAK_QUEUE.append((responder_name, line))
            responder.history.append(f"[replied to {speaker}] {line}")
            responder.spoke()

# ═══════════════════════════════════════════════════════════
# IDLE CHATTER — squads talk when nothing's happening
# ═══════════════════════════════════════════════════════════

LAST_IDLE = 0
IDLE_INTERVAL = 45  # seconds between idle chatter

def maybe_idle_chatter():
    """Occasionally, a squad member says something unprompted."""
    global LAST_IDLE
    now = time.time()
    if now - LAST_IDLE < IDLE_INTERVAL:
        return
    if not SQUAD:
        return
    if random.random() > 0.15:  # 15% chance per check
        return

    LAST_IDLE = now
    name = random.choice(list(SQUAD.keys()))
    member = SQUAD[name]
    if not member.can_speak():
        return

    topics = [
        "the landscape around you",
        "how hungry or tired you are",
        "a memory from before this journey",
        "the weather or the sky",
        "what you think of the squad leader's decisions",
        "a rumor you heard about nearby ruins",
        "whether you'll survive the next fight",
    ]

    prompt = (
        f"You are {name}, a {member.personality} in Kenshi's wasteland. "
        f"Your mood is {member.mood}. During a quiet moment, you think about "
        f"{random.choice(topics)}. "
        f"Say ONE short idle thought aloud. No quotes. Under 15 words."
    )
    line = ask_tissue(TISSUE_MAP["idle"], prompt)
    if line:
        SPEAK_QUEUE.append((name, line))
        member.history.append(f"[idle] {line}")
        member.spoke()
        trigger_cascade(name, line, "idle")

# ═══════════════════════════════════════════════════════════
# NERVE CONNECTION — TCP to gaia_nerve.dll
# ═══════════════════════════════════════════════════════════

EVENT_HANDLERS = {
    "select":  lambda parts: handle_select(parts[1], ""),
    "event":   None,  # dispatched below
}

def dispatch_event(line: str):
    """Parse a nerve line and dispatch to the right handler."""
    parts = line.strip().split("\t")
    if not parts:
        return

    kind = parts[0]
    if kind == "select" and len(parts) >= 2:
        handle_select(parts[1], "")
    elif kind == "event" and len(parts) >= 3:
        etype = parts[1]  # hit, ko, death, faction
        who = parts[2] if len(parts) > 2 else ""
        detail = parts[3] if len(parts) > 3 else ""
        if etype == "hit":
            handle_hit(who, detail)
        elif etype == "ko":
            handle_ko(who, detail)
        elif etype == "death":
            handle_death(who, detail)
        elif etype == "faction":
            handle_faction(who, detail)


def nerve_reader(sock: socket.socket):
    """Read lines from the nerve and dispatch events."""
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
                    print(f"  [nerve ←] {text}")
                    dispatch_event(text)
        except (ConnectionError, OSError):
            break
    print("  [nerve] disconnected")


def nerve_writer(sock: socket.socket):
    """Send queued dialogue lines back through the nerve."""
    while True:
        time.sleep(0.5)

        # Process cascades
        process_cascades()

        # Idle chatter
        maybe_idle_chatter()

        # Send queued lines
        while SPEAK_QUEUE:
            name, line = SPEAK_QUEUE.popleft()
            msg = f"{name}\t{line}\n".encode("utf-8")
            try:
                sock.sendall(msg)
                print(f"  [nerve →] {name}: {line}")
            except (ConnectionError, OSError):
                return


def connect_nerve() -> Optional[socket.socket]:
    """Connect to gaia_nerve.dll."""
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.connect((NERVE_HOST, NERVE_PORT))
        sock.settimeout(None)
        print(f"  [nerve] connected to {NERVE_HOST}:{NERVE_PORT}")
        return sock
    except ConnectionRefusedError:
        return None
    except Exception as e:
        print(f"  [nerve] connection failed: {e}")
        return None

# ═══════════════════════════════════════════════════════════
# MAIN — the cortex awakens
# ═══════════════════════════════════════════════════════════

def main():
    print("=" * 55)
    print("  KENSHI CORTEX — squads think, speak, cascade")
    print("  nerve: 127.0.0.1:7741")
    print("  ollama: localhost:11434")
    print(f"  tissues: {len(TISSUE_MAP)} routes")
    print("=" * 55)

    # Verify ollama is running
    try:
        req = urllib.request.urlopen("http://localhost:11434/api/tags", timeout=3)
        tags = json.loads(req.read().decode())
        models = [m["name"] for m in tags.get("models", [])]
        tissue_count = sum(1 for m in models if "tissue" in m)
        print(f"\n  [ollama] {len(models)} models, {tissue_count} tissues")
        for model in TISSUE_MAP.values():
            status = "✓" if any(model in m for m in models) else "✗ MISSING"
            print(f"    {model}: {status}")
    except Exception as e:
        print(f"\n  [ollama] WARNING: {e}")
        print("  cortex will retry on each request")

    # Connect to nerve (retry loop)
    print(f"\n  [nerve] waiting for Kenshi + gaia_nerve.dll on :{NERVE_PORT}...")
    sock = None
    while sock is None:
        sock = connect_nerve()
        if sock is None:
            print("  [nerve] Kenshi not running yet, retrying in 5s...")
            time.sleep(5)

    print("\n  ═══ CORTEX ONLINE — squads are thinking ═══\n")

    # Launch reader and writer threads
    reader = threading.Thread(target=nerve_reader, args=(sock,), daemon=True)
    writer = threading.Thread(target=nerve_writer, args=(sock,), daemon=True)
    reader.start()
    writer.start()

    try:
        reader.join()
    except KeyboardInterrupt:
        print("\n  [cortex] shutting down")
    finally:
        sock.close()
        print("  [cortex] nerve disconnected. squads go silent.")


if __name__ == "__main__":
    main()
