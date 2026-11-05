// Build script for the game DLL.
// - Embeds Dolphin's free replacement DSP ROMs (GPL) from the pinned upstream recompiler checkout.
// - Links SDL3 and the Windows/MinGW system libraries in the same order as the accepted build.
use std::path::PathBuf;

fn dir(var: &str, default: PathBuf) -> PathBuf {
    println!("cargo:rerun-if-env-changed={var}");
    std::env::var_os(var).map(PathBuf::from).unwrap_or(default)
}

fn main() {
    let root = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("../..");
    let generated = dir("WP_GENERATED_DIR", root.join("generated"));
    assert!(std::env::var_os("CARGO_FEATURE_RUST_GAME_DESCRIPTION").is_some(), "build with the features listed in tools/build/features.txt");
    if std::env::var_os("CARGO_FEATURE_RUST_DSP_ROM_REGISTRY").is_some() {
        let roms = generated.join("upstream/third_party/dolphin/Data/Sys/GC");
        for (key, file) in [("WP_PRIVATE_DSP_IROM", "dsp_rom.bin"), ("WP_PRIVATE_DSP_COEF", "dsp_coef.bin")] {
            let p = roms.join(file);
            assert!(p.exists(), "missing {} (run the recompiler pipeline first)", p.display());
            println!("cargo:rerun-if-changed={}", p.display());
            println!("cargo:rustc-env={}={}", key, p.to_string_lossy().replace('\\', "/"));
        }
    }
    let sdl = dir("WP_SDL3_DIR", root.join("build/deps/SDL3/x86_64-w64-mingw32"));
    println!("cargo:rustc-link-search=native={}", sdl.join("lib").display());
    println!("cargo:rustc-link-arg=-Wl,--start-group");
    for name in ["SDL3", "gdi32", "user32", "d3d11", "d3dcompiler", "dxgi", "ole32", "windowscodecs", "dwmapi", "kernel32", "winspool",
                 "shell32", "oleaut32", "uuid", "comdlg32", "advapi32", "stdc++"] {
        println!("cargo:rustc-link-arg=-l{name}");
    }
    // Support/import libraries rescanned inside the group so late CRT/pthread/compiler-support references resolve.
    for name in ["kernel32", "ntdll", "userenv", "ws2_32", "dbghelp", "gcc_eh", ":libpthread.a", "msvcrt", "mingwex", "mingw32", "gcc", "msvcrt",
                 "mingwex", "user32", "kernel32"] {
        println!("cargo:rustc-link-arg=-l{name}");
    }
    println!("cargo:rustc-link-arg=-Wl,--end-group");
}
