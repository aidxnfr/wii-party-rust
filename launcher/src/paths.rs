//! Folder layout of an installation (all under one folder the user picks).
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct Layout { pub root: PathBuf }

impl Layout {
    pub fn new(root: impl Into<PathBuf>) -> Self { Self { root: root.into() } }
    /// Unpacked project source (replaced on every install/update).
    pub fn source(&self) -> PathBuf { self.root.join("source") }
    /// Generated game code from the user's disc (inside source/ because the game crate refers to ../../generated;
    /// kept between updates and regenerated only when the recompiler or translator changes).
    pub fn generated(&self) -> PathBuf { self.source().join("generated") }
    pub fn tools(&self) -> PathBuf { self.root.join("tools") }
    pub fn tool(&self, key: &str) -> PathBuf { self.tools().join(key) }
    pub fn downloads(&self) -> PathBuf { self.root.join("downloads") }
    pub fn target(&self) -> PathBuf { self.root.join("build").join("target") }
    /// The playable game: host exe, game DLL, SDL3.dll, stats window.
    pub fn game(&self) -> PathBuf { self.root.join("game") }
    /// Saves, settings and logs. Never touched by installs or updates.
    pub fn data(&self) -> PathBuf { self.root.join("data") }
    pub fn mingw_bin(&self) -> PathBuf { self.tool("mingw").join("MinGW").join("bin") }
    pub fn sdl3(&self) -> PathBuf { self.tool("sdl3").join("SDL3-3.4.16").join("x86_64-w64-mingw32") }
    pub fn python(&self) -> PathBuf { self.tool("python").join("python.exe") }
    pub fn rustup_home(&self) -> PathBuf { self.tool("rust").join("rustup") }
    pub fn cargo_home(&self) -> PathBuf { self.tool("rust").join("cargo") }
    pub fn extracted(&self) -> PathBuf { self.generated().join("upstream").join("games").join("wiiparty").join("extracted") }
    pub fn host_exe(&self) -> PathBuf { self.game().join("wii-party.exe") }
    pub fn game_dll(&self) -> PathBuf { self.game().join("wiiparty_game.dll") }
    pub fn stamp(&self) -> PathBuf { self.root.join("installed.json") }
}

pub fn default_root() -> PathBuf {
    std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".")).join("WiiPartyRust")
}

pub fn free_gigabytes(path: &Path) -> Option<f64> {
    let mut p = path.to_path_buf();
    while !p.exists() { if !p.pop() { return None; } }
    fs_free(&p)
}

#[cfg(windows)]
fn fs_free(p: &Path) -> Option<f64> {
    use std::os::windows::ffi::OsStrExt;
    let wide: Vec<u16> = p.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    let mut free: u64 = 0;
    extern "system" { fn GetDiskFreeSpaceExW(dir: *const u16, avail: *mut u64, total: *mut u64, free: *mut u64) -> i32; }
    let ok = unsafe { GetDiskFreeSpaceExW(wide.as_ptr(), &mut free, std::ptr::null_mut(), std::ptr::null_mut()) };
    (ok != 0).then(|| free as f64 / (1u64 << 30) as f64)
}
#[cfg(not(windows))]
fn fs_free(_p: &Path) -> Option<f64> { None }
