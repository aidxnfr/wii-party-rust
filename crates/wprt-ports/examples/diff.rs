// Differential Rust side for the O3 ports. The reference side is diff/ref.cpp.
use std::fs;
use std::io::Write;

use wprt_ports::format::{format_guest, Cpu, Memory};
use wprt_ports::{gx_lighting, gx_state, keymap, log, padmap, png, settings};

fn b(value: bool) -> u8 {
    value as u8
}

fn bits(value: f32) -> u32 {
    value.to_bits()
}

fn channel_hex(material: bool, enable: bool, ambient: bool, diffuse: u32, attenuation: u32, mask: u32) -> u32 {
    (if material { 1 } else { 0 })
        | (if enable { 2 } else { 0 })
        | ((mask & 15) << 2)
        | (if ambient { 0x40 } else { 0 })
        | ((diffuse & 3) << 7)
        | ((attenuation & 3) << 9)
        | (((mask >> 4) & 15) << 11)
}

fn set_color(xf: &mut [u32], index: u32, r: u8, g: u8, b: u8, a: u8) {
    xf[index as usize] = ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | a as u32;
}

fn print_keymap() {
    println!("## keymap");
    for i in 0..keymap::ACTION_COUNT {
        let action = keymap::action_at(i);
        println!(
            "a {} {} {} {} {}",
            i,
            action.name,
            action.buttons,
            action.defaults,
            keymap::setting_key(i)
        );
    }
    for name in ["A", "a", "F1", "F24", "f01", "MouseLeft", "WheelUp", "nope", "F10", "f10"] {
        println!("k {} {}", name, keymap::key_code(name));
    }
    for code in [0x41, 0x70, 0x87, 0x79, 0x100, 0x999, 0] {
        println!("n {} {} {}", code, keymap::key_name(code), b(keymap::reserved(code)));
    }
    for code in -2..=0x120 {
        let name = keymap::key_name(code);
        println!("x {} {} {}", code, name, b(keymap::reserved(code)));
        if !name.is_empty() {
            println!(
                "y {} {} {}",
                code,
                keymap::key_code(&name),
                keymap::key_code(&format!("  {}\t", name))
            );
        }
    }
    let parsed = keymap::parse("A,B,a,F10,NoSuch,");
    print!("p");
    for code in &parsed.codes {
        print!(" {}", code);
    }
    print!(" |");
    for name in &parsed.invalid {
        print!(" {}", name);
    }
    println!();
    println!("f {}", keymap::format(&[0x41, 0x70, 0x100, 0x999], "|"));
    let defaults = keymap::defaults();
    print!("d");
    for i in 0..keymap::ACTION_COUNT {
        print!(" :");
        for code in &defaults.codes[i] {
            print!("{},", code);
        }
    }
    println!(" {} {}", b(defaults.has(keymap::Action::A, 0x0D)), b(defaults.bound(0x100)));
    let value = |key: &str| -> String {
        for i in 0..keymap::ACTION_COUNT {
            if keymap::setting_key(i) == key {
                return keymap::action_at(i).defaults.to_string();
            }
        }
        String::new()
    };
    println!(
        "l {} {} {} {}",
        b(keymap::legacy_directions(&value)),
        b(keymap::direction(7)),
        b(keymap::direction(10)),
        b(keymap::direction(11))
    );
    let mut invalid = Vec::new();
    let loaded = keymap::load(&value, &mut invalid);
    print!("i {}", invalid.len());
    for i in 0..keymap::ACTION_COUNT {
        print!(" :");
        for code in &loaded.codes[i] {
            print!("{},", code);
        }
    }
    println!();
}

fn print_padmap() {
    println!("## padmap");
    for i in 0..padmap::ACTION_COUNT {
        println!(
            "a {} {} {} {}",
            i,
            padmap::action_name(i),
            padmap::action_button(i),
            padmap::setting_key(i)
        );
    }
    for i in -1..=17 {
        println!("p {} {}", i, padmap::physical_name(i));
    }
    println!(
        "q {} {} {}",
        padmap::physical_from_name("south"),
        padmap::physical_from_name("right_trigger"),
        padmap::physical_from_name("nope")
    );
    let mut pressed = padmap::Pressed::default();
    pressed.held[padmap::K_SOUTH] = true;
    pressed.held[padmap::K_RIGHT_SHOULDER] = true;
    pressed.held[padmap::K_START] = true;
    let mut automatic: padmap::Mapping = [padmap::AUTOMATIC; padmap::ACTION_COUNT];
    automatic.fill(padmap::AUTOMATIC);
    println!("b {} {}", padmap::buttons(&pressed, false, &automatic), padmap::buttons(&pressed, true, &automatic));
    let mut nintendo = pressed;
    nintendo.nintendo_layout = true;
    println!("n {}", padmap::buttons(&nintendo, false, &automatic));
    let mut custom = automatic;
    custom[0] = padmap::K_EAST as i32;
    custom[2] = padmap::K_WEST as i32;
    println!("c {}", padmap::buttons(&pressed, false, &custom));
    let current = padmap::current_mapping();
    print!("m");
    for value in current {
        print!(" {}", value);
    }
    println!();
}

fn print_png() {
    println!("## png");
    let pixels = [
        0x11223344, 0x55667788, 0x99aabbcc, 0xddeeff00, 0x01020304, 0x05060708, 0x090a0b0c, 0x0d0e0f10,
        0x11121314, 0x15161718, 0x191a1b1c, 0x1d1e1f20,
    ];
    let encoded = png::encode_png(&pixels, 4, 3);
    print!("e {} ", encoded.len());
    for byte in &encoded {
        print!("{:x} ", byte);
    }
    println!();
    png::save_png("wprt-ports-png.tmp", &pixels, 4, 3);
    let saved = fs::read("wprt-ports-png.tmp").unwrap_or_default();
    println!("s {} {}", saved.len(), b(saved == encoded));
    png::save_png_rgba("wprt-ports-rgba.tmp", &pixels, 4, 3);
    let rgba = fs::read("wprt-ports-rgba.tmp").unwrap_or_default();
    println!("r {} {:x}", rgba.len(), rgba.get(20).copied().unwrap_or(0));
}

fn print_gx_state() {
    println!("## gx_state");
    for mode in [0u32, 1, 2, 8, 16, 0x800, 0x1000, 0x2000, 0x1200, 0x7800, 0xFFFF] {
        let state = gx_state::blend_state(mode, true, true, true);
        println!(
            "b {} {} {} {} {} {} {} {} {} {} {} {} {}",
            mode,
            b(state.enable),
            state.operation as u8,
            state.alpha_operation as u8,
            state.source as u8,
            state.destination as u8,
            state.source_alpha as u8,
            state.destination_alpha as u8,
            b(state.color_update),
            b(state.alpha_update),
            b(state.logic_op),
            state.logic_mode,
            state.output as u8
        );
    }
    for mode in 0..16 {
        println!("l {} {} {}", mode, b(gx_state::logic_op_exact(mode)), gx_state::logic_op_name(mode));
    }
    let mut bp = vec![0u32; 0x50];
    bp[0x22] = (3u32 << 16) | (5u32 << 19);
    for i in 0..8u32 {
        bp[(0x30 + 2 * i) as usize] = (i << 18) | ((7 - i) << 19);
    }
    let offsets = gx_state::line_point_offsets(&bp);
    println!(
        "o {:x} {:x} {} {}",
        bits(offsets.line),
        bits(offsets.point),
        offsets.line_coordinates,
        offsets.point_coordinates
    );
    println!(
        "n {} {}",
        b(gx_state::line_offset_negative_side(1.0, -1.0, true)),
        b(gx_state::line_offset_negative_side(1.0, -1.0, false))
    );
    let mut vertex = gx_state::ScreenVertex::default();
    for i in 0..8 {
        vertex.uv[i][0] = i as f32;
        vertex.uv[i][1] = (i + 1) as f32;
        vertex.uv[i][2] = 0.5;
    }
    gx_state::offset_texture_coordinates(&mut vertex, 0xA5, 2.0, -3.0);
    print!("v");
    for i in 0..8 {
        print!(" {:x} {:x}", bits(vertex.uv[i][0]), bits(vertex.uv[i][1]));
    }
    println!();
}

fn print_gx_lighting() {
    println!("## gx_lighting");
    let mut xf = vec![0u32; 0x1200];
    for i in 0..9 {
        xf[0x400 + i] = if i == 0 || i == 4 || i == 8 { 1.0f32.to_bits() } else { 0.0f32.to_bits() };
    }
    set_color(&mut xf, 0x100C, 100, 150, 200, 255);
    set_color(&mut xf, 0x100C + 1, 10, 20, 30, 40);
    set_color(&mut xf, 0x100A, 10, 20, 30, 40);
    set_color(&mut xf, 0x100A + 1, 1, 2, 3, 4);
    xf[0x100E] = channel_hex(false, true, false, 2, 2, 0x3);
    xf[0x100E + 1] = 0;
    xf[0x1010] = 0;
    xf[0x1010 + 1] = 0;
    let set_light = |xf: &mut [u32], number: u32, r: u8, g: u8, b: u8, a: u8, px: f32, py: f32, pz: f32, dx: f32, dy: f32, dz: f32| {
        let base = 0x600 + number * 16;
        set_color(xf, base + 3, r, g, b, a);
        xf[(base + 4) as usize] = 1.0f32.to_bits();
        xf[(base + 5) as usize] = 0.5f32.to_bits();
        xf[(base + 6) as usize] = 0.25f32.to_bits();
        xf[(base + 7) as usize] = 1.0f32.to_bits();
        xf[(base + 8) as usize] = 0.0f32.to_bits();
        xf[(base + 9) as usize] = 0.0f32.to_bits();
        xf[(base + 10) as usize] = px.to_bits();
        xf[(base + 11) as usize] = py.to_bits();
        xf[(base + 12) as usize] = pz.to_bits();
        xf[(base + 13) as usize] = dx.to_bits();
        xf[(base + 14) as usize] = dy.to_bits();
        xf[(base + 15) as usize] = dz.to_bits();
    };
    set_light(&mut xf, 0, 255, 128, 64, 32, 0.0, 1.0, -2.0, 0.0, 0.0, 1.0);
    set_light(&mut xf, 1, 32, 64, 128, 255, 1.0, 0.0, -1.0, 1.0, 0.0, 0.0);
    for i in 2..8u32 {
        set_light(
            &mut xf,
            i,
            (20 + i * 20) as u8,
            (30 + i * 10) as u8,
            (40 + i * 5) as u8,
            (50 + i * 20) as u8,
            0.5 * i as f32,
            -0.25 * i as f32,
            -1.0 - i as f32,
            0.0,
            0.0,
            1.0,
        );
    }
    let position = [0.0f32, 0.0, 0.0];
    let normal = [0.0f32, 0.0, 1.0];
    let vertex_color = [[200u8, 100, 50, 255], [10, 20, 30, 40]];
    let mut out = [[0u8; 4]; 2];
    gx_lighting::begin(&xf);
    gx_lighting::light_channels(&position, &normal, &vertex_color, &mut out);
    println!(
        "c {} {} {} {} {} {} {} {}",
        out[0][0], out[0][1], out[0][2], out[0][3], out[1][0], out[1][1], out[1][2], out[1][3]
    );
    let mut transformed = [0.0f32; 3];
    gx_lighting::transform_normal(&xf, 2, &normal, &mut transformed);
    println!("t {:x} {:x} {:x}", bits(transformed[0]), bits(transformed[1]), bits(transformed[2]));
    for attenuation in 0..4u32 {
        for diffuse in 0..4u32 {
            for mask in [0u32, 1, 2, 3, 15, 16, 255] {
                xf[0x100E] = channel_hex(false, true, false, diffuse, attenuation, mask);
                xf[0x100E + 1] = 0;
                xf[0x1010] = 0;
                xf[0x1010 + 1] = 0;
                gx_lighting::begin(&xf);
                gx_lighting::light_channels(&position, &normal, &vertex_color, &mut out);
                print!("m {} {} {}", attenuation, diffuse, mask);
                for k in 0..2 {
                    for c in 0..4 {
                        print!(" {}", out[k][c]);
                    }
                }
                println!();
            }
        }
    }
    xf[0x100E] = 0;
    xf[0x100E + 1] = 0;
    xf[0x1010] = channel_hex(false, true, false, 2, 2, 3);
    xf[0x1010 + 1] = 0;
    gx_lighting::begin(&xf);
    gx_lighting::light_channels(&position, &normal, &vertex_color, &mut out);
    print!("a");
    for k in 0..2 {
        for c in 0..4 {
            print!(" {}", out[k][c]);
        }
    }
    println!();
    xf[0x100E] = channel_hex(true, true, true, 2, 2, 3);
    xf[0x100E + 1] = channel_hex(false, false, false, 0, 0, 0);
    xf[0x1010] = channel_hex(true, true, true, 2, 2, 3);
    xf[0x1010 + 1] = 0;
    gx_lighting::begin(&xf);
    gx_lighting::light_channels(&position, &normal, &vertex_color, &mut out);
    print!("w");
    for k in 0..2 {
        for c in 0..4 {
            print!(" {}", out[k][c]);
        }
    }
    println!();
}

struct Mem {
    data: Vec<u8>,
}

impl Mem {
    fn new() -> Self {
        Self { data: vec![0; 0x20000] }
    }

    fn put_string(&mut self, mut address: u32, text: &str) {
        for &byte in text.as_bytes() {
            self.data[address as usize] = byte;
            address += 1;
        }
        self.data[address as usize] = 0;
    }

    fn write32(&mut self, address: u32, value: u32) {
        self.data[address as usize..address as usize + 4].copy_from_slice(&value.to_be_bytes());
    }

    fn write64(&mut self, address: u32, value: u64) {
        self.data[address as usize..address as usize + 8].copy_from_slice(&value.to_be_bytes());
    }
}

impl Memory for Mem {
    fn rd8(&self, address: u32) -> u8 {
        self.data[(address & 0x1FFFFFFF) as usize]
    }

    fn rd32(&self, address: u32) -> u32 {
        let index = (address & 0x1FFFFFFF) as usize;
        u32::from_be_bytes(self.data[index..index + 4].try_into().unwrap())
    }

    fn rdf64(&self, address: u32) -> f64 {
        let index = (address & 0x1FFFFFFF) as usize;
        f64::from_bits(u64::from_be_bytes(self.data[index..index + 8].try_into().unwrap()))
    }
}

fn print_format() {
    println!("## format");
    let mut c = Cpu::default();
    c.r[1] = 0x3000;
    c.r[3] = 123;
    c.r[4] = 0xFFFFFFFF;
    c.r[5] = 0xAB;
    c.r[6] = 0x141;
    c.r[7] = 0x1100;
    c.r[8] = 0x1234;
    c.f[1] = 1.25;
    c.f[2] = 2.5;
    c.f[3] = 0.001;
    let mut mem = Mem::new();
    mem.put_string(0x1000, "hello %d %u %08x %c %s %p");
    mem.put_string(0x1100, "guest");
    println!("f1 [{}]", format_guest(&c, &mem, 0x1000, 3));
    mem.put_string(0x1200, "%.3f %g %e");
    println!("f2 [{}]", format_guest(&c, &mem, 0x1200, 3));
    mem.put_string(0x1300, "%lld %llx");
    c.r[3] = 0x11223344;
    c.r[4] = 0x55667788;
    c.r[5] = 0x99AABBCC;
    c.r[6] = 0xDDEEFF00;
    println!("f3 [{}]", format_guest(&c, &mem, 0x1300, 3));
    mem.put_string(0x1400, "%*d %s");
    c.r[3] = 5;
    c.r[4] = 42;
    c.r[5] = 0x1100;
    println!("f4 [{}]", format_guest(&c, &mem, 0x1400, 3));
    mem.put_string(0x1500, "%s");
    c.r[3] = 0;
    println!("f5 [{}]", format_guest(&c, &mem, 0x1500, 3));
    mem.put_string(0x1600, "%q");
    println!("f6 [{}]", format_guest(&c, &mem, 0x1600, 3));
    mem.put_string(0x1700, "%d %d %d %d %d %d %d %d %d %d");
    for i in 0..8 {
        c.r[3 + i] = i as u32 + 1;
    }
    mem.write32(0x3008, 9);
    mem.write32(0x300C, 10);
    println!("f7 [{}]", format_guest(&c, &mem, 0x1700, 3));
    mem.put_string(0x1800, "%i %+d % d %05d %#x %#o %X");
    c.r[3] = (-12i32) as u32;
    c.r[4] = 12;
    c.r[5] = 12;
    c.r[6] = 42;
    c.r[7] = 42;
    c.r[8] = 42;
    c.r[9] = 42;
    println!("f8 [{}]", format_guest(&c, &mem, 0x1800, 3));
    mem.put_string(0x1900, "%10.2f %-10.2f %+.2f");
    c.f[1] = 3.14159;
    c.f[2] = 3.14159;
    c.f[3] = 3.14159;
    println!("f9 [{}]", format_guest(&c, &mem, 0x1900, 3));
    mem.put_string(0x1A00, "%s %.3s %10s %-10s");
    c.r[3] = 0x1100;
    c.r[4] = 0x1100;
    c.r[5] = 0x1100;
    c.r[6] = 0x1100;
    println!("f10 [{}]", format_guest(&c, &mem, 0x1A00, 3));
    mem.put_string(0x1B00, "%f %f %f %f %f %f %f %f %f");
    for i in 1..=8 {
        c.f[i] = i as f64;
    }
    mem.write64(0x3008, 9.0f64.to_bits());
    println!("f11 [{}]", format_guest(&c, &mem, 0x1B00, 3));
}

fn print_settings() {
    println!("## settings");
    std::env::set_var("WPRT_SETTINGS_ENV", "1");
    fs::write(
        "wprt-ports-settings.tmp",
        "[system]\nlanguage=fr\npal60=0\n[keys]\na=Q\n[pad]\na=east\n",
    )
    .unwrap();
    settings::load("wprt-ports-settings.tmp");
    println!(
        "v {} {} {} {} {}",
        settings::text("system.language", None),
        b(settings::flag("system.pal60", None)),
        settings::number("saves.backups", None),
        settings::text("keys.a", None),
        settings::revision()
    );
    println!(
        "e {} {} {}",
        b(settings::flag("video.fullscreen", Some("WPRT_SETTINGS_ENV"))),
        settings::number("saves.backups", Some("WPRT_SETTINGS_ENV")),
        settings::text("system.language", Some("WPRT_SETTINGS_ENV"))
    );
    print!("k");
    for key in settings::keys() {
        print!(" {}", key);
    }
    println!();
    settings::store("system.language", "de");
    println!("r {}", settings::revision());
    let live_flag = settings::LiveFlag::new("video.pal60", None);
    let live_number = settings::LiveNumber::new("saves.backups", None);
    println!("l {} {}", b(live_flag.call()), live_number.call());
    settings::store("video.pal60", "1");
    settings::store("saves.backups", "9");
    println!("l {} {}", b(live_flag.call()), live_number.call());
    let mapping = padmap::current_mapping();
    print!("m");
    for value in mapping {
        print!(" {}", value);
    }
    println!();
    let contents = fs::read_to_string("wprt-ports-settings.tmp").unwrap_or_default();
    print!("file\n{}end\n", contents);
    let _ = std::io::stdout().flush();
}

fn print_log() {
    println!("## log");
    std::env::set_var("WP_LOG_FILE", "wprt-ports-log.tmp");
    println!("e {}", b(log::enabled()));
    log::write("cat", "hello 7");
    log::write("longcat", "no args");
    if let Ok(contents) = fs::read_to_string("wprt-ports-log.tmp") {
        for line in contents.lines() {
            if let Some(end) = line.find("] ") {
                println!("l {}", &line[end + 2..]);
            }
        }
    }
}

fn main() {
    print_keymap();
    print_padmap();
    print_png();
    print_gx_state();
    print_gx_lighting();
    print_format();
    print_settings();
    print_log();
}
