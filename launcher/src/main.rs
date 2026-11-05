//! Wii Party (Rust) launcher: builds the game on your PC from your own disc, then plays it.
//! Modelled on the arelkair/wiiparty-recomp launcher (Install / Play / Settings / Controls).
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
mod install;
mod paths;
mod payload;
mod play;
mod prefs;
mod tools;

use eframe::egui;
use install::{Event, Options};
use paths::Layout;
use prefs::Prefs;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;

const GITHUB: &str = "https://github.com/aidxnfr/wii-party-rust";
const UPSTREAM: &str = "https://github.com/arelkair/wiiparty-recomp";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !args.is_empty() { std::process::exit(cli(&args)); }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([820.0, 600.0]).with_min_inner_size([640.0, 480.0]).with_title("Wii Party (Rust)"),
        ..Default::default()
    };
    let _ = eframe::run_native("Wii Party (Rust)", options, Box::new(|_cc| Ok(Box::new(App::new()))));
}

/// Command line: `install --disc <image> [--dir <folder>] [--jobs N] [--offline]` or `play [--dir <folder>]`.
fn cli(args: &[String]) -> i32 {
    let mut p = Prefs::load();
    let mut i = 1;
    while i < args.len() {
        let v = args.get(i + 1).cloned().unwrap_or_default();
        match args[i].as_str() {
            "--disc" => { p.disc = Some(PathBuf::from(v)); i += 1; }
            "--dir" => { p.install_dir = PathBuf::from(v); i += 1; }
            "--jobs" => { p.jobs = v.parse().unwrap_or(p.jobs); i += 1; }
            "--offline" => p.offline = true,
            other => { eprintln!("unknown option {other}"); return 2; }
        }
        i += 1;
    }
    match args[0].as_str() {
        "install" => {
            let (tx, rx) = std::sync::mpsc::channel();
            let o = Options { root: p.install_dir.clone(), disc: p.disc.clone(), jobs: p.jobs, offline: p.offline };
            std::thread::spawn(move || install::run(&o, tx));
            for ev in rx {
                match ev {
                    Event::Step(n, t, s) => println!("== [{n}/{t}] {s}"),
                    Event::Line(l) => println!("{l}"),
                    Event::Done(Ok(())) => { p.save(); return 0; }
                    Event::Done(Err(e)) => { eprintln!("error: {e}"); return 1; }
                }
            }
            1
        }
        "play" => match play::play(&Layout::new(&p.install_dir), &p) { Ok(()) => 0, Err(e) => { eprintln!("{e}"); 1 } },
        _ => { eprintln!("usage: wii-party-launcher [install --disc <image> [--dir <folder>] [--jobs N] [--offline] | play [--dir <folder>]]"); 2 }
    }
}

#[derive(PartialEq, Clone, Copy)]
enum Page { Home, Settings, Controls, About }

struct App {
    prefs: Prefs,
    page: Page,
    rx: Option<Receiver<Event>>,
    step: Option<(usize, usize, String)>,
    log: Vec<String>,
    message: Option<(bool, String)>,
}

impl App {
    fn new() -> Self { Self { prefs: Prefs::load(), page: Page::Home, rx: None, step: None, log: Vec::new(), message: None } }
    fn busy(&self) -> bool { self.rx.is_some() }
    fn start_install(&mut self) {
        self.prefs.save();
        let (tx, rx) = std::sync::mpsc::channel();
        let o = Options { root: self.prefs.install_dir.clone(), disc: self.prefs.disc.clone(), jobs: self.prefs.jobs, offline: self.prefs.offline };
        std::thread::spawn(move || install::run(&o, tx));
        self.rx = Some(rx); self.log.clear(); self.message = None; self.step = None;
    }
    fn poll(&mut self) {
        let Some(rx) = &self.rx else { return };
        let mut done = None;
        while let Ok(ev) = rx.try_recv() {
            match ev {
                Event::Step(n, t, s) => { self.log.push(format!("== {s}")); self.step = Some((n, t, s)); }
                Event::Line(l) => self.log.push(l),
                Event::Done(r) => done = Some(r),
            }
        }
        if self.log.len() > 2000 { self.log.drain(..self.log.len() - 2000); }
        if let Some(r) = done {
            self.rx = None;
            self.message = Some(match r { Ok(()) => (true, "Installed. Press Play!".into()), Err(e) => (false, e) });
        }
    }

    fn home(&mut self, ui: &mut egui::Ui) {
        let layout = Layout::new(&self.prefs.install_dir);
        let installed = play::installed(&layout);
        egui::Grid::new("paths").num_columns(3).spacing([8.0, 8.0]).show(ui, |ui| {
            ui.label("Install folder");
            ui.label(self.prefs.install_dir.display().to_string());
            if ui.add_enabled(!self.busy(), egui::Button::new("Change…")).clicked() {
                if let Some(d) = rfd::FileDialog::new().pick_folder() { self.prefs.install_dir = d.join("WiiPartyRust"); self.prefs.save(); }
            }
            ui.end_row();
            ui.label("Your disc image");
            ui.label(self.prefs.disc.as_ref().map(|d| d.display().to_string()).unwrap_or_else(|| "not chosen".into()));
            if ui.add_enabled(!self.busy(), egui::Button::new("Choose…")).clicked() {
                if let Some(f) = rfd::FileDialog::new().add_filter("Wii disc image", &["iso", "wbfs", "rvz", "wia", "ciso", "gcm"]).pick_file() {
                    self.prefs.disc = Some(f); self.prefs.save();
                }
            }
            ui.end_row();
        });
        if let Some(gb) = paths::free_gigabytes(&self.prefs.install_dir) {
            let need = if installed { 2.0 } else { 25.0 };
            ui.label(egui::RichText::new(format!("{gb:.0} GB free, about {need:.0} GB needed")).color(if gb < need { egui::Color32::LIGHT_RED } else { ui.visuals().weak_text_color() }));
        }
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            let can_install = !self.busy() && (self.prefs.disc.is_some() || layout.extracted().join("sys/main.dol").exists());
            let label = if installed { "Update / Rebuild" } else { "Install" };
            if ui.add_enabled(can_install, egui::Button::new(egui::RichText::new(label).size(18.0)).min_size([140.0, 36.0].into())).clicked() { self.start_install(); }
            if ui.add_enabled(installed && !self.busy(), egui::Button::new(egui::RichText::new("▶ Play").size(18.0)).min_size([140.0, 36.0].into())).clicked() {
                if let Err(e) = play::play(&layout, &self.prefs) { self.message = Some((false, e)); }
            }
            if ui.add_enabled(layout.data().exists(), egui::Button::new("Open saves folder")).clicked() { let _ = open::that(layout.data()); }
        });
        ui.label(egui::RichText::new("Installing builds the game on this PC from your disc. Nothing from the game is downloaded or uploaded. \
            The first build takes a while (about 30–90 minutes, depending on your CPU). Updates keep your saves and settings.").weak());
        if let Some((n, t, s)) = &self.step {
            ui.add_space(6.0);
            let frac = if self.busy() { (*n as f32 - 0.5) / *t as f32 } else { 1.0 };
            ui.add(egui::ProgressBar::new(frac).text(format!("step {n} of {t}: {s}")));
        }
        if let Some((ok, m)) = &self.message {
            ui.colored_label(if *ok { egui::Color32::LIGHT_GREEN } else { egui::Color32::LIGHT_RED }, m);
        }
        ui.add_space(6.0);
        egui::ScrollArea::vertical().stick_to_bottom(true).auto_shrink([false, false]).show(ui, |ui| {
            for l in &self.log { ui.monospace(l); }
        });
    }

    fn settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("Build");
        ui.add(egui::Slider::new(&mut self.prefs.jobs, 1..=16).text("parallel compile jobs (each needs ~3 GB RAM)"));
        ui.checkbox(&mut self.prefs.offline, "Offline mode (use tools already in the downloads folder; no network)");
        ui.add_space(10.0);
        ui.heading("Game");
        ui.label("Smooth motion: draws extra in-between frames for high refresh rate screens. The game itself stays at 60. \
                  Adds about 10 ms of delay and can show small glitches, so it is off by default.");
        ui.horizontal(|ui| {
            for (v, name) in [(0u32, "Off (60)"), (1, "2x (120 Hz)"), (2, "3x (180 Hz)"), (3, "4x (240 Hz)")] {
                ui.radio_value(&mut self.prefs.smooth_motion, v, name);
            }
        });
        ui.checkbox(&mut self.prefs.show_stats, "Open the performance stats window with the game");
        ui.add_space(10.0);
        if ui.button("Save").clicked() { self.prefs.save(); self.message = Some((true, "Settings saved.".into())); self.page = Page::Home; }
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Keyboard and mouse (Wii Remote 1)");
        egui::Grid::new("keys").striped(true).show(ui, |ui| {
            for (k, v) in [("Mouse", "point"), ("Enter / Space", "A"), ("Backspace", "B"), ("Enter + Backspace (title)", "A + B"),
                           ("WASD / arrows", "D-pad"), ("H", "HOME"), ("Left Shift / middle mouse", "shake"), ("T / G or wheel", "swing"),
                           ("Q / E / R / F", "tilt"), ("Tab", "grip toggle"), ("F10", "options menu"), ("F11", "full screen")] {
                ui.label(k); ui.label(v); ui.end_row();
            }
        });
        ui.label("Game controllers act as Wii Remotes 1–4. Key bindings follow the in-game options (F10).");
    }

    fn about(&mut self, ui: &mut egui::Ui) {
        ui.heading("Wii Party (Rust)");
        ui.label(format!("Version {} ({})", env!("CARGO_PKG_VERSION"), payload::REVISION));
        ui.add_space(6.0);
        ui.label("A Rust + Bevy port of Wii Party for PC.");
        ui.horizontal_wrapped(|ui| {
            ui.label("Built on");
            ui.hyperlink_to("wiiparty-recomp by arelkair", UPSTREAM);
            ui.label("— the static recompiler, runtime design and research this project would not exist without.");
        });
        ui.hyperlink_to("Project page", GITHUB);
        ui.add_space(6.0);
        ui.label("Also uses: Dolphin (DSP and GX references), nod/nodtool, SDL, Bevy, wgpu, egui. See THIRD_PARTY_NOTICES.md.");
        ui.label("Not affiliated with or endorsed by Nintendo. You need your own legally obtained copy of the game.");
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll();
        if self.busy() { ctx.request_repaint_after(std::time::Duration::from_millis(100)); }
        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                for (p, name) in [(Page::Home, "Home"), (Page::Settings, "Settings"), (Page::Controls, "Controls"), (Page::About, "About")] {
                    ui.selectable_value(&mut self.page, p, name);
                }
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| match self.page {
            Page::Home => self.home(ui),
            Page::Settings => self.settings(ui),
            Page::Controls => self.controls(ui),
            Page::About => self.about(ui),
        });
    }
}
