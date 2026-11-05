// port of engine/src/platform/settings.cpp

use crate::{keymap, padmap};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Mutex;

struct SettingOption {
    key: String,
    fallback: String,
}

const BASE_OPTIONS: [(&str, &str); 21] = [
    ("video.scale", "1"),
    ("video.fullscreen", "0"),
    ("video.copy_filter", "1"),
    ("video.dump_textures", "0"),
    ("video.custom_textures", "1"),
    ("input.gamepads", "1"),
    ("input.auto_grip", "1"),
    ("input.joycon_pairs", "1"),
    ("input.real_wiimotes", "1"),
    ("input.real_wiimote_mouse", "1"),
    ("input.wake_on_mouse", "1"),
    ("input.hide_cursor", "0"),
    ("input.controls_hint", "0"),
    ("system.interface_language", "en"),
    ("system.language", "auto"),
    ("system.pal60", "1"),
    ("system.skip_notices", "1"),
    ("system.options_menu", "1"),
    ("audio.mute", "0"),
    ("audio.wiimote_speaker", "1"),
    ("saves.backups", "5"),
];

// port of engine/src/platform/settings.cpp options
fn options() -> Vec<SettingOption> {
    let mut result: Vec<SettingOption> = BASE_OPTIONS
        .iter()
        .map(|(key, fallback)| SettingOption { key: key.to_string(), fallback: fallback.to_string() })
        .collect();
    for i in 0..keymap::ACTION_COUNT {
        result.push(SettingOption {
            key: keymap::setting_key(i),
            fallback: keymap::action_at(i).defaults.to_string(),
        });
    }
    for i in 0..padmap::ACTION_COUNT {
        result.push(SettingOption { key: padmap::setting_key(i), fallback: "auto".to_string() });
    }
    result
}

struct State {
    path: String,
    values: BTreeMap<String, String>,
}

static STATE: Mutex<State> = Mutex::new(State { path: String::new(), values: BTreeMap::new() });
static REVISION: AtomicU32 = AtomicU32::new(1);

// port of engine/src/platform/settings.cpp fallback_of
fn fallback_of(key: &str) -> String {
    for option in options() {
        if option.key == key {
            return option.fallback;
        }
    }
    String::new()
}

// port of engine/src/platform/settings.cpp trimmed
fn trimmed(text: &str) -> String {
    text.trim_matches(|ch| ch == ' ' || ch == '\t' || ch == '\r').to_string()
}

// port of engine/src/platform/settings.cpp write_file
fn write_file(state: &State) {
    if state.path.is_empty() {
        return;
    }
    let mut out = String::new();
    let mut section = String::new();
    for option in options() {
        let (group, name) = option.key.split_once('.').unwrap_or((&option.key, ""));
        if group != section {
            if !section.is_empty() {
                out.push('\n');
            }
            out.push('[');
            out.push_str(group);
            out.push_str("]\n");
            section = group.to_string();
        }
        out.push_str(name);
        out.push('=');
        out.push_str(state.values.get(&option.key).map(String::as_str).unwrap_or(""));
        out.push('\n');
    }
    let _ = std::fs::write(&state.path, out);
}

// port of engine/src/platform/settings.cpp value_of
fn value_of(key: &str, variable: Option<&str>) -> String {
    if let Some(variable) = variable {
        if let Ok(value) = std::env::var(variable) {
            return value;
        }
    }
    let state = STATE.lock().unwrap();
    state.values.get(key).cloned().unwrap_or_else(|| fallback_of(key))
}

// port of engine/src/platform/settings.cpp load
pub fn load(path: &str) {
    let mut state = STATE.lock().unwrap();
    state.path = path.to_string();
    for option in options() {
        state.values.insert(option.key.clone(), option.fallback.clone());
    }
    if let Ok(contents) = std::fs::read_to_string(path) {
        let mut section = String::new();
        for raw in contents.lines() {
            let line = trimmed(raw);
            if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                section = trimmed(&line[1..line.len() - 1]);
                continue;
            }
            let Some(equals) = line.find('=') else { continue };
            let key = format!("{}.{}", section, trimmed(&line[..equals]));
            if state.values.contains_key(&key) {
                state.values.insert(key, trimmed(&line[equals + 1..]));
            }
        }
    }
    write_file(&state);
    REVISION.fetch_add(1, Ordering::SeqCst);
}

// port of engine/src/platform/settings.cpp flag
pub fn flag(key: &str, variable: Option<&str>) -> bool {
    let value = value_of(key, variable);
    !(value == "0" || value == "false" || value == "off" || value == "no")
}

// port of engine/src/platform/settings.cpp atoi / number
fn atoi(text: &str) -> i32 {
    let bytes = text.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    let mut sign = 1i64;
    if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
        if bytes[i] == b'-' {
            sign = -1;
        }
        i += 1;
    }
    let mut value = 0i64;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        value = value * 10 + (bytes[i] - b'0') as i64;
        i += 1;
    }
    (sign * value) as i32
}

// port of engine/src/platform/settings.cpp number
pub fn number(key: &str, variable: Option<&str>) -> i32 {
    atoi(&value_of(key, variable))
}

// port of engine/src/platform/settings.cpp text
pub fn text(key: &str, variable: Option<&str>) -> String {
    value_of(key, variable)
}

// port of engine/src/platform/settings.cpp store
pub fn store(key: &str, value: &str) {
    let mut state = STATE.lock().unwrap();
    state.values.insert(key.to_string(), value.to_string());
    write_file(&state);
    REVISION.fetch_add(1, Ordering::SeqCst);
}

// port of engine/src/platform/settings.cpp keys
pub fn keys() -> Vec<String> {
    BASE_OPTIONS.iter().map(|(key, _)| key.to_string()).collect()
}

// port of engine/src/platform/settings.cpp revision
pub fn revision() -> u32 {
    REVISION.load(Ordering::SeqCst)
}

// port of engine/src/platform/settings.cpp LiveFlag::operator()
pub struct LiveFlag {
    key: String,
    variable: Option<String>,
    cache: AtomicU64,
}

impl LiveFlag {
    pub fn new(key: &str, variable: Option<&str>) -> Self {
        Self {
            key: key.to_string(),
            variable: variable.map(str::to_string),
            cache: AtomicU64::new(0),
        }
    }

    pub fn call(&self) -> bool {
        let current = revision() as u64;
        let mut cached = self.cache.load(Ordering::Acquire);
        if (cached >> 32) != current {
            cached = (current << 32) | if flag(&self.key, self.variable.as_deref()) { 1 } else { 0 };
            self.cache.store(cached, Ordering::Release);
        }
        (cached & 1) != 0
    }
}

// port of engine/src/platform/settings.cpp LiveNumber::operator()
pub struct LiveNumber {
    key: String,
    variable: Option<String>,
    cache: AtomicU64,
}

impl LiveNumber {
    pub fn new(key: &str, variable: Option<&str>) -> Self {
        Self {
            key: key.to_string(),
            variable: variable.map(str::to_string),
            cache: AtomicU64::new(0),
        }
    }

    pub fn call(&self) -> i32 {
        let current = revision() as u64;
        let mut cached = self.cache.load(Ordering::Acquire);
        if (cached >> 32) != current {
            cached = (current << 32) | number(&self.key, self.variable.as_deref()) as u32 as u64;
            self.cache.store(cached, Ordering::Release);
        }
        cached as u32 as i32
    }
}
