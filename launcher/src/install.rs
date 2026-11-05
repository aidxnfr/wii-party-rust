//! Install / update: unpack the source, fetch pinned tools, recompile the user's disc, build the game DLL.
//! Every step can be re-run; finished work is detected and skipped.
use crate::paths::Layout;
use crate::tools::{Pack, Tool, RUST_TOOLCHAIN, TOOLS};
use sha2::{Digest, Sha256};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;

pub enum Event { Step(usize, usize, String), Line(String), Done(Result<(), String>) }

pub struct Options { pub root: PathBuf, pub disc: Option<PathBuf>, pub jobs: usize, pub offline: bool }

pub struct Ctx<'a> { pub l: Layout, pub o: &'a Options, pub tx: Sender<Event> }
impl Ctx<'_> {
    pub fn line(&self, s: impl Into<String>) { let _ = self.tx.send(Event::Line(s.into())); }
}

type R = Result<(), String>;
fn e<T: std::fmt::Display>(what: &str) -> impl FnOnce(T) -> String + '_ { move |err| format!("{what}: {err}") }

pub const STEPS: &[&str] = &["Unpacking the project source", "Getting the build tools", "Recompiling your disc",
                             "Building the game (this is the long part)", "Finishing up"];

pub fn run(o: &Options, tx: Sender<Event>) {
    let c = Ctx { l: Layout::new(&o.root), o, tx: tx.clone() };
    let r = (|| -> R {
        std::fs::create_dir_all(&o.root).map_err(e("create install folder"))?;
        let steps: [fn(&Ctx) -> R; 5] = [unpack_source, get_tools, recompile, build_game, finish];
        for (i, step) in steps.iter().enumerate() {
            let _ = tx.send(Event::Step(i + 1, steps.len(), STEPS[i].to_string()));
            step(&c)?;
        }
        Ok(())
    })();
    let _ = tx.send(Event::Done(r));
}

// ---------- 1. source ----------
fn unpack_source(c: &Ctx) -> R {
    let src = c.l.source();
    // keep generated/ (your recompiled game code) and build/ (link helpers); replace everything else
    if src.exists() {
        for entry in std::fs::read_dir(&src).map_err(e("read source"))? {
            let p = entry.map_err(e("read source"))?.path();
            let name = p.file_name().unwrap().to_string_lossy().to_string();
            if name == "generated" || name == "build" { continue; }
            if p.is_dir() { std::fs::remove_dir_all(&p) } else { std::fs::remove_file(&p) }.map_err(e("clean old source"))?;
        }
    }
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(crate::payload::SOURCE)).map_err(e("open embedded source"))?;
    zip.extract(&src).map_err(e("unpack source"))?;
    c.line(format!("source {} unpacked to {}", crate::payload::REVISION, src.display()));
    Ok(())
}

// ---------- 2. tools ----------
fn sha256_file(p: &Path) -> std::io::Result<String> {
    let mut f = std::fs::File::open(p)?; let mut h = Sha256::new(); let mut buf = vec![0u8; 1 << 20];
    loop { let n = f.read(&mut buf)?; if n == 0 { break; } h.update(&buf[..n]); }
    Ok(h.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

fn download(c: &Ctx, t: &Tool) -> Result<PathBuf, String> {
    let dir = c.l.downloads(); std::fs::create_dir_all(&dir).map_err(e("downloads folder"))?;
    let path = dir.join(t.file);
    if path.exists() && sha256_file(&path).ok().as_deref() == Some(t.sha256) { return Ok(path); }
    if c.o.offline { return Err(format!("offline mode: put {} into {}", t.file, dir.display())); }
    c.line(format!("downloading {} ...", t.name));
    let resp = ureq::get(t.url).call().map_err(e(t.url))?;
    let total: Option<u64> = resp.header("Content-Length").and_then(|v| v.parse().ok());
    let mut reader = resp.into_reader();
    let tmp = path.with_extension("part");
    let mut f = std::fs::File::create(&tmp).map_err(e("write download"))?;
    let (mut got, mut next) = (0u64, 0u64); let mut buf = vec![0u8; 1 << 16];
    loop {
        let n = reader.read(&mut buf).map_err(e(t.url))?; if n == 0 { break; }
        f.write_all(&buf[..n]).map_err(e("write download"))?; got += n as u64;
        if got >= next { next = got + (16 << 20);
            c.line(match total { Some(t2) => format!("  {} MB of {} MB", got >> 20, t2 >> 20), None => format!("  {} MB", got >> 20) }); }
    }
    drop(f);
    let sum = sha256_file(&tmp).map_err(e("hash download"))?;
    if sum != t.sha256 { let _ = std::fs::remove_file(&tmp); return Err(format!("{} failed its SHA-256 check (got {sum})", t.name)); }
    std::fs::rename(&tmp, &path).map_err(e("store download"))?;
    Ok(path)
}

fn get_tools(c: &Ctx) -> R {
    for t in TOOLS {
        let dir = c.l.tool(t.key); let ok = dir.join(".installed");
        if std::fs::read_to_string(&ok).ok().as_deref() == Some(t.sha256) { continue; }
        let file = download(c, t)?;
        if dir.exists() { std::fs::remove_dir_all(&dir).map_err(e("clean tool folder"))?; }
        std::fs::create_dir_all(&dir).map_err(e("tool folder"))?;
        c.line(format!("installing {} ...", t.name));
        match t.pack {
            Pack::Zip | Pack::Wheel => {
                let f = std::fs::File::open(&file).map_err(e("open archive"))?;
                zip::ZipArchive::new(f).and_then(|mut z| z.extract(&dir)).map_err(e(t.file))?;
            }
            Pack::TarGz => {
                let f = std::fs::File::open(&file).map_err(e("open archive"))?;
                tar::Archive::new(flate2::read::GzDecoder::new(f)).unpack(&dir).map_err(e(t.file))?;
            }
            Pack::SelfExtracting => {
                let mut o = std::ffi::OsString::from("-o"); o.push(dir.as_os_str());
                run_cmd(c, Command::new(&file).arg("-y").arg(o), "unpack MinGW")?;
            }
            Pack::Single => { std::fs::copy(&file, dir.join(t.file)).map_err(e("copy tool"))?; }
        }
        if t.key == "python" {
            // embedded Python ignores PYTHONPATH while a ._pth file exists
            for entry in std::fs::read_dir(&dir).map_err(e("python folder"))?.flatten() {
                if entry.path().extension().is_some_and(|x| x == "_pth") { let _ = std::fs::remove_file(entry.path()); }
            }
        }
        if t.key == "rustup" { install_rust(c)?; }
        std::fs::write(&ok, t.sha256).map_err(e("mark tool"))?;
    }
    Ok(())
}

fn install_rust(c: &Ctx) -> R {
    let mut cmd = Command::new(c.l.tool("rustup").join("rustup-init.exe"));
    cmd.args(["-y", "--no-modify-path", "--profile", "minimal", "--default-host", "x86_64-pc-windows-gnu", "--default-toolchain", RUST_TOOLCHAIN])
        .env("RUSTUP_HOME", c.l.rustup_home()).env("CARGO_HOME", c.l.cargo_home());
    run_cmd(c, &mut cmd, "install the Rust toolchain")
}

// ---------- 3. recompile ----------
fn recompile(c: &Ctx) -> R {
    let mut cmd = Command::new(c.l.python());
    cmd.arg(c.l.source().join("recompiler").join("pipeline.py"))
        .arg("--nodtool").arg(c.l.tool("nodtool").join("nodtool.exe"))
        .arg("--out").arg(c.l.generated())
        .env("PYTHONPATH", c.l.tool("pycparser")).env("PYTHONUTF8", "1").current_dir(c.l.source());
    match &c.o.disc {
        Some(d) => { cmd.arg("--disc").arg(d); }
        None if c.l.extracted().join("sys").join("main.dol").exists() => {}
        None => return Err("choose your Wii Party disc image first".into()),
    }
    run_cmd(c, &mut cmd, "recompile the disc")
}

// ---------- 4. build ----------
fn find_file(dir: &Path, name: &str) -> Option<PathBuf> {
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let p = entry.path();
        if p.is_dir() { if let Some(f) = find_file(&p, name) { return Some(f); } }
        else if p.file_name().is_some_and(|n| n == name) { return Some(p); }
    }
    None
}

fn build_game(c: &Ctx) -> R {
    let src = c.l.source();
    // nuwen's MinGW ships libgcc.a but no libgcc_eh.a; the Rust GNU target asks for it by that name
    let compat = src.join("build").join("link-compat");
    std::fs::create_dir_all(&compat).map_err(e("link-compat"))?;
    let libgcc = find_file(&c.l.tool("mingw"), "libgcc.a").ok_or("MinGW libgcc.a not found")?;
    std::fs::copy(&libgcc, compat.join("libgcc_eh.a")).map_err(e("link-compat"))?;
    let features: Vec<String> = std::fs::read_to_string(src.join("crates/game/features.txt")).map_err(e("features.txt"))?
        .lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')).map(String::from).collect();
    let path = std::env::join_paths([c.l.mingw_bin(), c.l.cargo_home().join("bin")].into_iter()
        .chain(std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()))).map_err(e("PATH"))?;
    let mut cmd = Command::new(c.l.cargo_home().join("bin").join("cargo.exe"));
    cmd.arg("build").arg("--release").arg("-j").arg(c.o.jobs.max(1).to_string())
        .args(["--target", "x86_64-pc-windows-gnu", "--manifest-path", "crates/game/Cargo.toml", "--features"]).arg(features.join(","))
        .current_dir(&src).env("PATH", path)
        .env("RUSTUP_HOME", c.l.rustup_home()).env("CARGO_HOME", c.l.cargo_home()).env("RUSTUP_TOOLCHAIN", RUST_TOOLCHAIN)
        .env("CARGO_TARGET_DIR", c.l.target()).env("CARGO_INCREMENTAL", "0")
        .env("RUSTC_BOOTSTRAP", "wprt_final_unwind")
        .env("WP_GENERATED_DIR", c.l.generated()).env("WP_SDL3_DIR", c.l.sdl3())
        .env("CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER", c.l.mingw_bin().join("g++.exe"))
        .env("CARGO_TARGET_X86_64_PC_WINDOWS_GNU_AR", c.l.mingw_bin().join("ar.exe"));
    if c.o.offline { cmd.arg("--offline"); }
    run_cmd(c, &mut cmd, "build the game")
}

// ---------- 5. finish ----------
fn finish(c: &Ctx) -> R {
    let game = c.l.game(); std::fs::create_dir_all(&game).map_err(e("game folder"))?;
    let dll = c.l.target().join("x86_64-pc-windows-gnu").join("release").join("wiiparty_game.dll");
    copy_staged(&dll, &c.l.game_dll())?;
    if crate::payload::HOST_EXE.is_empty() { return Err("this launcher was built without the game host (WP_HOST_EXE); see docs/BUILDING.md".into()); }
    std::fs::write(game.join("wii-party.exe.new"), crate::payload::HOST_EXE).map_err(e("write game exe"))?;
    std::fs::rename(game.join("wii-party.exe.new"), c.l.host_exe()).map_err(e("replace game exe (is the game still running?)"))?;
    copy_staged(&c.l.sdl3().join("bin").join("SDL3.dll"), &game.join("SDL3.dll"))?;
    // optional performance window
    let stats = c.l.source().join("tools").join("stats").join("stats.cpp");
    let mut cmd = Command::new(c.l.mingw_bin().join("g++.exe"));
    cmd.args(["-O2", "-municode", "-mwindows", "-static"]).arg(&stats).arg("-o").arg(game.join("stats.exe"))
        .args(["-lgdi32", "-luser32", "-lpsapi", "-lshell32"]);
    if run_cmd(c, &mut cmd, "build the stats window").is_err() { c.line("(stats window skipped)"); }
    std::fs::create_dir_all(c.l.data()).map_err(e("data folder"))?;
    let stamp = serde_json::json!({ "revision": crate::payload::REVISION, "disc": c.o.disc });
    std::fs::write(c.l.stamp(), serde_json::to_string_pretty(&stamp).unwrap()).map_err(e("write install record"))?;
    c.line("Wii Party is ready. Press Play.");
    Ok(())
}

fn copy_staged(from: &Path, to: &Path) -> R {
    let tmp = to.with_extension("new");
    std::fs::copy(from, &tmp).map_err(|err| format!("copy {}: {err}", from.display()))?;
    std::fs::rename(&tmp, to).map_err(|err| format!("replace {} (is the game still running?): {err}", to.display()))
}

pub fn run_cmd(c: &Ctx, cmd: &mut Command, what: &str) -> R {
    c.line(format!("> {:?}", cmd));
    #[cfg(windows)] { use std::os::windows::process::CommandExt; cmd.creation_flags(0x0800_0000); } // no console window
    let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().map_err(|err| format!("{what}: {err}"))?;
    let (out, err) = (child.stdout.take().unwrap(), child.stderr.take().unwrap());
    let tx2 = c.tx.clone();
    let t = std::thread::spawn(move || for l in BufReader::new(err).lines().map_while(Result::ok) { let _ = tx2.send(Event::Line(l)); });
    for l in BufReader::new(out).lines().map_while(Result::ok) { c.line(l); }
    let _ = t.join();
    let status = child.wait().map_err(|err| format!("{what}: {err}"))?;
    if status.success() { Ok(()) } else { Err(format!("could not {what} (exit code {:?}); see the log above", status.code())) }
}
