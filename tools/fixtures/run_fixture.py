#!/usr/bin/env python3
"""Run one deterministic accuracy fixture and compare it with a reference ("gold") run.

  python tools/fixtures/run_fixture.py --install <install folder> --script tests/fixtures/F23-boardisland.txt \
      --frames 19500 --name F23-mine [--gold <folder with frames.txt + frameNNN.png>] [--env WP_INTERP=2]

The game runs headless with a scripted controller and a frame-counting clock, so every run of the same build
produces the same per-frame state hashes (frames.txt) and the same PNG every 60 frames. A change is exact when
frames.txt and every PNG are byte-identical to the gold run. See docs/TESTING.md.
"""
import argparse, hashlib, os, shutil, subprocess, sys, time
from pathlib import Path


def sha(p: Path) -> str:
    return hashlib.sha256(p.read_bytes()).hexdigest()


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--install", type=Path, required=True, help="launcher install folder (has game/ and source/)")
    ap.add_argument("--script", type=Path, required=True)
    ap.add_argument("--frames", type=int, required=True)
    ap.add_argument("--name", required=True)
    ap.add_argument("--gold", type=Path)
    ap.add_argument("--nand", type=Path, help="NAND folder to start from (default: a fresh one)")
    ap.add_argument("--out", type=Path, default=Path("fixture-runs"))
    ap.add_argument("--env", action="append", default=[], help="extra KEY=VALUE for the game")
    a = ap.parse_args()
    game = a.install / "game"
    extracted = a.install / "source" / "generated" / "upstream" / "games" / "wiiparty" / "extracted"
    run = (a.out / a.name).resolve(); obs = run / "observe"
    if run.exists(): sys.exit(f"{run} already exists")
    nand = run / "games" / "wiiparty" / "nand"
    if a.nand: shutil.copytree(a.nand, nand)
    else: nand.mkdir(parents=True)
    obs.mkdir(parents=True)
    env = dict(os.environ)
    for k in ("WP_DSP_SELFTEST", "WP_DSP_VERIFY", "WP_DUMP_AUDIO", "WP_DUMP_OUTPUT"): env.pop(k, None)
    env.update({
        "PATH": str(game) + os.pathsep + env.get("PATH", ""), "WGPU_BACKEND": "dx12",
        "WP_OFFSCREEN": "1", "WP_HOST_INPUT": "1", "WP_HEADLESS": "1", "WP_MUTE": "1", "WP_GAMEPAD": "0", "WP_REAL_WIIMOTES": "0",
        "WP_DETERMINISTIC": "1", "WP_GPU_THREAD": "0", "WP_SCRIPT_CLOCK": "frames", "WP_INPUT_SCRIPT": "@" + str(a.script.resolve()),
        "WP_COMPARE_LOG": str(obs / "frames.txt"), "WP_COMPARE_MAX_FRAMES": str(a.frames),
        "WP_SAVE_FRAME": str(obs / "frame%03d.png"), "WP_SAVE_EVERY": "60", "WP_LOG_FILE": str(obs / "runtime.log"),
        "WP_HOST_RUN_SECONDS": "7200", "WII_PARTY_PRIVATE_ROOT": str(run / "private"), "WII_PARTY_SDL3": str(game / "SDL3.dll"),
    })
    for kv in a.env:
        k, v = kv.split("=", 1); env[k] = v
    t0 = time.time()
    with open(obs / "stdout.txt", "wb") as out, open(obs / "stderr.txt", "wb") as err:
        r = subprocess.run([str(game / "wii-party.exe"), str(game / "wiiparty_game.dll"), str(extracted), str(nand)],
                           cwd=run, env=env, stdout=out, stderr=err, timeout=7800)
    print(f"{a.name}: exit {r.returncode} after {time.time() - t0:.0f} s")
    if not a.gold: return r.returncode
    rows = "ROWS_SAME" if sha(a.gold / "frames.txt") == sha(obs / "frames.txt") else "ROWS_DIFF"
    pngs = sorted(a.gold.glob("*.png")); diff = sum(1 for p in pngs if not (obs / p.name).exists() or sha(p) != sha(obs / p.name))
    print(f"RESULT {a.name} {rows} png {len(pngs)} diff {diff}")
    return 0 if rows == "ROWS_SAME" and diff == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
