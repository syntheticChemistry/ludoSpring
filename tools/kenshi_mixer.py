#!/usr/bin/env python3
"""Kenshi mod mixer: eats Nexus Mods, mixes random loadouts into the game, keeps crashes as data.

  pull N    probe random Nexus mod ids, download N new data-only mods into the local library
  mix K     judge the running loadout, back up the active save, swap in K random library mods
  judge     close the current loadout (crash dumps, session length) without changing mods
  clear     judge, then remove every mixed mod and restore the baseline mods.cfg
  status    library size, open loadout, and per-mod fitness from the ledger

Data lives under %LOCALAPPDATA%\\Gaia\\kenshi\\mixer (never in the repo). The Nexus key is read
from ludoSpring's data/keys and never printed. Mods shipping executable code are refused.
"""

import json
import os
import random
import shutil
import subprocess
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

KENSHI = Path(os.environ.get("KENSHI_DIR", r"M:\SteamLibrary\steamapps\common\Kenshi"))
MODS_CFG = KENSHI / "data" / "mods.cfg"
SAVES = Path(os.environ["LOCALAPPDATA"]) / "kenshi" / "save"
CRASH_DUMPS = [Path(os.environ["LOCALAPPDATA"]) / "CrashDumps", Path(r"C:\Windows\LiveKernelReports\WATCHDOG")]
HOME = Path(os.environ["LOCALAPPDATA"]) / "Gaia" / "kenshi" / "mixer"
LIBRARY = HOME / "library"
BACKUPS = HOME / "save-backups"
LEDGER = HOME / "ledger.jsonl"
STATE = HOME / "state.json"
KEY_FILES = [Path(os.environ.get("NEXUS_KEY_FILE", r"C:\Users\mokke\Desktop\ludoSpring\data\keys\nexusmods.json")),
             Path(r"C:\Users\mokke\Desktop\ludoSpring\data\nexus-api-key.json")]
SEVEN_ZIP = Path(r"C:\Program Files\7-Zip\7z.exe")
API = "https://api.nexusmods.com/v1/games/kenshi"
MAX_MOD_ID = 2400
KEEP_BACKUPS = 3
CODE = {".dll", ".exe", ".asi", ".bat", ".cmd", ".ps1", ".vbs", ".lua", ".py", ".js"}
MAX_ARCHIVE_MB = 300


def nexus(path):
    key = next(json.loads(f.read_text())["key"] for f in KEY_FILES if f.exists())
    req = urllib.request.Request(API + path, headers={
        "apikey": key, "Application-Name": "Gaia-kenshi-mixer", "Application-Version": "0.1",
        "User-Agent": "Gaia-kenshi-mixer/0.1"})
    with urllib.request.urlopen(req, timeout=30) as r:
        left = r.headers.get("x-rl-hourly-remaining")
        if left is not None and int(left) < 20:
            raise SystemExit(f"nexus hourly budget nearly spent ({left} left); stopping")
        return json.load(r)


def load_state():
    return json.loads(STATE.read_text()) if STATE.exists() else {"installed": [], "baseline": None, "open": None}


def save_state(state):
    STATE.write_text(json.dumps(state, indent=1))


def ledger(entry):
    with open(LEDGER, "a", encoding="utf-8") as fh:
        fh.write(json.dumps(entry) + "\n")


def kenshi_running():
    out = subprocess.run(["tasklist", "/FI", "IMAGENAME eq kenshi_x64.exe", "/NH"], capture_output=True, text=True).stdout
    return "kenshi_x64.exe" in out


def library():
    return sorted((json.loads(p.read_text()) for p in LIBRARY.glob("*/mod.json")), key=lambda m: m["mod_id"])


def pick_file(files):
    main = [f for f in files if f.get("category_name") == "MAIN"] or [f for f in files if f.get("category_name") not in ("OLD_VERSION", "ARCHIVED", "DELETED")]
    return max(main, key=lambda f: f.get("uploaded_timestamp", 0)) if main else None


def mod_units(root):
    """Each installable unit is a .mod file plus the folder it sits in."""
    return [m for m in root.rglob("*.mod") if m.is_file()]


def eat(mod_id):
    """Download one Nexus mod into the library. Returns its record, or a reason it was refused."""
    if (LIBRARY / str(mod_id) / "mod.json").exists():
        return "already eaten"
    try:
        info = nexus(f"/mods/{mod_id}.json")
    except urllib.error.HTTPError as e:
        return f"http {e.code}"
    if info.get("status") != "published" or not info.get("available") or info.get("contains_adult_content"):
        return f"status {info.get('status')}"
    file = pick_file(nexus(f"/mods/{mod_id}/files.json").get("files", []))
    if not file:
        return "no main file"
    if (file.get("size_kb") or 0) > MAX_ARCHIVE_MB * 1024:
        return f"too big ({file['size_kb'] // 1024} MB)"
    links = nexus(f"/mods/{mod_id}/files/{file['file_id']}/download_link.json")
    if not links:
        return "no download link"
    where = LIBRARY / str(mod_id)
    raw = where / "raw"
    shutil.rmtree(where, ignore_errors=True)
    raw.mkdir(parents=True)
    archive = where / file["file_name"]
    url = urllib.parse.quote(links[0]["URI"], safe=":/?&=%")
    with urllib.request.urlopen(urllib.request.Request(url, headers={"User-Agent": "Gaia-kenshi-mixer/0.1"}), timeout=120) as r, \
            open(archive, "wb") as out:
        shutil.copyfileobj(r, out)
    subprocess.run([str(SEVEN_ZIP), "x", f"-o{raw}", "-y", str(archive)], capture_output=True)
    archive.unlink()
    code = sorted({p.suffix.lower() for p in raw.rglob("*") if p.suffix.lower() in CODE})
    units = mod_units(raw)
    reason = f"ships code {code}" if code else None if units else "no .mod inside"
    if reason:
        shutil.rmtree(where, ignore_errors=True)
        return reason
    record = {"mod_id": mod_id, "name": info["name"], "version": info.get("version"), "author": info.get("author"),
              "category_id": info.get("category_id"), "endorsements": info.get("endorsement_count"),
              "file": file["file_name"], "units": [str(u.relative_to(raw)) for u in units],
              "summary": (info.get("summary") or "")[:300], "eaten": time.strftime("%Y-%m-%dT%H:%M:%S")}
    (where / "mod.json").write_text(json.dumps(record, indent=1))
    return record


def pull(n):
    LIBRARY.mkdir(parents=True, exist_ok=True)
    got, tries = 0, 0
    while got < n and tries < n * 15:
        tries += 1
        mod_id = random.randint(1, MAX_MOD_ID)
        try:
            result = eat(mod_id)
        except (OSError, ValueError) as e:
            shutil.rmtree(LIBRARY / str(mod_id), ignore_errors=True)
            result = f"failed: {type(e).__name__}"
        if isinstance(result, dict):
            got += 1
            print(f"ate #{mod_id} {result['name']} ({len(result['units'])} .mod)", flush=True)
        else:
            print(f"skip #{mod_id}: {result}", flush=True)
    print(f"library now holds {len(library())} mods")


def crashes_since(start):
    found = []
    for folder in CRASH_DUMPS:
        if folder.exists():
            found += [p.name for p in folder.glob("*.dmp") if p.stat().st_mtime > start
                      and (folder.name == "WATCHDOG" or p.name.lower().startswith("kenshi"))]
    return sorted(found)


def judge(state):
    open_ = state.get("open")
    if not open_:
        return
    saves = [p.stat().st_mtime for p in SAVES.glob("*/*") if p.is_file()] if SAVES.exists() else []
    last = max([s for s in saves if s > open_["started"]], default=open_["started"])
    crashes = crashes_since(open_["started"])
    entry = dict(open_, ended=time.time(), crashes=crashes, played_minutes=round((last - open_["started"]) / 60, 1),
                 verdict="crashed" if crashes else "survived")
    ledger(entry)
    state["open"] = None
    print(f"loadout {open_['loadout']} {entry['verdict']}: {len(crashes)} crash dump(s), "
          f"{entry['played_minutes']} min of saves")


def read_cfg():
    return [l.strip() for l in MODS_CFG.read_text().splitlines() if l.strip()]


def unmix(state):
    for name in state["installed"]:
        shutil.rmtree(KENSHI / "mods" / name, ignore_errors=True)
    if state.get("baseline") is not None:
        MODS_CFG.write_text("\n".join(state["baseline"]) + "\n")
    state["installed"] = []


def backup_save():
    if not SAVES.exists():
        return None
    newest = max((d for d in SAVES.iterdir() if d.is_dir()), key=lambda d: d.stat().st_mtime, default=None)
    if not newest:
        return None
    dest = BACKUPS / f"{time.strftime('%Y%m%d-%H%M%S')}-{newest.name}"
    shutil.copytree(newest, dest)
    for old in sorted(BACKUPS.iterdir())[:-KEEP_BACKUPS]:
        shutil.rmtree(old, ignore_errors=True)
    return dest.name


def fitness():
    stats = {}
    if LEDGER.exists():
        for line in LEDGER.read_text().splitlines():
            entry = json.loads(line)
            for mod in entry["mods"]:
                s = stats.setdefault(mod["mod_id"], {"name": mod["name"], "runs": 0, "crashed": 0, "minutes": 0.0})
                s["runs"] += 1
                s["crashed"] += entry["verdict"] == "crashed"
                s["minutes"] += entry["played_minutes"]
    return stats


def mix(k):
    if kenshi_running():
        raise SystemExit("Kenshi is running; mods.cfg is only read at launch. Close the game first.")
    state = load_state()
    judge(state)
    unmix(state)
    state["baseline"] = read_cfg()
    stats = fitness()
    pool = [m for m in library() if not (KENSHI / "mods" / Path(m["units"][0]).stem).exists()]
    weight = lambda m: (1 + stats.get(m["mod_id"], {}).get("runs", 0)) ** -1 / (1 + 2 * stats.get(m["mod_id"], {}).get("crashed", 0))
    chosen = []
    while pool and len(chosen) < k:
        m = random.choices(pool, weights=[weight(p) for p in pool])[0]
        pool.remove(m)
        chosen.append(m)
    lines, installed = list(state["baseline"]), []
    for m in chosen:
        unit = LIBRARY / str(m["mod_id"]) / "raw" / random.choice(m["units"])
        name = unit.stem
        shutil.copytree(unit.parent, KENSHI / "mods" / name, dirs_exist_ok=True)
        installed.append(name)
        lines.append(f"{name}.mod")
        print(f"mixed in #{m['mod_id']} {m['name']} as {name}.mod")
    MODS_CFG.write_text("\n".join(lines) + "\n")
    state["installed"] = installed
    state["open"] = {"loadout": time.strftime("%Y%m%d-%H%M%S"), "started": time.time(), "save_backup": backup_save(),
                     "baseline": state["baseline"],
                     "mods": [{"mod_id": m["mod_id"], "name": m["name"], "version": m["version"]} for m in chosen]}
    save_state(state)
    print(f"loadout {state['open']['loadout']} ready; save backed up as {state['open']['save_backup']}")


def main():
    HOME.mkdir(parents=True, exist_ok=True)
    cmd = sys.argv[1] if len(sys.argv) > 1 else "status"
    arg = int(sys.argv[2]) if len(sys.argv) > 2 else 3
    if cmd == "pull":
        pull(arg)
    elif cmd == "mix":
        mix(arg)
    elif cmd in ("judge", "clear"):
        if cmd == "clear" and kenshi_running():
            raise SystemExit("Kenshi is running; close it before clearing mixed mods.")
        state = load_state()
        judge(state)
        if cmd == "clear":
            unmix(state)
        save_state(state)
    else:
        state = load_state()
        print(f"library: {len(library())} mods; open loadout: {state['open']['loadout'] if state.get('open') else 'none'}")
        for mod_id, s in sorted(fitness().items(), key=lambda kv: -kv[1]["runs"]):
            print(f"  #{mod_id} {s['name'][:40]:40} runs {s['runs']} crashed {s['crashed']} minutes {s['minutes']:.0f}")


if __name__ == "__main__":
    main()
