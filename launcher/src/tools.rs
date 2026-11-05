//! Pinned build tools. Every download is checked against its published SHA-256 before use.
//! To update a tool: change its version/url/sha256 here (and in docs/BUILDING.md), then rebuild the launcher.

#[derive(Clone, Copy, PartialEq)]
pub enum Pack {
    /// .zip extracted into the tool folder
    Zip,
    /// .tar.gz extracted into the tool folder
    TarGz,
    /// 7-Zip self-extracting .exe (`-y -o<folder>`)
    SelfExtracting,
    /// single executable copied into the tool folder
    Single,
    /// Python wheel (.whl = zip) extracted into the tool folder
    Wheel,
}

pub struct Tool {
    pub key: &'static str,
    pub name: &'static str,
    pub url: &'static str,
    pub sha256: &'static str,
    pub file: &'static str,
    pub pack: Pack,
}

/// Rust toolchain installed by rustup-init (rustup verifies its own components).
pub const RUST_TOOLCHAIN: &str = "1.99.0-x86_64-pc-windows-gnu";

pub const TOOLS: &[Tool] = &[
    Tool { key: "nodtool", name: "nodtool 1.4.4", url: "https://github.com/encounter/nod/releases/download/v1.4.4/nodtool-windows-x86_64.exe",
           sha256: "fb3203a68a59fa19ba9de0aa2f8c339396b6e4c5f9274d0858d302074cdedf8b", file: "nodtool.exe", pack: Pack::Single },
    Tool { key: "python", name: "Python 3.12.10", url: "https://www.python.org/ftp/python/3.12.10/python-3.12.10-embed-amd64.zip",
           sha256: "4acbed6dd1c744b0376e3b1cf57ce906f9dc9e95e68824584c8099a63025a3c3", file: "python-3.12.10-embed-amd64.zip", pack: Pack::Zip },
    Tool { key: "pycparser", name: "pycparser 2.23", url: "https://files.pythonhosted.org/packages/a0/e3/59cd50310fc9b59512193629e1984c1f95e5c8ae6e5d8c69532ccc65a7fe/pycparser-2.23-py3-none-any.whl",
           sha256: "e5c6e8d3fbad53479cab09ac03729e0a9faf2bee3db8208a550daf5af81a5934", file: "pycparser-2.23-py3-none-any.whl", pack: Pack::Wheel },
    Tool { key: "mingw", name: "MinGW-w64 GCC 15.2 (nuwen.net 20.0)", url: "https://nuwen.net/files/mingw/mingw-20.0.exe",
           sha256: "bce8bdaa095848561488d34fd80371a1f36eb0a590202eb534f32e026e5cd8ef", file: "mingw-20.0.exe", pack: Pack::SelfExtracting },
    Tool { key: "sdl3", name: "SDL 3.4.16 (MinGW)", url: "https://github.com/libsdl-org/SDL/releases/download/release-3.4.16/SDL3-devel-3.4.16-mingw.tar.gz",
           sha256: "c7ef65bd72eabac6e5b535411dbd8d5824d0aab24fd62ff8812666b336f18a9c", file: "SDL3-devel-3.4.16-mingw.tar.gz", pack: Pack::TarGz },
    Tool { key: "rustup", name: "rustup 1.29.1", url: "https://static.rust-lang.org/rustup/archive/1.29.1/x86_64-pc-windows-msvc/rustup-init.exe",
           sha256: "6f4bef66261261fcb43131be8720bab817d403a09edec7455c371974b90bdb7e", file: "rustup-init.exe", pack: Pack::Single },
];
