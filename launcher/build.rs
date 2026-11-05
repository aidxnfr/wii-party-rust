//! Embeds the project source (as source.zip) and the prebuilt game host (WP_HOST_EXE) into the launcher.
use std::io::Write;
use std::path::{Path, PathBuf};

const INCLUDE: &[&str] = &[".cargo", "crates", "recompiler", "third_party", "tools", "README.md", "LICENSE", "THIRD_PARTY_NOTICES.md", "rust-toolchain.toml"];
const SKIP_DIRS: &[&str] = &["target", "generated", "build", "__pycache__", ".git"];

fn add(zip: &mut zip::ZipWriter<std::fs::File>, root: &Path, path: &Path) {
    let rel = path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/");
    if path.is_dir() {
        if SKIP_DIRS.contains(&path.file_name().unwrap().to_str().unwrap()) { return; }
        let mut entries: Vec<_> = std::fs::read_dir(path).unwrap().flatten().map(|e| e.path()).collect();
        entries.sort();
        for e in entries { add(zip, root, &e); }
    } else {
        println!("cargo:rerun-if-changed={}", path.display());
        let opts = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        zip.start_file(rel, opts).unwrap();
        zip.write_all(&std::fs::read(path).unwrap()).unwrap();
    }
}

fn main() {
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("..");
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let mut zip = zip::ZipWriter::new(std::fs::File::create(out.join("source.zip")).unwrap());
    for name in INCLUDE {
        let p = root.join(name);
        if p.exists() { add(&mut zip, &root, &p); }
    }
    zip.finish().unwrap();
    println!("cargo:rerun-if-env-changed=WP_HOST_EXE");
    match std::env::var_os("WP_HOST_EXE") {
        Some(p) => { println!("cargo:rerun-if-changed={}", Path::new(&p).display()); std::fs::copy(&p, out.join("wii-party.exe")).expect("copy WP_HOST_EXE"); }
        None => {
            println!("cargo:warning=WP_HOST_EXE not set: this launcher cannot install the game (build crates/host first, see docs/BUILDING.md)");
            std::fs::write(out.join("wii-party.exe"), b"").unwrap();
        }
    }
    let rev = std::process::Command::new("git").args(["rev-parse", "--short=12", "HEAD"]).current_dir(&root).output().ok()
        .filter(|o| o.status.success()).map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_else(|| "dev".into());
    println!("cargo:rustc-env=WP_REVISION={rev}");
    println!("cargo:rerun-if-changed=../.git/HEAD");
}
