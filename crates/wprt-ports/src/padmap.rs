// port of engine/src/input/padmap.cpp

use crate::settings;

pub const K_SOUTH: usize = 0;
pub const K_EAST: usize = 1;
pub const K_WEST: usize = 2;
pub const K_NORTH: usize = 3;
pub const K_BACK: usize = 4;
pub const K_GUIDE: usize = 5;
pub const K_START: usize = 6;
pub const K_LEFT_STICK: usize = 7;
pub const K_RIGHT_STICK: usize = 8;
pub const K_LEFT_SHOULDER: usize = 9;
pub const K_RIGHT_SHOULDER: usize = 10;
pub const K_DPAD_UP: usize = 11;
pub const K_DPAD_DOWN: usize = 12;
pub const K_DPAD_LEFT: usize = 13;
pub const K_DPAD_RIGHT: usize = 14;
pub const K_LEFT_TRIGGER: usize = 15;
pub const K_RIGHT_TRIGGER: usize = 16;
pub const PHYSICAL_COUNT: usize = 17;
pub const AUTOMATIC: i32 = -1;
pub const ACTION_COUNT: usize = 7;

pub const BUTTON_LEFT: u32 = 0x0001;
pub const BUTTON_RIGHT: u32 = 0x0002;
pub const BUTTON_DOWN: u32 = 0x0004;
pub const BUTTON_UP: u32 = 0x0008;
pub const BUTTON_PLUS: u32 = 0x0010;
pub const BUTTON_TWO: u32 = 0x0100;
pub const BUTTON_ONE: u32 = 0x0200;
pub const BUTTON_B: u32 = 0x0400;
pub const BUTTON_A: u32 = 0x0800;
pub const BUTTON_MINUS: u32 = 0x1000;
pub const BUTTON_HOME: u32 = 0x8000;

#[derive(Clone, Copy)]
pub struct Action {
    pub name: &'static str,
    pub button: u32,
}

const ACTIONS: [Action; ACTION_COUNT] = [
    Action { name: "a", button: BUTTON_A },
    Action { name: "b", button: BUTTON_B },
    Action { name: "one", button: BUTTON_ONE },
    Action { name: "two", button: BUTTON_TWO },
    Action { name: "plus", button: BUTTON_PLUS },
    Action { name: "minus", button: BUTTON_MINUS },
    Action { name: "home", button: BUTTON_HOME },
];

const PHYSICAL_NAMES: [&str; PHYSICAL_COUNT] = [
    "south",
    "east",
    "west",
    "north",
    "back",
    "guide",
    "start",
    "left_stick",
    "right_stick",
    "left_shoulder",
    "right_shoulder",
    "dpad_up",
    "dpad_down",
    "dpad_left",
    "dpad_right",
    "left_trigger",
    "right_trigger",
];

#[derive(Clone, Copy, Default)]
pub struct Pressed {
    pub held: [bool; PHYSICAL_COUNT],
    pub nintendo_layout: bool,
}

pub type Mapping = [i32; ACTION_COUNT];

// port of engine/src/input/padmap.cpp automatic
fn automatic(pressed: &Pressed, sideways: bool) -> u32 {
    let held = |physical: usize| pressed.held[physical];
    let mut b = 0u32;
    if held(K_RIGHT_SHOULDER) {
        b |= BUTTON_A;
    }
    if held(K_RIGHT_TRIGGER) {
        b |= BUTTON_B;
    }
    if sideways {
        if held(K_SOUTH) {
            b |= BUTTON_TWO;
        }
        if held(K_WEST) || held(K_EAST) {
            b |= BUTTON_ONE;
        }
        if held(K_NORTH) {
            b |= BUTTON_A;
        }
    } else {
        if held(if pressed.nintendo_layout { K_EAST } else { K_SOUTH }) {
            b |= BUTTON_A;
        }
        if held(if pressed.nintendo_layout { K_SOUTH } else { K_EAST }) {
            b |= BUTTON_B;
        }
        if held(K_WEST) {
            b |= BUTTON_ONE;
        }
        if held(K_NORTH) {
            b |= BUTTON_TWO;
        }
    }
    if held(K_BACK) {
        b |= BUTTON_MINUS;
    }
    if held(K_START) {
        b |= BUTTON_PLUS;
    }
    if held(K_GUIDE) {
        b |= BUTTON_HOME;
    }
    b
}

// port of engine/src/input/padmap.cpp action_name
pub fn action_name(action: usize) -> &'static str {
    if action < ACTION_COUNT {
        ACTIONS[action].name
    } else {
        ""
    }
}

// port of engine/src/input/padmap.cpp action_button
pub fn action_button(action: usize) -> u32 {
    if action < ACTION_COUNT {
        ACTIONS[action].button
    } else {
        0
    }
}

// port of engine/src/input/padmap.cpp setting_key
pub fn setting_key(action: usize) -> String {
    format!("pad.{}", action_name(action))
}

// port of engine/src/input/padmap.cpp physical_name
pub fn physical_name(physical: i32) -> &'static str {
    if physical >= 0 && (physical as usize) < PHYSICAL_COUNT {
        PHYSICAL_NAMES[physical as usize]
    } else {
        "auto"
    }
}

// port of engine/src/input/padmap.cpp physical_from_name
pub fn physical_from_name(name: &str) -> i32 {
    for (index, physical) in PHYSICAL_NAMES.iter().enumerate() {
        if name == *physical {
            return index as i32;
        }
    }
    AUTOMATIC
}

// port of engine/src/input/padmap.cpp current_mapping
pub fn current_mapping() -> Mapping {
    let mut mapping = [AUTOMATIC; ACTION_COUNT];
    for i in 0..ACTION_COUNT {
        mapping[i] = physical_from_name(&settings::text(&setting_key(i), None));
    }
    mapping
}

// port of engine/src/input/padmap.cpp buttons
pub fn buttons(pressed: &Pressed, sideways: bool, mapping: &Mapping) -> u32 {
    let mut custom = false;
    for &physical in mapping {
        custom = custom || physical != AUTOMATIC;
    }
    if !custom || sideways {
        return automatic(pressed, sideways);
    }
    let mut unmapped = *pressed;
    for &physical in mapping {
        if physical != AUTOMATIC {
            unmapped.held[physical as usize] = false;
        }
    }
    let defaults = automatic(&unmapped, false);
    let mut b = 0u32;
    for i in 0..ACTION_COUNT {
        if mapping[i] == AUTOMATIC {
            b |= defaults & ACTIONS[i].button;
        } else if pressed.held[mapping[i] as usize] {
            b |= ACTIONS[i].button;
        }
    }
    b
}
