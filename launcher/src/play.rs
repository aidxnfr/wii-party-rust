//! Start the installed game with the same settings as the accepted build.
use crate::paths::Layout;
use crate::prefs::Prefs;
use std::process::Command;

pub fn installed(l: &Layout) -> bool { l.host_exe().exists() && l.game_dll().exists() && l.extracted().join("sys/main.dol").exists() }

pub fn play(l: &Layout, p: &Prefs) -> Result<(), String> {
    if !installed(l) { return Err("Wii Party is not installed yet. Press Install first.".into()); }
    let data = l.data(); let nand = data.join("games").join("wiiparty").join("nand");
    std::fs::create_dir_all(&nand).map_err(|e| format!("save folder: {e}"))?;
    std::fs::write(data.join("interp.txt"), format!("{}\n", p.smooth_motion)).map_err(|e| format!("settings: {e}"))?;
    let path = std::env::join_paths(std::iter::once(l.game()).chain(std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())))
        .map_err(|e| e.to_string())?;
    let log = data.join("runtime.log");
    let mut cmd = Command::new(l.host_exe());
    cmd.arg(l.game_dll()).arg(l.extracted()).arg(&nand).current_dir(&data).env("PATH", path)
        // GPU publication path of the accepted build (DX12, shared device, source-exact presenter)
        .env("WPGX_HOST_GPU_PUBLICATION", "1").env("WPGX_HOST_DX12_FORCE_EARLY", "1").env("WPGX_HOST_SOURCE_PRESENT", "1")
        .env("WGPU_BACKEND", "dx12").env("WPGX_BACKEND", "dx12")
        .env("WP_OFFSCREEN", "1").env("WP_HOST_INPUT", "1").env("WP_REAL_WIIMOTES", "0")
        .env("WP_LOG_FILE", &log).env("WP_INTERP_FILE", data.join("interp.txt"))
        .env("WII_PARTY_SDL3", l.game().join("SDL3.dll")).env("WII_PARTY_PRIVATE_ROOT", data.join("private"));
    for k in ["WPGX_HOST_SHARED_MINIMAL", "WPGX_HOST_GPU_CPU_VIEW", "WP_GAMEPAD", "WP_INPUT_SCRIPT", "WP_DETERMINISTIC", "WP_COMPARE_LOG"] { cmd.env_remove(k); }
    let game = cmd.spawn().map_err(|e| format!("could not start the game: {e}"))?;
    let stats = l.game().join("stats.exe");
    if p.show_stats && stats.exists() { let _ = Command::new(stats).arg(&log).arg(game.id().to_string()).spawn(); }
    Ok(())
}
