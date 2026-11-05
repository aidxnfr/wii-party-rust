//! Files embedded into the launcher at build time (see build.rs).
/// The project source (crates/, recompiler/, third_party/, tools/ ...) as a zip.
pub static SOURCE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/source.zip"));
/// The prebuilt Bevy host (crates/host, our own code). Empty if the launcher was built without WP_HOST_EXE.
pub static HOST_EXE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/wii-party.exe"));
/// Source revision (git commit) the launcher was built from.
pub const REVISION: &str = env!("WP_REVISION");
