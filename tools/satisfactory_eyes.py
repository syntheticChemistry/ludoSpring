"""
satisfactory_eyes.py — Gaia's eyes inside the factory.

Reads Satisfactory save files and extracts factory state:
buildings, conveyors, resources, production chains.
Feeds into the scratch board (scratch-board.jsonl) that
connects to Kenshi through the game triad.

Usage:
    python satisfactory_eyes.py              # read newest save, print summary
    python satisfactory_eyes.py profile      # deep profile of all saves
    python satisfactory_eyes.py watch        # watch mode: poll saves, narrate changes
    python satisfactory_eyes.py board        # extract factory state to scratch board
"""

import sys, os, json, time, glob
from datetime import datetime
from pathlib import Path
from collections import Counter

SAVE_DIR = os.path.join(os.environ.get("LOCALAPPDATA", ""), "FactoryGame", "Saved", "SaveGames")
GAIA_DIR = os.path.join(os.environ.get("LOCALAPPDATA", ""), "Gaia", "satisfactory")
BOARD_PATH = os.path.join(os.environ.get("LOCALAPPDATA", ""), "Gaia", "scratch-board.jsonl")
TRIAD_DIR = os.path.join(os.path.dirname(__file__), "..", "..", "..", "data", "triad")

os.makedirs(GAIA_DIR, exist_ok=True)
os.makedirs(TRIAD_DIR, exist_ok=True)

def find_saves():
    """Find all Satisfactory .sav files."""
    saves = []
    for root, dirs, files in os.walk(SAVE_DIR):
        for f in files:
            if f.endswith(".sav"):
                full = os.path.join(root, f)
                saves.append({
                    "path": full,
                    "name": f[:-4],
                    "size_kb": round(os.path.getsize(full) / 1024, 1),
                    "modified": datetime.fromtimestamp(os.path.getmtime(full)).isoformat(),
                })
    saves.sort(key=lambda s: s["modified"], reverse=True)
    return saves


def read_save(path):
    """Parse a Satisfactory save using the satisfactory-save library."""
    try:
        import satisfactory_save as s
        save = s.SaveGame(path)
        return save
    except Exception as e:
        print(f"[satisfactory_eyes] parse error: {e}")
        return None


def extract_header(save):
    """Extract save header info."""
    h = save.mSaveHeader
    return {
        "session_name": str(getattr(h, "SessionName", "?")),
        "map_name": str(getattr(h, "MapName", "?")),
        "save_version": getattr(h, "SaveVersion", 0),
        "build_version": getattr(h, "BuildVersion", 0),
        "played_seconds": getattr(h, "PlayDurationSeconds", 0),
        "played_hours": round(getattr(h, "PlayDurationSeconds", 0) / 3600, 1),
    }


def extract_objects(save):
    """Extract and classify all save objects."""
    objects = save.mPersistentAndRuntimeData.SaveObjects
    class_counts = Counter()
    buildings = Counter()
    conveyors = Counter()
    machines = Counter()
    resources = Counter()
    vehicles = Counter()
    connections = Counter()
    misc = Counter()

    for obj in objects:
        cn = obj.BaseHeader.ClassName
        class_counts[cn] += 1
        short = cn.split("/")[-1].replace("_C", "").replace("Build_", "").replace("BP_", "")

        if "Conveyor" in cn or "Belt" in cn or "Pipe" in cn:
            conveyors[short] += 1
        elif "Build_" in cn or "Foundation" in cn or "Wall" in cn or "Ramp" in cn:
            buildings[short] += 1
        elif "Miner" in cn or "Smelter" in cn or "Constructor" in cn or \
             "Assembler" in cn or "Manufacturer" in cn or "Refinery" in cn or \
             "Generator" in cn or "Packager" in cn or "OilPump" in cn or "Blender" in cn:
            machines[short] += 1
        elif "Resource" in cn or "Fracking" in cn or "Geyser" in cn or "Deposit" in cn:
            resources[short] += 1
        elif "Vehicle" in cn or "Train" in cn or "Tractor" in cn or "Explorer" in cn:
            vehicles[short] += 1
        elif "Connection" in cn or "Power" in cn or "Inventory" in cn:
            connections[short] += 1
        else:
            misc[short] += 1

    # Also count per-level objects
    level_objects = 0
    level_classes = Counter()
    try:
        for i in range(len(save.mPerLevelDataMap)):
            for obj in save.mPerLevelDataMap.Values[i].SaveObjects:
                level_objects += 1
                cn = obj.BaseHeader.ClassName
                level_classes[cn] += 1
    except:
        pass

    return {
        "total_objects": len(objects),
        "level_objects": level_objects,
        "unique_classes": len(class_counts),
        "building_count": sum(buildings.values()),
        "conveyor_count": sum(conveyors.values()),
        "machine_count": sum(machines.values()),
        "resource_count": sum(resources.values()),
        "vehicle_count": sum(vehicles.values()),
        "connection_count": sum(connections.values()),
        "top_classes": class_counts.most_common(20),
        "machine_types": machines.most_common(20),
        "building_types": buildings.most_common(10),
        "conveyor_types": conveyors.most_common(10),
        "vehicle_types": vehicles.most_common(5),
        "level_top": level_classes.most_common(10),
    }


def extract_levels(save):
    """Extract per-level data."""
    levels = []
    try:
        for i in range(len(save.mPerLevelDataMap)):
            name = str(save.mPerLevelDataMap.Keys[i])
            obj_count = len(save.mPerLevelDataMap.Values[i].SaveObjects)
            levels.append({"name": name, "objects": obj_count})
    except Exception as e:
        levels.append({"error": str(e)})
    return levels


def summarize(path=None):
    """Read a save and print a summary."""
    if path is None:
        saves = find_saves()
        if not saves:
            print("[satisfactory_eyes] no saves found")
            return
        path = saves[0]["path"]
        print(f"[satisfactory_eyes] reading newest save: {saves[0]['name']}")

    save = read_save(path)
    if save is None:
        return

    header = extract_header(save)
    print(f"\n{'='*60}")
    print(f"  SATISFACTORY — {header['session_name']}")
    print(f"  {header['played_hours']} hours played | Build {header['build_version']}")
    print(f"{'='*60}")

    objects = extract_objects(save)
    print(f"\n  Total objects:  {objects['total_objects']}")
    print(f"  Level objects:  {objects['level_objects']}")
    print(f"  Unique classes: {objects['unique_classes']}")
    print(f"  ─────────────────────────")
    print(f"  Buildings:   {objects['building_count']}")
    print(f"  Conveyors:   {objects['conveyor_count']}")
    print(f"  Machines:    {objects['machine_count']}")
    print(f"  Resources:   {objects['resource_count']}")
    print(f"  Vehicles:    {objects['vehicle_count']}")
    print(f"  Connections: {objects['connection_count']}")

    if objects["building_types"]:
        print(f"\n  BUILDINGS:")
        for m, c in objects["building_types"][:10]:
            print(f"    {c:5d}  {m}")

    if objects["conveyor_types"]:
        print(f"\n  CONVEYORS:")
        for m, c in objects["conveyor_types"][:10]:
            print(f"    {c:5d}  {m}")

    if objects["machine_types"]:
        print(f"\n  MACHINES:")
        for m, c in objects["machine_types"][:10]:
            print(f"    {c:5d}  {m}")

    if objects["vehicle_types"]:
        print(f"\n  VEHICLES:")
        for m, c in objects["vehicle_types"][:5]:
            print(f"    {c:5d}  {m}")

    if objects["level_top"]:
        print(f"\n  WORLD (per-level):")
        for cls, count in objects["level_top"][:10]:
            short = cls.split("/")[-1].replace("_C", "").replace("BP_", "")
            print(f"    {count:5d}  {short}")

    levels = extract_levels(save)

    return {"header": header, "objects": objects, "levels": levels}


def profile_all():
    """Profile all saves."""
    saves = find_saves()
    print(f"[satisfactory_eyes] found {len(saves)} saves\n")
    for s in saves:
        print(f"  {s['name']:40s}  {s['size_kb']:8.1f} KB  {s['modified']}")
    print()

    sessions = {}
    for s in saves:
        base = s["name"].split("_autosave")[0].split("_")[0]
        if base not in sessions:
            sessions[base] = []
        sessions[base].append(s)

    print(f"  SESSIONS: {len(sessions)}")
    for name, saves_in in sessions.items():
        total_kb = sum(s["size_kb"] for s in saves_in)
        newest = saves_in[0]["modified"]
        print(f"    {name:30s}  {len(saves_in):2d} saves  {total_kb:8.1f} KB total  newest: {newest}")


def write_to_board(data, source="satisfactory"):
    """Write factory state to the shared scratch board."""
    entry = {
        "ts": datetime.now().isoformat(),
        "source": source,
        "data": data,
    }
    with open(BOARD_PATH, "a") as f:
        f.write(json.dumps(entry) + "\n")
    print(f"[board] wrote to {BOARD_PATH}")


def board_extract(path=None):
    """Extract factory state and write to scratch board."""
    if path is None:
        saves = find_saves()
        if not saves:
            return
        path = saves[0]["path"]

    save = read_save(path)
    if save is None:
        return

    header = extract_header(save)
    objects = extract_objects(save)

    board_data = {
        "game": "satisfactory",
        "session": header["session_name"],
        "hours": header["played_hours"],
        "factory": {
            "total_objects": objects["total_objects"],
            "buildings": objects["building_count"],
            "conveyors": objects["conveyor_count"],
            "machines": objects["machine_count"],
            "top_machines": [{"type": m, "count": c} for m, c in objects["machine_types"][:10]],
        },
    }
    write_to_board(board_data)

    # Also save detailed state
    state_path = os.path.join(GAIA_DIR, "factory-state.json")
    with open(state_path, "w") as f:
        json.dump(board_data, f, indent=2)
    print(f"[satisfactory_eyes] state saved to {state_path}")


def watch_mode():
    """Watch saves for changes and narrate."""
    print("[satisfactory_eyes] WATCH MODE — polling every 30s")
    last_modified = {}

    while True:
        saves = find_saves()
        for s in saves[:5]:
            mod = s["modified"]
            if s["name"] not in last_modified or last_modified[s["name"]] != mod:
                if s["name"] in last_modified:
                    print(f"\n[satisfactory_eyes] SAVE CHANGED: {s['name']}")
                    result = summarize(s["path"])
                    if result:
                        board_extract(s["path"])
                last_modified[s["name"]] = mod
        time.sleep(30)


if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "summary"

    if cmd == "profile":
        profile_all()
    elif cmd == "watch":
        watch_mode()
    elif cmd == "board":
        board_extract()
    else:
        summarize()
