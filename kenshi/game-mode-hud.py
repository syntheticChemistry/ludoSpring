#!/usr/bin/env python3
"""game-mode-hud.py — GAME MODE dashboard for Display 1 (Acer curved).

Serves a live hardware + game HUD at http://localhost:8888
Display 1 (Acer XZ320QU curved) = this dashboard + IDE
Display 2 (MSI MAG flat) = Kenshi game

Stats update every 2 seconds via /stats JSON endpoint.
"""
import http.server
import json
import os
import subprocess
import threading
import time
from pathlib import Path

PORT = 8888
MINDS_PATH = Path(os.environ.get("LOCALAPPDATA", "")) / "Gaia" / "kenshi" / "minds.json"
PLANS_PATH = Path(os.environ.get("LOCALAPPDATA", "")) / "Gaia" / "kenshi" / "squad-plans.jsonl"

def get_stats():
    stats = {"ts": time.time()}

    # GPU via nvidia-smi
    try:
        r = subprocess.run(
            ["nvidia-smi", "--query-gpu=temperature.gpu,utilization.gpu,utilization.memory,memory.used,memory.total,power.draw,fan.speed,clocks.gr,clocks.mem",
             "--format=csv,noheader,nounits"],
            capture_output=True, text=True, timeout=5
        )
        vals = [v.strip() for v in r.stdout.strip().split(",")]
        if len(vals) >= 9:
            stats["gpu"] = {
                "temp": int(vals[0]),
                "util": int(vals[1]),
                "mem_util": int(vals[2]),
                "vram_used": int(vals[3]),
                "vram_total": int(vals[4]),
                "power": float(vals[5]),
                "fan": vals[6],
                "clock_core": int(vals[7]),
                "clock_mem": int(vals[8]),
            }
    except Exception as e:
        stats["gpu_error"] = str(e)

    # RAM via ctypes (instant, no subprocess)
    try:
        import ctypes
        class MEMORYSTATUSEX(ctypes.Structure):
            _fields_ = [
                ("dwLength", ctypes.c_ulong),
                ("dwMemoryLoad", ctypes.c_ulong),
                ("ullTotalPhys", ctypes.c_ulonglong),
                ("ullAvailPhys", ctypes.c_ulonglong),
                ("ullTotalPageFile", ctypes.c_ulonglong),
                ("ullAvailPageFile", ctypes.c_ulonglong),
                ("ullTotalVirtual", ctypes.c_ulonglong),
                ("ullAvailVirtual", ctypes.c_ulonglong),
                ("ullAvailExtendedVirtual", ctypes.c_ulonglong),
            ]
        mem = MEMORYSTATUSEX()
        mem.dwLength = ctypes.sizeof(MEMORYSTATUSEX)
        ctypes.windll.kernel32.GlobalMemoryStatusEx(ctypes.byref(mem))
        total_gb = mem.ullTotalPhys / (1024**3)
        avail_gb = mem.ullAvailPhys / (1024**3)
        stats["ram"] = {
            "total_gb": round(total_gb, 1),
            "used_gb": round(total_gb - avail_gb, 1),
            "free_gb": round(avail_gb, 1),
        }
        stats["cpu_load"] = mem.dwMemoryLoad  # memory load % as proxy
    except:
        pass

    # CPU load via quick snapshot
    try:
        r = subprocess.run(
            ["typeperf", "\\Processor(_Total)\\% Processor Time", "-sc", "1"],
            capture_output=True, text=True, timeout=5
        )
        for line in r.stdout.split("\n"):
            if "," in line and "Processor" not in line and line.strip():
                val = line.strip().split(",")[-1].strip().strip('"')
                try:
                    stats["cpu_load"] = min(100, int(float(val)))
                except:
                    pass
    except:
        pass

    # Kenshi process
    try:
        r = subprocess.run(
            ["tasklist", "/fi", "imagename eq kenshi_x64.exe", "/fo", "csv", "/nh"],
            capture_output=True, text=True, timeout=5
        )
        if "kenshi_x64" in r.stdout:
            parts = r.stdout.strip().split(",")
            mem_str = parts[-1].strip().strip('"').replace(" K", "").replace(",", "")
            stats["kenshi"] = {
                "running": True,
                "pid": parts[1].strip().strip('"'),
                "mem_mb": int(mem_str) // 1024 if mem_str.isdigit() else 0,
            }
        else:
            stats["kenshi"] = {"running": False}
    except:
        stats["kenshi"] = {"running": False}

    # Squad minds
    try:
        minds = json.loads(MINDS_PATH.read_text(encoding="utf-8"))
        squad = []
        for name, m in minds.items():
            v = m.get("voice", {})
            squad.append({
                "name": name,
                "temperament": v.get("temperament", "?"),
                "trust": round(m.get("trust", 0), 1),
                "taps": m.get("taps", 0),
                "last_memory": m.get("memory", [""])[-1] if m.get("memory") else "",
            })
        stats["squad"] = squad
    except:
        stats["squad"] = []

    # Latest squad plan
    try:
        lines = PLANS_PATH.read_text(encoding="utf-8").strip().split("\n")
        if lines:
            stats["last_plan"] = json.loads(lines[-1])
    except:
        pass

    # Thinking squads process
    try:
        r = subprocess.run(
            ["tasklist", "/fi", "imagename eq python.exe", "/fo", "csv", "/nh"],
            capture_output=True, text=True, timeout=5
        )
        stats["thinking_squads_running"] = "thinking_squads" in r.stdout
    except:
        stats["thinking_squads_running"] = False

    return stats


DASHBOARD_HTML = r"""<!DOCTYPE html>
<html><head>
<meta charset="utf-8">
<title>GAIA — GAME MODE</title>
<style>
  * { margin: 0; padding: 0; box-sizing: border-box; }
  body {
    background: #0a0a0f;
    color: #c8d6e5;
    font-family: 'Cascadia Code', 'JetBrains Mono', 'Consolas', monospace;
    font-size: 14px;
    overflow: hidden;
    height: 100vh;
  }
  .grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    grid-template-rows: auto 1fr auto;
    gap: 8px;
    padding: 8px;
    height: 100vh;
  }
  .header {
    grid-column: 1 / -1;
    text-align: center;
    padding: 6px;
    border-bottom: 1px solid #1a1a2e;
  }
  .header h1 {
    font-size: 20px;
    color: #ff6b35;
    letter-spacing: 4px;
    text-transform: uppercase;
  }
  .header .sub {
    font-size: 11px;
    color: #576574;
    margin-top: 2px;
  }
  .panel {
    background: #0d0d15;
    border: 1px solid #1a1a2e;
    border-radius: 6px;
    padding: 10px;
    overflow: hidden;
  }
  .panel h2 {
    font-size: 11px;
    color: #576574;
    text-transform: uppercase;
    letter-spacing: 2px;
    margin-bottom: 8px;
    border-bottom: 1px solid #1a1a2e;
    padding-bottom: 4px;
  }
  .stat-row {
    display: flex;
    justify-content: space-between;
    padding: 3px 0;
    border-bottom: 1px solid #0f0f1a;
  }
  .stat-label { color: #576574; font-size: 12px; }
  .stat-value { color: #c8d6e5; font-size: 13px; font-weight: 500; }
  .stat-value.hot { color: #ff6b6b; }
  .stat-value.warm { color: #ffa726; }
  .stat-value.cool { color: #66bb6a; }
  .stat-value.ice { color: #42a5f5; }

  .bar-container {
    width: 100%;
    height: 6px;
    background: #1a1a2e;
    border-radius: 3px;
    margin-top: 2px;
    overflow: hidden;
  }
  .bar {
    height: 100%;
    border-radius: 3px;
    transition: width 0.5s ease;
  }
  .bar.gpu { background: linear-gradient(90deg, #66bb6a, #ffa726, #ff6b6b); }
  .bar.vram { background: linear-gradient(90deg, #42a5f5, #7e57c2); }
  .bar.cpu { background: linear-gradient(90deg, #26c6da, #ffa726); }
  .bar.ram { background: linear-gradient(90deg, #66bb6a, #42a5f5); }

  .squad-list {
    max-height: calc(100vh - 350px);
    overflow-y: auto;
  }
  .squad-member {
    padding: 6px;
    margin: 4px 0;
    background: #0f0f1a;
    border-radius: 4px;
    border-left: 3px solid #7e57c2;
  }
  .squad-member .name {
    color: #ff6b35;
    font-weight: 500;
    font-size: 13px;
  }
  .squad-member .detail {
    color: #576574;
    font-size: 11px;
    margin-top: 2px;
  }
  .squad-member .memory {
    color: #42a5f5;
    font-size: 11px;
    font-style: italic;
    margin-top: 2px;
  }

  .plan-box {
    background: #0f0f1a;
    border-left: 3px solid #ff6b35;
    padding: 8px;
    margin-top: 6px;
    font-size: 12px;
    color: #8395a7;
    border-radius: 0 4px 4px 0;
    max-height: 120px;
    overflow-y: auto;
  }

  .footer {
    grid-column: 1 / -1;
    text-align: center;
    padding: 4px;
    font-size: 10px;
    color: #2d3436;
  }

  .kenshi-badge {
    display: inline-block;
    padding: 2px 10px;
    border-radius: 10px;
    font-size: 11px;
    font-weight: 500;
  }
  .kenshi-badge.alive { background: #1b5e20; color: #66bb6a; }
  .kenshi-badge.dead { background: #4a1515; color: #ff6b6b; }

  .pulse { animation: pulse 2s ease-in-out infinite; }
  @keyframes pulse { 0%,100% { opacity: 1; } 50% { opacity: 0.5; } }
</style>
</head><body>
<div class="grid">
  <div class="header">
    <h1>⚔ GAIA — GAME MODE ⚔</h1>
    <div class="sub">
      <span id="kenshi-status"></span>
      · RTX 5090 · 9950X3D · <span id="clock"></span>
    </div>
  </div>

  <!-- GPU Panel -->
  <div class="panel">
    <h2>🔥 RTX 5090</h2>
    <div class="stat-row">
      <span class="stat-label">Temperature</span>
      <span class="stat-value" id="gpu-temp">--°C</span>
    </div>
    <div class="stat-row">
      <span class="stat-label">GPU Load</span>
      <span class="stat-value" id="gpu-util">--%</span>
    </div>
    <div class="bar-container"><div class="bar gpu" id="gpu-bar" style="width:0%"></div></div>
    <div class="stat-row">
      <span class="stat-label">VRAM</span>
      <span class="stat-value" id="vram">-- / -- GB</span>
    </div>
    <div class="bar-container"><div class="bar vram" id="vram-bar" style="width:0%"></div></div>
    <div class="stat-row">
      <span class="stat-label">Power</span>
      <span class="stat-value" id="gpu-power">-- W</span>
    </div>
    <div class="stat-row">
      <span class="stat-label">Core / Mem Clock</span>
      <span class="stat-value" id="gpu-clocks">-- / -- MHz</span>
    </div>

    <h2 style="margin-top:12px">💻 System</h2>
    <div class="stat-row">
      <span class="stat-label">CPU (9950X3D)</span>
      <span class="stat-value" id="cpu-load">--%</span>
    </div>
    <div class="bar-container"><div class="bar cpu" id="cpu-bar" style="width:0%"></div></div>
    <div class="stat-row">
      <span class="stat-label">RAM (96 GB DDR5)</span>
      <span class="stat-value" id="ram">-- / -- GB</span>
    </div>
    <div class="bar-container"><div class="bar ram" id="ram-bar" style="width:0%"></div></div>
    <div class="stat-row">
      <span class="stat-label">Kenshi Memory</span>
      <span class="stat-value" id="kenshi-mem">-- MB</span>
    </div>
  </div>

  <!-- Squad Panel -->
  <div class="panel">
    <h2>🗡 Squad — The Nameless</h2>
    <div class="squad-list" id="squad-list">
      <div style="color:#576574">Loading squad...</div>
    </div>

    <h2 style="margin-top:12px">📋 Squad Plan</h2>
    <div class="plan-box" id="plan-box">Waiting for thinking squads...</div>
  </div>

  <div class="footer">
    DISPLAY 1: Acer XZ320QU (curved) — Console · DISPLAY 2: MSI MAG 274Q (flat) — Kenshi HUD
    · <span id="update-ts"></span>
  </div>
</div>

<script>
function tempClass(t) {
  if (t >= 85) return 'hot';
  if (t >= 70) return 'warm';
  if (t >= 50) return 'cool';
  return 'ice';
}

function update() {
  fetch('/stats').then(r => r.json()).then(s => {
    // GPU
    if (s.gpu) {
      const g = s.gpu;
      document.getElementById('gpu-temp').textContent = g.temp + '°C';
      document.getElementById('gpu-temp').className = 'stat-value ' + tempClass(g.temp);
      document.getElementById('gpu-util').textContent = g.util + '%';
      document.getElementById('gpu-bar').style.width = g.util + '%';
      const vramGb = (g.vram_used / 1024).toFixed(1);
      const vramTotalGb = (g.vram_total / 1024).toFixed(1);
      document.getElementById('vram').textContent = vramGb + ' / ' + vramTotalGb + ' GB';
      document.getElementById('vram-bar').style.width = ((g.vram_used / g.vram_total) * 100) + '%';
      document.getElementById('gpu-power').textContent = g.power.toFixed(0) + ' W';
      document.getElementById('gpu-clocks').textContent = g.clock_core + ' / ' + g.clock_mem + ' MHz';
    }
    // CPU
    if (s.cpu_load !== undefined) {
      document.getElementById('cpu-load').textContent = s.cpu_load + '%';
      document.getElementById('cpu-bar').style.width = s.cpu_load + '%';
    }
    // RAM
    if (s.ram) {
      document.getElementById('ram').textContent = s.ram.used_gb + ' / ' + s.ram.total_gb + ' GB';
      document.getElementById('ram-bar').style.width = ((s.ram.used_gb / s.ram.total_gb) * 100) + '%';
    }
    // Kenshi
    if (s.kenshi && s.kenshi.running) {
      document.getElementById('kenshi-status').innerHTML = '<span class="kenshi-badge alive">⚔ KENSHI ALIVE</span>';
      document.getElementById('kenshi-mem').textContent = s.kenshi.mem_mb + ' MB';
    } else {
      document.getElementById('kenshi-status').innerHTML = '<span class="kenshi-badge dead">⚔ KENSHI DOWN</span>';
      document.getElementById('kenshi-mem').textContent = '—';
    }
    // Squad
    if (s.squad && s.squad.length > 0) {
      let html = '';
      for (const m of s.squad) {
        const trustStars = '★'.repeat(Math.floor(m.trust)) + '☆'.repeat(5 - Math.floor(m.trust));
        html += '<div class="squad-member">';
        html += '<div class="name">' + m.name + ' <span style="color:#576574;font-size:11px">' + trustStars + '</span></div>';
        html += '<div class="detail">' + m.temperament + ' · ' + m.taps + ' taps</div>';
        if (m.last_memory) html += '<div class="memory">"' + m.last_memory + '"</div>';
        html += '</div>';
      }
      document.getElementById('squad-list').innerHTML = html;
    }
    // Plan
    if (s.last_plan) {
      document.getElementById('plan-box').textContent = s.last_plan.plan || 'No plan yet';
    }
    // Clock
    document.getElementById('clock').textContent = new Date().toLocaleTimeString();
    document.getElementById('update-ts').textContent = 'Updated: ' + new Date().toLocaleTimeString();
  }).catch(e => {
    document.getElementById('update-ts').textContent = 'Error: ' + e;
  });
}

update();
setInterval(update, 2000);
</script>
</body></html>
"""


class HUDHandler(http.server.BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        pass

    def do_GET(self):
        if self.path == "/stats":
            data = get_stats()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Access-Control-Allow-Origin", "*")
            self.end_headers()
            self.wfile.write(json.dumps(data).encode())
        elif self.path == "/" or self.path == "/hud":
            self.send_response(200)
            self.send_header("Content-Type", "text/html")
            self.end_headers()
            self.wfile.write(DASHBOARD_HTML.encode())
        else:
            self.send_response(404)
            self.end_headers()


def main():
    print("=" * 50)
    print("  GAIA GAME MODE — HUD Server")
    print(f"  http://localhost:{PORT}")
    print("  Display 1 (Acer curved) → this dashboard")
    print("  Display 2 (MSI flat) → Kenshi")
    print("=" * 50)
    server = http.server.HTTPServer(("127.0.0.1", PORT), HUDHandler)
    server.serve_forever()


if __name__ == "__main__":
    main()
