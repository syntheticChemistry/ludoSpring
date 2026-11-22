#!/usr/bin/env python3
"""Test the cortex — make squad members think WITHOUT Kenshi running."""

import json
import urllib.request
import time

OLLAMA_URL = "http://localhost:11434/api/generate"

TISSUE_MAP = {
    "select":  "gaia-tissue-cortex",
    "hit":     "gaia-tissue-gut",
    "ko":      "gaia-tissue-dream",
    "death":   "gaia-tissue-heart",
    "faction": "gaia-tissue-oracle",
    "idle":    "gaia-tissue-general",
    "cascade": "gaia-tissue-reason",
}

def ask(model, prompt, max_tokens=60):
    payload = json.dumps({
        "model": model,
        "prompt": prompt,
        "stream": False,
        "options": {"num_predict": max_tokens, "temperature": 0.8, "top_p": 0.9}
    }).encode()
    req = urllib.request.Request(OLLAMA_URL, data=payload, headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=30) as resp:
        result = json.loads(resp.read().decode())
        text = result.get("response", "").strip().strip('"').strip("'")
        for end in [". ", "! ", "? "]:
            if end in text:
                text = text[:text.index(end) + 1]
                break
        return text[:120] if text else "(silence)"

print("=" * 60)
print("  CORTEX DRY RUN — testing tissue responses")
print("=" * 60)

# Check models
print("\n--- Tissue Models ---")
try:
    req = urllib.request.urlopen("http://localhost:11434/api/tags", timeout=3)
    tags = json.loads(req.read().decode())
    models = [m["name"] for m in tags.get("models", [])]
    for event, model in TISSUE_MAP.items():
        found = any(model in m for m in models)
        print(f"  {event:10s} → {model:30s} {'✓' if found else '✗'}")
except Exception as e:
    print(f"  ERROR: {e}")
    exit(1)

# Simulate events
scenarios = [
    ("select", "cortex",
     "You are Beep, a curious scavenger in Kenshi's wasteland. "
     "The player just selected you. Say ONE short line about what you notice. Under 15 words."),

    ("hit", "gut",
     "You are Ruka, a fierce warrior in Kenshi. "
     "You just got HIT by a Dust Bandit! React with ONE visceral line. Under 12 words."),

    ("ko", "dream",
     "You are Sadneil, a weary wanderer in Kenshi. "
     "You were knocked unconscious. You're dreaming. ONE surreal line. Under 15 words."),

    ("death", "heart",
     "Hobbs, a stoic warrior in Kenshi, has just died. "
     "Write their final thought — poetic, brief. Under 15 words."),

    ("faction", "oracle",
     "Your squad just encountered the Holy Nation faction in Kenshi. "
     "The leader says ONE short assessment. Under 15 words."),

    ("idle", "general",
     "You are Cat, a dark humorist in Kenshi. During a quiet moment, you think about "
     "how hungry you are. Say ONE short idle thought. Under 15 words."),

    ("cascade", "reason",
     "You are Agnu, a quiet scholar. Your squadmate Beep just said: "
     "'Look at that dust cloud, something's coming!' "
     "Respond with ONE short line. Under 12 words."),
]

print("\n--- Simulated Events ---\n")
for event, tissue_key, prompt in scenarios:
    model = TISSUE_MAP[event]
    print(f"  [{event}] asking {model}...", end=" ", flush=True)
    t0 = time.time()
    try:
        response = ask(model, prompt)
        dt = time.time() - t0
        print(f"({dt:.1f}s)")
        print(f"    → {response}\n")
    except Exception as e:
        print(f"ERROR: {e}\n")

print("=" * 60)
print("  CORTEX TEST COMPLETE")
print("  When Kenshi runs, launch: python kenshi_cortex.py")
print("=" * 60)
