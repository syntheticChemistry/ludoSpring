#!/usr/bin/env python3
"""thinking_squads.py — Squads that THINK, PLAN, and CAST.

Evolution of kenshi_eyes.py: squads are no longer just individuals who speak
when tapped. Now the squad thinks as a collective:

  Level 1: THINKING  — idle chatter, NPC-to-NPC dialogue, environmental awareness
  Level 2: PLANNING  — squad discusses goals, threats, routes
  Level 3: CASTING   — Gaia recommends squad composition for missions

Runs alongside kenshi_eyes.py (which handles tap responses and event reactions).
This module handles the UNPROMPTED life of the squad.

Usage:
    python thinking_squads.py          # one cycle of thinking
    python thinking_squads.py watch    # continuous thinking loop
"""
import json
import os
import random
import socket
import sys
import time
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

OLLAMA = os.environ.get("KENSHI_EYES_OLLAMA", "http://127.0.0.1:11434")
HEART_MODEL = "gaia-tissue-heart"
CORTEX_MODEL = "gaia-tissue-cortex"
TRIAGE_MODEL = "gaia-tissue-triage"
FAST_MODEL = "gaia-tissue-fast"

OUT = Path(os.environ.get("LOCALAPPDATA", "")) / "Gaia" / "kenshi"
MINDS = OUT / "minds.json"
NERVE_PORT = int(os.environ.get("KENSHI_NERVE_PORT", "7741"))

THINK_INTERVAL_S = 45
PLAN_INTERVAL_S = 180
IDLE_INTERVAL_S = 25

ANTI_REPEAT = {"repeat_penalty": 1.15, "presence_penalty": 0.4, "frequency_penalty": 0.3}


def generate(prompt, num_predict, model=HEART_MODEL, temperature=0.85):
    options = {"num_predict": num_predict, "temperature": temperature, **ANTI_REPEAT}
    body = json.dumps({"model": model, "prompt": prompt, "stream": False, "options": options}).encode()
    req = urllib.request.Request(f"{OLLAMA}/api/generate", data=body, headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=120) as r:
        return json.loads(r.read())["response"].strip()


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


def load_snapshot():
    snap_path = OUT / "snapshot.json"
    try:
        return json.loads(snap_path.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return {}


def remember(mind, text):
    mind["memory"] = (mind.get("memory", []) + [text])[-8:]


def nerve_send(line):
    try:
        s = socket.create_connection(("127.0.0.1", NERVE_PORT), timeout=0.5)
        s.sendall((line + "\n").encode("utf-8"))
        s.close()
        return True
    except OSError:
        return False


def deliver(speaker, words):
    line = f"{speaker}\t{words}"
    if not nerve_send(line):
        with open(OUT / "outbox.txt", "a", encoding="utf-8") as fh:
            fh.write(line + "\n")
    print(f"  [{speaker}]: {words}", flush=True)


# ═══════════════════════════════════════════════════════════════
# LEVEL 1: THINKING — idle chatter, NPC-to-NPC, awareness
# ═══════════════════════════════════════════════════════════════

def idle_thought(minds, snap):
    """One random squad member thinks aloud unprompted."""
    names = [n for n in minds if minds[n].get("voice")]
    if not names:
        return
    speaker = random.choice(names)
    mind = minds[speaker]
    v = mind["voice"]
    squad = next((ms for ms in snap.get("squads", {}).values() if speaker in ms), names)
    mates = [m for m in squad if m != speaker]

    factions = snap.get("factions", {})
    nearby = ", ".join(list(factions.keys())[:5])

    recent_mem = "\n".join(f"- {m}" for m in mind.get("memory", [])[-3:])
    recent_mem = f"Recent: {recent_mem}\n" if recent_mem else ""

    prompt = (
        f"You are {speaker}, one of the Nameless in Kenshi's sword-punk desert.\n"
        f"Temperament: {v['temperament']}. You want {v['want']}. Habit: {v['habit']}.\n"
        f"Squadmates nearby: {', '.join(mates) if mates else 'no one'}.\n"
        f"Factions in the area: {nearby or 'quiet'}.\n"
        f"{recent_mem}"
        "You mutter something to yourself or to a squadmate — an idle thought about "
        "the weather, the road, your hunger, a memory, or something you see. "
        "One short spoken line under 14 words. No actions, no quotes, no name prefix."
    )
    line = generate(prompt, 36, HEART_MODEL, 0.9)
    line = line.strip().strip('"').strip("*").strip()
    if line and len(line) > 5:
        remember(mind, f"you muttered: {line}")
        minds[speaker] = mind
        deliver(speaker, line)
        return speaker


def npc_to_npc(minds, snap):
    """Two squad members have a brief exchange."""
    names = [n for n in minds if minds[n].get("voice")]
    if len(names) < 2:
        return

    squad = next((ms for ms in snap.get("squads", {}).values() if len(ms) >= 2), None)
    if squad:
        pair = random.sample([n for n in squad if n in minds], min(2, len([n for n in squad if n in minds])))
    else:
        pair = random.sample(names, 2)

    if len(pair) < 2:
        return

    a_name, b_name = pair
    a_mind, b_mind = minds[a_name], minds[b_name]
    a_v, b_v = a_mind["voice"], b_mind["voice"]

    prompt_a = (
        f"You are {a_name}, one of the Nameless in Kenshi. "
        f"Temperament: {a_v['temperament']}. Habit: {a_v['habit']}.\n"
        f"You say something to {b_name} — a question, observation, or complaint. "
        "One short line under 12 words. Spoken dialogue only, no actions."
    )
    line_a = generate(prompt_a, 30, HEART_MODEL, 0.9).strip().strip('"').strip("*")

    if not line_a or len(line_a) < 4:
        return

    prompt_b = (
        f"You are {b_name}, one of the Nameless in Kenshi. "
        f"Temperament: {b_v['temperament']}. Habit: {b_v['habit']}.\n"
        f"{a_name} just said to you: \"{line_a}\"\n"
        "Reply in one short line under 12 words. Spoken dialogue only."
    )
    line_b = generate(prompt_b, 30, HEART_MODEL, 0.95).strip().strip('"').strip("*")

    if line_a:
        remember(a_mind, f"you said to {b_name}: {line_a}")
        minds[a_name] = a_mind
        deliver(a_name, line_a)

    if line_b and len(line_b) > 3:
        time.sleep(3)
        remember(b_mind, f"you replied to {a_name}: {line_b}")
        minds[b_name] = b_mind
        deliver(b_name, line_b)

    return a_name, b_name


# ═══════════════════════════════════════════════════════════════
# LEVEL 2: PLANNING — squad discusses goals and threats
# ═══════════════════════════════════════════════════════════════

def plan_squad(minds, snap):
    """The squad collectively discusses what to do next."""
    names = [n for n in minds if minds[n].get("voice")]
    if not names:
        return

    factions = snap.get("factions", {})
    hostile = [f for f in factions if factions[f] > 3 and f not in ("Nameless",)]
    zones = snap.get("zones_loaded", 0)
    squad_size = sum(len(ms) for ms in snap.get("squads", {}).values())

    prompt = (
        "You are the collective mind of the Nameless squad in Kenshi's sword-punk desert.\n"
        f"Squad size: {squad_size}. Zones explored: {zones}.\n"
        f"Factions nearby: {', '.join(list(factions.keys())[:8]) or 'quiet'}.\n"
        f"Potential threats: {', '.join(hostile) or 'none apparent'}.\n"
        f"Members: {', '.join(names[:10])}.\n"
        "What should the squad do next? Give a brief tactical plan in 2-3 sentences. "
        "Be specific to Kenshi: mention training, bases, raids, trading, or exploring. "
        "Plain text, no formatting."
    )
    plan = generate(prompt, 100, CORTEX_MODEL, 0.8)

    if plan:
        log_path = OUT / "squad-plans.jsonl"
        OUT.mkdir(parents=True, exist_ok=True)
        entry = {
            "ts": datetime.now(timezone.utc).isoformat(timespec="seconds"),
            "squad_size": squad_size,
            "zones": zones,
            "plan": plan,
        }
        with open(log_path, "a", encoding="utf-8") as fh:
            fh.write(json.dumps(entry) + "\n")

        for mind in minds.values():
            remember(mind, f"squad plan: {plan[:80]}")

        deliver("@gaia", f"Squad plan: {plan[:120]}")
        print(f"  [PLAN] {plan}", flush=True)
        return plan


# ═══════════════════════════════════════════════════════════════
# LEVEL 3: CASTING — recommend squad compositions
# ═══════════════════════════════════════════════════════════════

def cast_squad(minds, snap, mission="general patrol"):
    """Recommend which members should go on a specific mission."""
    names = [n for n in minds if minds[n].get("voice")]
    if len(names) < 3:
        return

    roster = []
    for name in names:
        m = minds[name]
        v = m.get("voice", {})
        trust = m.get("trust", 0)
        taps = m.get("taps", 0)
        roster.append(f"{name}: {v.get('temperament', '?')}, trust {trust:.1f}, taps {taps}, wants {v.get('want', '?')}")

    prompt = (
        f"You are Gaia, the squad advisor for the Nameless in Kenshi.\n"
        f"Mission: {mission}\n"
        f"Available members:\n" + "\n".join(f"  {r}" for r in roster) + "\n\n"
        "Pick 3-5 members for this mission and explain why in 2 sentences. "
        "Consider their temperament, trust level, and what they want. "
        "Format: list the names, then the reason."
    )
    cast = generate(prompt, 120, CORTEX_MODEL, 0.7)

    if cast:
        log_path = OUT / "squad-casts.jsonl"
        OUT.mkdir(parents=True, exist_ok=True)
        entry = {
            "ts": datetime.now(timezone.utc).isoformat(timespec="seconds"),
            "mission": mission,
            "cast": cast,
            "roster_size": len(names),
        }
        with open(log_path, "a", encoding="utf-8") as fh:
            fh.write(json.dumps(entry) + "\n")

        deliver("@gaia", f"Cast for {mission}: {cast[:120]}")
        print(f"  [CAST] {cast}", flush=True)
        return cast


# ═══════════════════════════════════════════════════════════════
# MAIN LOOP
# ═══════════════════════════════════════════════════════════════

def run(watch=False):
    print("=" * 60)
    print("  THINKING SQUADS — Kenshi Reborn")
    print(f"  Ollama: {OLLAMA}")
    print(f"  Nerve: 127.0.0.1:{NERVE_PORT}")
    print(f"  Models: {HEART_MODEL} (voice), {CORTEX_MODEL} (plan)")
    print("=" * 60)

    minds = load_minds()
    snap = load_snapshot()

    if not minds:
        print("  No minds found — run kenshi_eyes.py first to birth NPCs.")
        print("  (or launch Kenshi, tap some characters, let kenshi_eyes birth them)")
        return

    names = [n for n in minds if minds[n].get("voice")]
    print(f"\n  Squad: {len(names)} members with voices")
    for n in names:
        m = minds[n]
        v = m.get("voice", {})
        print(f"    {n}: {v.get('temperament', '?')} (trust {m.get('trust', 0):.1f}, {m.get('taps', 0)} taps)")

    if not watch:
        print("\n  --- ONE CYCLE ---")
        idle_thought(minds, snap)
        save_minds(minds)
        time.sleep(2)
        npc_to_npc(minds, snap)
        save_minds(minds)
        time.sleep(2)
        plan_squad(minds, snap)
        save_minds(minds)
        return

    print(f"\n  --- WATCH MODE ---")
    print(f"  Idle: every {IDLE_INTERVAL_S}s | Think: every {THINK_INTERVAL_S}s | Plan: every {PLAN_INTERVAL_S}s")

    last_idle = last_think = last_plan = 0
    cycle = 0

    while True:
        now = time.time()
        minds = load_minds()
        snap = load_snapshot()

        if now - last_idle >= IDLE_INTERVAL_S:
            last_idle = now
            try:
                idle_thought(minds, snap)
                save_minds(minds)
            except Exception as e:
                print(f"  idle error: {e}", flush=True)

        if now - last_think >= THINK_INTERVAL_S:
            last_think = now
            try:
                npc_to_npc(minds, snap)
                save_minds(minds)
            except Exception as e:
                print(f"  think error: {e}", flush=True)

        if now - last_plan >= PLAN_INTERVAL_S:
            last_plan = now
            cycle += 1
            try:
                plan_squad(minds, snap)
                save_minds(minds)
                if cycle % 3 == 0:
                    missions = ["raid the nearest bandit camp", "trade run to the Hub",
                                "explore the Fog Islands", "general patrol", "defend the base",
                                "hunt for ancient tech", "rescue a prisoner"]
                    cast_squad(minds, snap, random.choice(missions))
                    save_minds(minds)
            except Exception as e:
                print(f"  plan error: {e}", flush=True)

        time.sleep(5)


if __name__ == "__main__":
    run(watch=len(sys.argv) > 1 and sys.argv[1] == "watch")
