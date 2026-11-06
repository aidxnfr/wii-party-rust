//! Launcher preferences, stored in %APPDATA%\WiiPartyRust\launcher.json.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Prefs {
    pub install_dir: PathBuf,
    pub disc: Option<PathBuf>,
    /// parallel compile jobs for the game build (the generated code needs ~3 GB of RAM per job)
    pub jobs: usize,
    pub offline: bool,
    /// extra in-between frames for smooth motion (0 = off; adds latency, see docs/SMOOTH_MOTION.md)
    pub smooth_motion: u32,
    pub show_stats: bool,
    /// launcher look: "system", "light" or "dark"
    pub theme: String,
}

impl Default for Prefs {
    fn default() -> Self {
        let cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
        Self { install_dir: crate::paths::default_root(), disc: None, jobs: cores.clamp(1, 8), offline: false, smooth_motion: 0, show_stats: false, theme: "system".into() }
    }
}

fn file() -> PathBuf {
    std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".")).join("WiiPartyRust").join("launcher.json")
}

impl Prefs {
    pub fn load() -> Self { std::fs::read_to_string(file()).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default() }
    pub fn save(&self) {
        let f = file(); let _ = std::fs::create_dir_all(f.parent().unwrap());
        let _ = std::fs::write(f, serde_json::to_string_pretty(self).unwrap());
    }
}
