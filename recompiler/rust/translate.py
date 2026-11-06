#!/usr/bin/env python3
"""Translate the upstream recompiler's generated C++ (DOL + every REL module) into the Rust game crates.

usage: translate.py <upstream_generated_dir> <out_dir>

<upstream_generated_dir> is upstream's build/wiiparty/generated (dol/, modules/<name>/).
<out_dir> receives (all rewritten every run, never committed):
  game-code/dol/                one crate (wpgame-dol) with every DOL unit
  game-code/mods/<name>/        one crate per REL module (wpgame-mod-<name>)
  game-code-link/               wpgame-all-link: re-exports dol + modules for the game DLL
  all_modules.rs                module descriptor table included by crates/game
Function bodies come from cpp2rs.py unchanged; only crate packaging is done here.
"""
import os, re, subprocess, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
CPP2RS = HERE / "cpp2rs.py"
WPRT = "../../../crates/wprt"  # relative to a crate two levels below <out_dir>


def write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8", newline="\n")


class Progress:
    """Prints `@@progress` lines for the launcher, by share of C++ bytes translated (WP_PROGRESS_RANGE="lo hi")."""
    def __init__(self, total: int):
        r = os.environ.get("WP_PROGRESS_RANGE")
        self.lo, self.hi = (float(x) for x in r.split()) if r else (None, None)
        self.total, self.done = max(total, 1), 0

    def add(self, n: int) -> None:
        self.done += n
        if self.lo is not None:
            print(f"@@progress {self.lo + (self.hi - self.lo) * self.done / self.total:.4f}", flush=True)


def run_cpp2rs(module: str, dst: Path, srcs: list, header: list, desc=None) -> None:
    dst.parent.mkdir(parents=True, exist_ok=True)  # cpp2rs writes the file but does not create its folder
    args = [sys.executable, str(CPP2RS)] + (["--module-cpp", str(desc)] if desc else []) + [module, str(dst)] + [str(s) for s in srcs]
    r = subprocess.run(args, capture_output=True, text=True)
    last = (r.stderr.strip().splitlines() or ["(no output)"])[-1]
    print(f"{dst.name}: {last}", flush=True)
    if r.returncode != 0 or not last.endswith(" 0 failed"):
        sys.exit(f"translation failed for {module}: {last}")
    lines = dst.read_text(encoding="utf-8").split("\n")
    lines[1:4] = header
    write(dst, "\n".join(lines))


def crate_toml(name: str, deps: str) -> str:
    return f"""[package]
name = "{name}"
version = "0.1.0"
edition = "2021"
license = "GPL-3.0-or-later"
publish = false
# Generated locally from your own disc by recompiler/rust/translate.py. Never commit or upload.

[dependencies]
{deps}"""


def main() -> int:
    gen, out = Path(sys.argv[1]), Path(sys.argv[2])
    code = out / "game-code"
    # --- DOL ---
    dd = gen / "dol"
    parts = sorted(f for f in os.listdir(dd) if re.match(r"dol_\d+\.cpp$", f))
    dol_src = code / "dol" / "src" / "dol"
    mod_rs = ["// GENERATED (Claude). PRIVATE. The main executable (DOL)."]
    modules = sorted(p.name for p in (gen / "modules").iterdir() if p.is_dir())
    module_srcs = {m: sorted(gen / "modules" / m / f for f in os.listdir(gen / "modules" / m) if re.match(rf"{m}_\d+\.cpp$", f)) for m in modules}
    bar = Progress(sum((dd / f).stat().st_size for f in parts) + sum(p.stat().st_size for s in module_srcs.values() for p in s))
    for f in parts:
        n = f[:-4]
        run_cpp2rs("dol", dol_src / f"{n}.rs", [dd / f], ["use wprt::wp::{self, Cpu};", "#[allow(unused_imports)] use super::*;", ""])
        bar.add((dd / f).stat().st_size)
        mod_rs += [f"mod {n};", f"pub use {n}::*;"]
    ft = (dd / "function_table.cpp").read_text(errors="replace")

    def table(name):
        b = ft.index(f"{name}[] = {{"); e = ft.index("};", b)
        return re.findall(r"\{(0x[0-9a-f]+)u, (f_\w+)\}", ft[b:e])
    mod_rs += ["pub static FUNCTIONS: &[(u32, wprt::runtime::GuestFn)] = &["] + [f"    ({a}, {f})," for a, f in table("g_function_table")] + ["];"]
    mod_rs += ["pub static RESUMES: &[(u32, wprt::runtime::GuestFn)] = &["] + [f"    ({a}, {f})," for a, f in table("g_resume_table")] + ["];"]
    nt = (dd / "names.cpp").read_text(errors="replace")
    mod_rs += ["pub static NAMES: &[(u32, &str)] = &["] + [f'    ({a}, "{s}"),' for a, s in re.findall(r'\{(0x[0-9a-f]+)u, "([^"]*)"\}', nt)] + ["];"]
    write(dol_src / "mod.rs", "\n".join(mod_rs) + "\n")
    write(code / "dol" / "src" / "lib.rs", "pub mod dol;\n")
    write(code / "dol" / "Cargo.toml", crate_toml("wpgame-dol", f'wprt = {{ path = "{WPRT}" }}\n'))
    # --- REL modules ---
    for m in modules:
        md = gen / "modules" / m
        srcs = module_srcs[m]
        dst = code / "mods" / m / "src" / "lib.rs"
        run_cpp2rs(m, dst, srcs, ["use wprt::wp::{self, Cpu};", "#[allow(unused_imports)] use wpgame_dol::dol::*;", ""], md / f"{m}_module.cpp")
        bar.add(sum(p.stat().st_size for p in srcs))
        write(code / "mods" / m / "Cargo.toml", crate_toml(f"wpgame-mod-{m}", f'wprt = {{ path = "../{WPRT}" }}\nwpgame-dol = {{ path = "../../dol" }}\n'))
    # --- link crate + descriptor table ---
    deps = [f'wprt = {{ path = "../../crates/wprt" }}', 'wpgame-dol = { path = "../game-code/dol" }'] + \
           [f'wpgame-mod-{m} = {{ path = "../game-code/mods/{m}" }}' for m in modules]
    write(out / "game-code-link" / "Cargo.toml", crate_toml("wpgame-all-link", "\n".join(deps) + "\n"))
    lib = ["#![allow(unused_parens, arithmetic_overflow)]", "// GENERATED. Wii Party guest code translated from the recomp's generated C++.",
           "pub use wpgame_dol::dol;", "", "pub mod modules {"] + [f"    pub use wpgame_mod_{m} as {m};" for m in modules] + ["}", ""]
    write(out / "game-code-link" / "src" / "lib.rs", "\n".join(lib))
    entries = [f'ModuleDescriptor{{name:"{m}",identifier:wpgame_all::modules::{m}::MODULE_ID,signature:wpgame_all::modules::{m}::SIGNATURE,'
               f'section_count:wpgame_all::modules::{m}::SECTIONS,functions:wpgame_all::modules::{m}::FUNCTIONS,resumes:wpgame_all::modules::{m}::RESUMES}},'
               for m in modules]
    write(out / "all_modules.rs", "// GENERATED: module descriptors, sorted by module name like the upstream module table.\n"
          f"static MODULES:[ModuleDescriptor;{len(modules)}]=[\n" + "\n".join(entries) + "\n];\n")
    print(f"translated {len(parts)} DOL units and {len(modules)} modules")
    return 0


if __name__ == "__main__":
    sys.exit(main())
