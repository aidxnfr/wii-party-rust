#!/usr/bin/env python3
"""Disc image -> generated Rust game code. Everything is written under --out (default: generated/), never committed.

  python recompiler/pipeline.py --disc <your Wii Party image> [--nodtool <path>] [--out generated]

Steps (each skipped when its output is already up to date):
  1. upstream   download arelkair/wiiparty-recomp at the commit pinned in recompiler/upstream.toml
  2. extract    nodtool extracts your disc into the upstream checkout (games/wiiparty/extracted)
  3. recompile  upstream scripts: unpack_rels, recomp, recomp_rel --all, recomp, dsp/recomp_dsp  (C++ output)
  4. translate  recompiler/rust/translate.py: C++ -> Rust crates (game-code/, game-code-link/, all_modules.rs)
  5. dsp        recompiler/dsp/build_ax.py: AX audio microcode -> generated/dsp-ax
Requires Python 3.11+ and pycparser (recompiler/requirements.txt).
"""
import argparse, hashlib, io, json, os, shutil, subprocess, sys, tomllib, urllib.request, zipfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parent


def log(msg: str) -> None:
    print(f"[pipeline] {msg}", flush=True)


def run(cmd, cwd=None, env=None) -> None:
    log(" ".join(str(c) for c in cmd))
    r = subprocess.run([str(c) for c in cmd], cwd=cwd, env=env)
    if r.returncode != 0:
        sys.exit(f"step failed ({r.returncode}): {' '.join(str(c) for c in cmd)}")


def tree_hash(paths) -> str:
    h = hashlib.sha256()
    for p in sorted(paths):
        h.update(p.relative_to(REPO).as_posix().encode()); h.update(p.read_bytes())
    return h.hexdigest()


def fetch_upstream(cfg: dict, dst: Path) -> None:
    stamp = dst / ".wp-upstream-commit"
    if stamp.exists() and stamp.read_text().strip() == cfg["commit"]:
        return
    log(f"downloading upstream {cfg['repository']} @ {cfg['commit'][:12]}")
    if shutil.which("git"):
        if dst.exists(): shutil.rmtree(dst)
        run(["git", "clone", "--quiet", cfg["repository"], dst])
        run(["git", "-C", dst, "checkout", "--quiet", cfg["commit"]])
        head = subprocess.run(["git", "-C", str(dst), "rev-parse", "HEAD"], capture_output=True, text=True).stdout.strip()
        if head != cfg["commit"]: sys.exit(f"upstream checkout is {head}, expected {cfg['commit']}")
    else:
        url = cfg["repository"].replace("https://github.com/", "https://codeload.github.com/") + "/zip/" + cfg["commit"]
        data = urllib.request.urlopen(url, timeout=120).read()
        with zipfile.ZipFile(io.BytesIO(data)) as z:
            top = z.namelist()[0].split("/")[0]
            if not top.endswith(cfg["commit"]): sys.exit(f"unexpected archive root {top}")
            if dst.exists(): shutil.rmtree(dst)
            tmp = dst.parent / (dst.name + ".tmp"); shutil.rmtree(tmp, ignore_errors=True); z.extractall(tmp)
            (tmp / top).rename(dst); shutil.rmtree(tmp, ignore_errors=True)
    stamp.write_text(cfg["commit"] + "\n")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--disc", type=Path, help="your Wii Party disc image (.iso/.wbfs/.rvz/.wia/.ciso/.gcm)")
    ap.add_argument("--extracted", type=Path, help="an already extracted disc folder (instead of --disc)")
    ap.add_argument("--nodtool", default="nodtool", help="nodtool executable (https://github.com/encounter/nod)")
    ap.add_argument("--out", type=Path, default=REPO / "generated")
    ap.add_argument("--force", action="store_true", help="redo every step")
    a = ap.parse_args()
    cfg = tomllib.loads((HERE / "upstream.toml").read_text(encoding="utf-8"))
    out = a.out.resolve(); out.mkdir(parents=True, exist_ok=True)
    up = out / "upstream"
    fetch_upstream(cfg, up)
    game = up / "games" / cfg["game"]
    extracted = game / "extracted"
    if a.extracted:
        if extracted.is_symlink() or extracted.exists():
            if extracted.is_symlink(): extracted.unlink()
        try: extracted.symlink_to(a.extracted.resolve(), target_is_directory=True)
        except OSError: shutil.copytree(a.extracted, extracted, dirs_exist_ok=True)
    elif not (extracted / "sys" / "main.dol").exists() or a.force:
        if not a.disc: sys.exit("give --disc <image> (or --extracted <folder>)")
        run([a.nodtool, "extract", a.disc, extracted])
    boot = (extracted / "sys" / "boot.bin").read_bytes()[:6].decode("ascii", "replace")
    want = tomllib.loads((game / "game.toml").read_text(encoding="utf-8"))["id"]
    if boot[:len(want)] != want:
        sys.exit(f"this disc is {boot}, not Wii Party ({want})")
    log(f"disc {boot} OK")
    # 3. upstream recompiler (C++)
    cpp = up / "build" / cfg["game"] / "generated"
    marker = out / ".wp-recompiled"
    if a.force or not marker.exists() or marker.read_text().strip() != cfg["commit"]:
        env = dict(os.environ, WP_GAME=cfg["game"])
        for step in (["recompiler/unpack_rels.py"], ["recompiler/recomp.py"], ["recompiler/recomp_rel.py", "--all"],
                     ["recompiler/recomp.py"], ["recompiler/dsp/recomp_dsp.py"]):
            run([sys.executable] + step, cwd=up, env=env)
        marker.write_text(cfg["commit"] + "\n")
    # 4-5. Rust translation; redone when the C++ or our translator changes
    ours = list((HERE / "rust").glob("*.py")) + list((HERE / "dsp").rglob("*.py")) + list((HERE / "dsp" / "runtime").glob("*.rs"))
    key = json.dumps({"upstream": cfg["commit"], "translator": tree_hash(ours)})
    stamp = out / ".wp-translated"
    if a.force or not stamp.exists() or stamp.read_text() != key:
        for d in ("game-code", "game-code-link", "dsp-ax"):
            shutil.rmtree(out / d, ignore_errors=True)
        run([sys.executable, HERE / "rust" / "translate.py", cpp, out])
        run([sys.executable, HERE / "dsp" / "build_ax.py", up, cpp / "dsp" / "ax.cpp", out / "dsp-ax"])
        stamp.write_text(key)
    log(f"generated code ready in {out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
