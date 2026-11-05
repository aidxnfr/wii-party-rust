// port of engine/src/input/keymap.cpp

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Action {
    A = 0,
    B,
    One,
    Two,
    Plus,
    Minus,
    Home,
    Up,
    Down,
    Left,
    Right,
    Shake,
    SwingUp,
    SwingDown,
    TiltLeft,
    TiltRight,
    TiltUp,
    TiltDown,
    Grip,
    Screenshot,
    Count,
}

pub const ACTION_COUNT: usize = Action::Count as usize;
pub const K_WHEEL_UP: i32 = 0x100;
pub const K_WHEEL_DOWN: i32 = 0x101;

#[derive(Clone, Copy)]
pub struct ActionInfo {
    pub name: &'static str,
    pub buttons: u32,
    pub defaults: &'static str,
}

// port of engine/src/input/keymap.cpp kActions / input::kButton*
pub const ACTIONS: [ActionInfo; ACTION_COUNT] = [
    ActionInfo { name: "a", buttons: 0x0800, defaults: "Enter,Space,MouseLeft" },
    ActionInfo { name: "b", buttons: 0x0400, defaults: "Backspace,MouseRight" },
    ActionInfo { name: "one", buttons: 0x0200, defaults: "1" },
    ActionInfo { name: "two", buttons: 0x0100, defaults: "2" },
    ActionInfo { name: "plus", buttons: 0x0010, defaults: "Plus,NumPlus" },
    ActionInfo { name: "minus", buttons: 0x1000, defaults: "Minus,NumMinus" },
    ActionInfo { name: "home", buttons: 0x8000, defaults: "H" },
    ActionInfo { name: "up", buttons: 0x0008, defaults: "Up,W" },
    ActionInfo { name: "down", buttons: 0x0004, defaults: "Down,S" },
    ActionInfo { name: "left", buttons: 0x0001, defaults: "Left,A" },
    ActionInfo { name: "right", buttons: 0x0002, defaults: "Right,D" },
    ActionInfo { name: "shake", buttons: 0x10000, defaults: "MouseMiddle,LeftShift" },
    ActionInfo { name: "swing_up", buttons: 0x20000, defaults: "WheelUp,T" },
    ActionInfo { name: "swing_down", buttons: 0x40000, defaults: "WheelDown,G" },
    ActionInfo { name: "tilt_left", buttons: 0x80000, defaults: "Q" },
    ActionInfo { name: "tilt_right", buttons: 0x100000, defaults: "E" },
    ActionInfo { name: "tilt_up", buttons: 0x200000, defaults: "R" },
    ActionInfo { name: "tilt_down", buttons: 0x400000, defaults: "F" },
    ActionInfo { name: "grip", buttons: 0, defaults: "Tab" },
    ActionInfo { name: "screenshot", buttons: 0, defaults: "F9" },
];

struct Legacy {
    action: Action,
    keys: &'static str,
}

const LEGACY_DIRECTIONS: [Legacy; 4] = [
    Legacy { action: Action::Up, keys: "Up,A" },
    Legacy { action: Action::Down, keys: "Down,D" },
    Legacy { action: Action::Left, keys: "Left,S" },
    Legacy { action: Action::Right, keys: "Right,W" },
];

const KEYS: [(&str, i32); 59] = [
    ("MouseLeft", 0x01),
    ("MouseRight", 0x02),
    ("MouseMiddle", 0x04),
    ("MouseX1", 0x05),
    ("MouseX2", 0x06),
    ("WheelUp", K_WHEEL_UP),
    ("WheelDown", K_WHEEL_DOWN),
    ("Backspace", 0x08),
    ("Tab", 0x09),
    ("Enter", 0x0D),
    ("Shift", 0x10),
    ("Ctrl", 0x11),
    ("Alt", 0x12),
    ("Pause", 0x13),
    ("CapsLock", 0x14),
    ("Escape", 0x1B),
    ("Space", 0x20),
    ("PageUp", 0x21),
    ("PageDown", 0x22),
    ("End", 0x23),
    ("Home", 0x24),
    ("Left", 0x25),
    ("Up", 0x26),
    ("Right", 0x27),
    ("Down", 0x28),
    ("Insert", 0x2D),
    ("Delete", 0x2E),
    ("Num0", 0x60),
    ("Num1", 0x61),
    ("Num2", 0x62),
    ("Num3", 0x63),
    ("Num4", 0x64),
    ("Num5", 0x65),
    ("Num6", 0x66),
    ("Num7", 0x67),
    ("Num8", 0x68),
    ("Num9", 0x69),
    ("NumMultiply", 0x6A),
    ("NumPlus", 0x6B),
    ("NumMinus", 0x6D),
    ("NumPeriod", 0x6E),
    ("NumDivide", 0x6F),
    ("LeftShift", 0xA0),
    ("RightShift", 0xA1),
    ("LeftCtrl", 0xA2),
    ("RightCtrl", 0xA3),
    ("LeftAlt", 0xA4),
    ("RightAlt", 0xA5),
    ("Semicolon", 0xBA),
    ("Plus", 0xBB),
    ("Comma", 0xBC),
    ("Minus", 0xBD),
    ("Period", 0xBE),
    ("Slash", 0xBF),
    ("Backquote", 0xC0),
    ("LeftBracket", 0xDB),
    ("Backslash", 0xDC),
    ("RightBracket", 0xDD),
    ("Quote", 0xDE),
];

const K_F1: i32 = 0x70;
const K_F24: i32 = 0x87;

// port of engine/src/input/keymap.cpp action(Action)
pub fn action(which: Action) -> &'static ActionInfo {
    &ACTIONS[which as usize]
}

// port of engine/src/input/keymap.cpp action(size_t)
pub fn action_at(index: usize) -> &'static ActionInfo {
    &ACTIONS[index]
}

// port of engine/src/input/keymap.cpp setting_key
pub fn setting_key(index: usize) -> String {
    format!("keys.{}", ACTIONS[index].name)
}

// port of engine/src/input/keymap.cpp lower
fn lower(text: &str) -> String {
    text.bytes()
        .map(|byte| if byte.is_ascii_uppercase() { (byte + 32) as char } else { byte as char })
        .collect()
}

// port of engine/src/input/keymap.cpp trimmed
fn trimmed(text: &str) -> String {
    text.trim_matches(|ch| ch == ' ' || ch == '\t' || ch == '\r' || ch == '\n').to_string()
}

// port of engine/src/input/keymap.cpp key_code
pub fn key_code(name: &str) -> i32 {
    let key = lower(&trimmed(name));
    let bytes = key.as_bytes();
    if bytes.len() == 1 && bytes[0].is_ascii_alphanumeric() {
        return bytes[0].to_ascii_uppercase() as i32;
    }
    if (2..=3).contains(&bytes.len())
        && bytes[0] == b'f'
        && bytes[1..].iter().all(|byte| byte.is_ascii_digit())
    {
        let number: i32 = key[1..].parse().unwrap_or(0);
        if (1..=K_F24 - K_F1 + 1).contains(&number) && bytes[1] != b'0' {
            return K_F1 + number - 1;
        }
        return -1;
    }
    for (key_name, code) in KEYS {
        if lower(key_name) == key {
            return code;
        }
    }
    -1
}

// port of engine/src/input/keymap.cpp key_name
pub fn key_name(code: i32) -> String {
    if (b'0' as i32..=b'9' as i32).contains(&code) || (b'A' as i32..=b'Z' as i32).contains(&code) {
        return (code as u8 as char).to_string();
    }
    if (K_F1..=K_F24).contains(&code) {
        return format!("F{}", code - K_F1 + 1);
    }
    for (name, key_code) in KEYS {
        if key_code == code {
            return name.to_string();
        }
    }
    String::new()
}

// port of engine/src/input/keymap.cpp reserved
pub fn reserved(code: i32) -> bool {
    code == 0x79 || code == 0x7A || code == 0x7B
}

#[derive(Default)]
pub struct Parsed {
    pub codes: Vec<i32>,
    pub invalid: Vec<String>,
}

// port of engine/src/input/keymap.cpp parse
pub fn parse(list: &str) -> Parsed {
    let mut result = Parsed::default();
    let mut start = 0usize;
    while start <= list.len() {
        let comma = list[start..].find(',').map(|offset| start + offset);
        let end = comma.unwrap_or(list.len());
        let token = trimmed(&list[start..end]);
        if !token.is_empty() {
            let code = key_code(&token);
            if code < 0 || reserved(code) {
                result.invalid.push(token);
            } else if !result.codes.contains(&code) {
                result.codes.push(code);
            }
        }
        if comma.is_none() {
            break;
        }
        start = end + 1;
    }
    result
}

// port of engine/src/input/keymap.cpp format
pub fn format(codes: &[i32], separator: &str) -> String {
    let mut result = String::new();
    for &code in codes {
        let name = key_name(code);
        if name.is_empty() {
            continue;
        }
        if !result.is_empty() {
            result.push_str(separator);
        }
        result.push_str(&name);
    }
    result
}

#[derive(Default)]
pub struct Bindings {
    pub codes: [Vec<i32>; ACTION_COUNT],
}

impl Bindings {
    // port of engine/src/input/keymap.cpp Bindings::of
    pub fn of(&self, which: Action) -> &Vec<i32> {
        &self.codes[which as usize]
    }

    // port of engine/src/input/keymap.cpp Bindings::has
    pub fn has(&self, which: Action, code: i32) -> bool {
        self.of(which).contains(&code)
    }

    // port of engine/src/input/keymap.cpp Bindings::bound
    pub fn bound(&self, code: i32) -> bool {
        self.codes.iter().any(|list| list.contains(&code))
    }
}

// port of engine/src/input/keymap.cpp defaults
pub fn defaults() -> Bindings {
    let mut result = Bindings::default();
    for i in 0..ACTION_COUNT {
        result.codes[i] = parse(ACTIONS[i].defaults).codes;
    }
    result
}

// port of engine/src/input/keymap.cpp legacy_directions
pub fn legacy_directions(value: &dyn Fn(&str) -> String) -> bool {
    for entry in &LEGACY_DIRECTIONS {
        if parse(&value(&setting_key(entry.action as usize))).codes != parse(entry.keys).codes {
            return false;
        }
    }
    true
}

// port of engine/src/input/keymap.cpp direction
pub fn direction(index: usize) -> bool {
    index >= Action::Up as usize && index <= Action::Right as usize
}

// port of engine/src/input/keymap.cpp load
pub fn load(value: &dyn Fn(&str) -> String, invalid: &mut Vec<String>) -> Bindings {
    let mut result = Bindings::default();
    let legacy = legacy_directions(value);
    for i in 0..ACTION_COUNT {
        let key = setting_key(i);
        let source = if legacy && direction(i) { ACTIONS[i].defaults.to_string() } else { value(&key) };
        let parsed = parse(&source);
        for name in &parsed.invalid {
            invalid.push(format!("{}: {}", key, name));
        }
        result.codes[i] = parsed.codes;
    }
    result
}
