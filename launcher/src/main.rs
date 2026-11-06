//! Wii Party (Rust) launcher: builds the game on your PC from your own disc, then plays it.
//! The interface is a port of the arelkair/wiiparty-recomp launcher (sidebar, Game / Settings / Controls / Licenses pages,
//! install and progress panels); see ui.rs.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
mod install;
mod paths;
mod payload;
mod play;
mod prefs;
mod tools;
mod ui;

use eframe::egui::{self, pos2, vec2, Id, Rect, UiBuilder};
use install::{Event, Options, CANCELLED, STEPS};
use paths::Layout;
use prefs::Prefs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use ui::{Font, Kind};

const GITHUB: &str = "https://github.com/aidxnfr/wii-party-rust";
const UPSTREAM: &str = "https://github.com/arelkair/wiiparty-recomp";

const SIDEBAR: f32 = 232.0;
const PAGE_PAD_X: f32 = 48.0;
const PAGE_PAD_TOP: f32 = 44.0;
const CONTENT_MAX: f32 = 720.0;
/// Rough share of the install time each step takes, for the progress bar.
const STEP_WEIGHT: [f32; 5] = [0.02, 0.10, 0.25, 0.60, 0.03];

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !args.is_empty() { std::process::exit(cli(&args)); }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1040.0, 712.0])
            .with_min_inner_size([820.0, 592.0])
            .with_decorations(false)
            .with_title("Wii Party (Rust)"),
        ..Default::default()
    };
    let _ = eframe::run_native("Wii Party (Rust)", options, Box::new(|cc| {
        ui::install_fonts(&cc.egui_ctx);
        Ok(Box::new(App::new()))
    }));
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
            let o = Options { root: p.install_dir.clone(), disc: p.disc.clone(), jobs: p.jobs, offline: p.offline, cancel: Arc::new(AtomicBool::new(false)) };
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
enum Page { Game, Settings, Controls, Licenses }
const PAGES: [(Page, &str); 4] = [(Page::Game, "Game"), (Page::Settings, "Settings"), (Page::Controls, "Controls"), (Page::Licenses, "Licenses")];

#[derive(PartialEq, Clone, Copy)]
enum StepState { Waiting, Running, Done, Failed }
#[derive(PartialEq, Clone, Copy)]
enum Outcome { None, Success, Failure, Cancelled }

struct App {
    prefs: Prefs,
    page: Page,
    page_since: f64,
    dark: Option<bool>,
    rx: Option<Receiver<Event>>,
    cancel: Arc<AtomicBool>,
    states: Vec<StepState>,
    current: Option<usize>,
    log: Vec<String>,
    outcome: Outcome,
    result: String,
    error: String,
    progress_view: bool,
    show_details: bool,
    license: usize,
    disc_line: Option<String>,
}

impl App {
    fn new() -> Self {
        Self {
            prefs: Prefs::load(), page: Page::Game, page_since: 0.0, dark: None, rx: None, cancel: Arc::new(AtomicBool::new(false)),
            states: vec![StepState::Waiting; STEPS.len()], current: None, log: Vec::new(), outcome: Outcome::None,
            result: String::new(), error: String::new(), progress_view: false, show_details: false, license: 0, disc_line: None,
        }
    }
    fn busy(&self) -> bool { self.rx.is_some() }
    fn layout(&self) -> Layout { Layout::new(&self.prefs.install_dir) }
    fn extracted(&self) -> bool { self.layout().extracted().join("sys/main.dol").exists() }
    fn outdated(&self) -> bool {
        std::fs::read_to_string(self.layout().stamp()).ok().and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            .and_then(|v| v["revision"].as_str().map(|r| r != payload::REVISION)).unwrap_or(false)
    }

    fn set_page(&mut self, ctx: &egui::Context, page: Page) {
        if page != self.page { self.page = page; self.page_since = ctx.input(|i| i.time); }
    }

    fn start_install(&mut self) {
        if self.prefs.disc.is_none() && !self.extracted() { self.result = "Choose your copy of Wii Party first.".into(); return; }
        self.prefs.save();
        self.cancel = Arc::new(AtomicBool::new(false));
        let (tx, rx) = std::sync::mpsc::channel();
        let o = Options { root: self.prefs.install_dir.clone(), disc: self.prefs.disc.clone(), jobs: self.prefs.jobs, offline: self.prefs.offline, cancel: self.cancel.clone() };
        std::thread::spawn(move || install::run(&o, tx));
        self.rx = Some(rx);
        self.log.clear();
        self.states = vec![StepState::Waiting; STEPS.len()];
        self.current = None;
        self.outcome = Outcome::None;
        self.result.clear();
        self.error.clear();
        self.progress_view = true;
        self.disc_line = None;
    }

    fn poll(&mut self) {
        let Some(rx) = &self.rx else { return };
        let mut done = None;
        while let Ok(ev) = rx.try_recv() {
            match ev {
                Event::Step(n, _, s) => {
                    if let Some(c) = self.current { self.states[c] = StepState::Done; }
                    self.current = Some(n - 1);
                    self.states[n - 1] = StepState::Running;
                    self.log.push(format!("== {s}"));
                }
                Event::Line(l) => self.log.push(l),
                Event::Done(r) => done = Some(r),
            }
        }
        if self.log.len() > 6000 { self.log.drain(..self.log.len() - 6000); }
        if let Some(r) = done {
            self.rx = None;
            match r {
                Ok(()) => {
                    for s in &mut self.states { *s = StepState::Done; }
                    self.outcome = Outcome::Success;
                    self.result = "The game is installed.".into();
                    self.progress_view = false;
                }
                Err(e) => {
                    if let Some(c) = self.current { self.states[c] = StepState::Failed; }
                    self.outcome = if e == CANCELLED { Outcome::Cancelled } else { Outcome::Failure };
                    if e != CANCELLED { self.log.push(format!("error: {e}")); self.error = e; }
                }
            }
        }
    }

    fn progress(&self) -> f32 {
        if self.outcome == Outcome::Success { return 1.0; }
        let Some(c) = self.current else { return 0.0 };
        let before: f32 = STEP_WEIGHT[..c].iter().sum();
        if self.states[c] == StepState::Done { before + STEP_WEIGHT[c] } else { before }
    }

    // ---------- frame ----------
    fn sidebar(&mut self, ui: &mut egui::Ui) {
        let p = ui::palette(ui.ctx());
        let full = ui.max_rect();
        ui.painter().rect_filled(Rect::from_min_max(pos2(full.max.x - 1.0, full.min.y), full.max), 0.0, p.separator);
        let origin = full.min + vec2(0.0, ui::TITLEBAR);
        let height = full.height() - ui::TITLEBAR;
        ui::die(ui, origin + vec2(24.0, 28.0), 40.0);
        ui::paint_text(ui, origin + vec2(76.0, 30.0), egui::Align2::LEFT_TOP, "Wii Party", Font::Semibold, 16.0, p.text);
        ui::paint_text(ui, origin + vec2(76.0, 51.0), egui::Align2::LEFT_TOP, "Rust", Font::Regular, ui::size::DETAIL, p.secondary);
        let inner = SIDEBAR - 40.0;
        let (top, step) = (100.0, 38.0);
        let index = PAGES.iter().position(|(pg, _)| *pg == self.page).unwrap_or(0) as f32;
        let at = ui::animate(ui.ctx(), Id::new("nav-indicator"), index, 0.18);
        let pill = Rect::from_min_size(origin + vec2(20.0, top + at * step), vec2(inner, 36.0));
        ui.painter().rect_filled(pill, 10.0, p.control_hover);
        for (i, (pg, name)) in PAGES.iter().enumerate() {
            let r = Rect::from_min_size(origin + vec2(20.0, top + step * i as f32), vec2(inner, 36.0));
            let clicked = ui.scope_builder(UiBuilder::new().max_rect(r).id_salt(("nav", i)), |ui| ui::nav_item(ui, name, self.page == *pg, inner)).inner;
            if clicked { self.set_page(ui.ctx(), *pg); }
        }
        let version = format!("{}  ·  {}", env!("CARGO_PKG_VERSION"), payload::REVISION);
        ui::paint_text(ui, origin + vec2(24.0, height - 36.0), egui::Align2::LEFT_TOP, &version, Font::Regular, ui::size::SMALL, p.tertiary);
    }

    fn page(&mut self, ui: &mut egui::Ui) {
        let full = ui.max_rect();
        let area = Rect::from_min_max(pos2(full.min.x, full.min.y + ui::TITLEBAR), full.max);
        let now = ui.input(|i| i.time);
        let shown = ui::ease_out(((now - self.page_since) / 0.28) as f32);
        if shown < 1.0 { ui.ctx().request_repaint(); }
        let offset = ((1.0 - shown) * 10.0).round();
        let content = (area.width() - PAGE_PAD_X * 2.0).min(CONTENT_MAX);
        let available = area.height() - PAGE_PAD_TOP - 32.0;
        let mut child = ui.new_child(UiBuilder::new().max_rect(area).id_salt(("page", self.page as u8)));
        child.multiply_opacity(shown);
        if self.page == Page::Settings || self.page == Page::Controls {
            egui::ScrollArea::vertical().id_salt(("scroll", self.page as u8)).auto_shrink([false, false]).show(&mut child, |ui| {
                ui.add_space(PAGE_PAD_TOP + offset);
                ui.horizontal_top(|ui| {
                    ui.add_space(PAGE_PAD_X);
                    ui.vertical(|ui| {
                        ui.set_width(content);
                        match self.page { Page::Settings => self.settings_page(ui, content), _ => self.controls_page(ui, content) }
                    });
                });
                ui.add_space(40.0);
            });
        } else {
            let r = Rect::from_min_size(pos2(area.min.x + PAGE_PAD_X, area.min.y + PAGE_PAD_TOP + offset), vec2(content, available));
            child.scope_builder(UiBuilder::new().max_rect(r), |ui| match self.page {
                Page::Game => self.game_page(ui, content, available),
                _ => self.licenses_page(ui, content, available),
            });
        }
    }

    fn heading(ui: &mut egui::Ui, title: &str, intro: &str, width: f32) {
        let p = ui::palette(ui.ctx());
        ui::text(ui, title, Font::Semibold, ui::size::HEADING, p.text, None);
        if !intro.is_empty() {
            ui::gap(ui, 8.0);
            ui::text(ui, intro, Font::Regular, ui::size::LEAD, p.secondary, Some(width.min(620.0)));
        }
        ui::gap(ui, 28.0);
    }

    // ---------- Game ----------
    fn game_page(&mut self, ui: &mut egui::Ui, width: f32, height: f32) {
        if self.progress_view { self.progress_panel(ui, width, height); }
        else if play::installed(&self.layout()) { self.play_panel(ui, width, height); }
        else { self.install_panel(ui, width); }
    }

    fn disc_line(&mut self) -> String {
        if self.disc_line.is_none() {
            let boot = std::fs::read(self.layout().extracted().join("sys/boot.bin")).unwrap_or_default();
            let id: String = boot.iter().take(6).map(|&b| b as char).filter(|c| c.is_ascii_alphanumeric()).collect();
            let region = match id.chars().nth(3) { Some('E') => "USA", Some('P') => "Europe", Some('J') => "Japan", Some('K') => "Korea", _ => "" };
            self.disc_line = Some(if region.is_empty() { id } else { format!("{id}  ·  {region}") });
        }
        self.disc_line.clone().unwrap_or_default()
    }

    fn play_panel(&mut self, ui: &mut egui::Ui, width: f32, height: f32) {
        let p = ui::palette(ui.ctx());
        let start = ui.cursor().min;
        let outdated = self.outdated();
        let hero = (72.0 + 28.0 + 48.0 + 6.0 + 20.0 + 24.0 + 32.0 + 48.0) + if outdated { 36.0 + 96.0 } else { 0.0 };
        ui.add_space(((height - 40.0 - hero) * 0.42).max(0.0).round());
        ui::die(ui, ui.cursor().min, 72.0);
        ui.allocate_exact_size(vec2(72.0, 72.0), egui::Sense::hover());
        ui::gap(ui, 28.0);
        ui::text(ui, "Wii Party", Font::Semibold, ui::size::DISPLAY, p.text, None);
        ui::gap(ui, 6.0);
        ui::text(ui, "Ready to play.", Font::Regular, ui::size::LEAD, p.secondary, None);
        let line = self.disc_line();
        if !line.is_empty() {
            ui::gap(ui, 6.0);
            ui::text(ui, &line, Font::Regular, ui::size::DETAIL, p.tertiary, Some(width));
        }
        ui::gap(ui, 26.0);
        if ui::button(ui, "Play", Kind::Primary, 180.0, true, 48.0) {
            match play::play(&self.layout(), &self.prefs) {
                Ok(()) => { self.result.clear(); }
                Err(e) => { self.result = e; self.outcome = Outcome::Failure; }
            }
        }
        if !self.result.is_empty() {
            ui::gap(ui, 14.0);
            let col = if self.outcome == Outcome::Failure { p.danger } else { p.secondary };
            ui::text(ui, &self.result, Font::Regular, ui::size::DETAIL, col, Some(width));
        }
        if outdated {
            ui::gap(ui, 36.0);
            let mut update = false;
            ui::card(ui, width, |ui| {
                ui::gap(ui, 18.0);
                ui.horizontal_top(|ui| {
                    ui.add_space(20.0);
                    ui.vertical(|ui| {
                        ui::text(ui, "An update is ready", Font::Semibold, ui::size::BODY, p.text, None);
                        ui::gap(ui, 4.0);
                        ui::text(ui, "This launcher brings a newer version of the game. Install it to rebuild the game; your saves and settings stay as they are.",
                                 Font::Regular, ui::size::DETAIL, p.secondary, Some(width - 180.0));
                    });
                    let r = Rect::from_min_size(pos2(start.x + width - 20.0 - 110.0, ui.min_rect().center().y - 18.0), vec2(110.0, 36.0));
                    update = ui::button(&mut ui.new_child(UiBuilder::new().max_rect(r)), "Update", Kind::Secondary, 110.0, !self.busy(), 36.0);
                });
                ui::gap(ui, 18.0);
            });
            if update { self.start_install(); }
        }
        let y = start.y + height - 24.0;
        let footer = Rect::from_min_size(pos2(start.x, y), vec2(width, 24.0));
        ui.scope_builder(UiBuilder::new().max_rect(footer).layout(egui::Layout::left_to_right(egui::Align::Center)), |ui| {
            for (key, name) in [("F10", "Options"), ("F11", "Full screen")] {
                ui::keycap(ui, key);
                ui.add_space(8.0);
                ui::text(ui, name, Font::Regular, ui::size::DETAIL, p.secondary, None);
                ui.add_space(24.0);
            }
        });
    }

    fn install_panel(&mut self, ui: &mut egui::Ui, width: f32) {
        let p = ui::palette(ui.ctx());
        ui::die(ui, ui.cursor().min, 56.0);
        ui.allocate_exact_size(vec2(56.0, 56.0), egui::Sense::hover());
        ui::gap(ui, 24.0);
        ui::text(ui, "Wii Party", Font::Semibold, ui::size::DISPLAY, p.text, None);
        ui::gap(ui, 6.0);
        ui::text(ui, "Choose your copy of Wii Party to install it.", Font::Regular, ui::size::LEAD, p.secondary, Some(width));
        ui::gap(ui, 28.0);
        let extracted = self.extracted();
        let busy = self.busy();
        let mut change_folder = false;
        let mut choose_disc = false;
        ui::card(ui, width, |ui| {
            let line = |ui: &mut egui::Ui, label: &str, shown: &str, muted: bool, action: &str, enabled: bool| -> bool {
                let pad = 18.0;
                let top = ui.cursor().min;
                let fitted = ui::fit_start(ui, shown, Font::Regular, ui::size::BODY, width - pad * 2.0 - 130.0);
                let lh = ui::text_size(ui, label, Font::Semibold, ui::size::DETAIL, None).y;
                let vh = ui::text_size(ui, &fitted, Font::Regular, ui::size::BODY, None).y;
                let h = 16.0 + lh + 4.0 + vh + 16.0;
                ui.allocate_rect(Rect::from_min_size(top, vec2(width, h)), egui::Sense::hover());
                ui::paint_text(ui, top + vec2(pad, 16.0), egui::Align2::LEFT_TOP, label, Font::Semibold, ui::size::DETAIL, p.secondary);
                ui::paint_text(ui, top + vec2(pad, 16.0 + lh + 4.0), egui::Align2::LEFT_TOP, &fitted, Font::Regular, ui::size::BODY, if muted { p.tertiary } else { p.text });
                let r = Rect::from_min_size(pos2(top.x + width - pad - 100.0, top.y + ((h - 36.0) * 0.5).round()), vec2(100.0, 36.0));
                ui::button(&mut ui.new_child(UiBuilder::new().max_rect(r).id_salt(label)), action, Kind::Secondary, 100.0, enabled, 36.0)
            };
            let folder = self.prefs.install_dir.display().to_string();
            change_folder = line(ui, "Install folder", &folder, false, "Change", !busy);
            ui::inset_separator(ui, width);
            if extracted {
                line(ui, "Disc", "Already extracted", false, "Choose", false);
            } else {
                let name = self.prefs.disc.as_ref().and_then(|d| d.file_name()).map(|n| n.to_string_lossy().to_string());
                choose_disc = line(ui, "Disc", name.as_deref().unwrap_or("No disc chosen"), name.is_none(), "Choose", !busy);
            }
        });
        if change_folder {
            if let Some(d) = rfd::FileDialog::new().pick_folder() {
                self.prefs.install_dir = if d.file_name().is_some_and(|n| n == "WiiPartyRust") { d } else { d.join("WiiPartyRust") };
                self.prefs.save();
                self.disc_line = None;
            }
        }
        if choose_disc {
            if let Some(f) = rfd::FileDialog::new().set_title("Wii disc images").add_filter("Wii disc images", &["iso", "wbfs", "rvz", "wia", "ciso", "gcm"]).pick_file() {
                self.prefs.disc = Some(f);
                self.prefs.save();
                self.result.clear();
            }
        }
        ui::gap(ui, 16.0);
        let need = 25.0;
        if let Some(free) = paths::free_gigabytes(&self.prefs.install_dir) {
            let enough = free >= need;
            let s = if enough { format!("Needs about {need:.0} GB of free space. That drive has {free:.0} GB free.") }
                    else { format!("Not enough free space: the installation needs about {need:.0} GB and that drive has {free:.0} GB free.") };
            ui::text(ui, &s, Font::Semibold, ui::size::DETAIL, if enough { p.text } else { p.danger }, Some(width.min(620.0)));
            ui::gap(ui, 6.0);
        }
        ui::text(ui, "Installing translates the game from your own disc into a native program. Missing tools are downloaded from their official sites; \
                      nothing from the game is downloaded and nothing leaves your computer. The first install takes about 30 to 90 minutes, depending on your processor.",
                 Font::Regular, ui::size::DETAIL, p.secondary, Some(width.min(620.0)));
        ui::gap(ui, 28.0);
        if ui::button(ui, "Install", Kind::Primary, 160.0, !busy, 44.0) { self.start_install(); }
        if !self.result.is_empty() {
            ui::gap(ui, 14.0);
            ui::text(ui, &self.result, Font::Regular, ui::size::DETAIL, p.danger, Some(width));
        }
    }

    fn progress_panel(&mut self, ui: &mut egui::Ui, width: f32, height: f32) {
        let p = ui::palette(ui.ctx());
        let running = self.busy();
        let start = ui.cursor().min;
        ui::text(ui, if running { "Installing" } else { "Install" }, Font::Semibold, ui::size::HEADING, p.text, None);
        ui::gap(ui, 8.0);
        let status = if running { self.current.map(|c| STEPS[c].to_string()).unwrap_or_else(|| "Starting".into()) }
                     else if self.outcome == Outcome::Cancelled { "The installation was cancelled.".into() }
                     else { "The installation stopped at a failed step. The details show the error.".into() };
        let col = if !running && self.outcome == Outcome::Failure { p.danger } else { p.secondary };
        ui::text(ui, &status, Font::Regular, ui::size::LEAD, col, Some(width));
        if !running && !self.error.is_empty() {
            ui::gap(ui, 4.0);
            ui::text(ui, &self.error, Font::Regular, ui::size::DETAIL, p.danger, Some(width));
        }
        ui::gap(ui, 20.0);
        let frac = self.progress();
        ui::progress_bar(ui, Id::new("install-progress"), frac, width, 6.0);
        ui::gap(ui, 10.0);
        ui::text(ui, &format!("{}%", (frac * 100.0).round() as i32), Font::Semibold, ui::size::DETAIL, p.text, None);
        ui::gap(ui, 24.0);

        let footer = 36.0 + 24.0;
        let top = ui.cursor().min.y;
        let room = start.y + height - footer - top;
        let n = STEPS.len() as f32;
        let steps_h = (if self.show_details { (room * 0.42).max(120.0) } else { room }).min(30.0 * n + 16.0);
        ui::card(ui, width, |ui| {
            ui::gap(ui, 8.0);
            for (i, title) in STEPS.iter().enumerate() {
                let state = self.states[i];
                let (rect, _) = ui.allocate_exact_size(vec2(width, 30.0), egui::Sense::hover());
                let r = rect.shrink2(vec2(8.0, 0.0));
                if state == StepState::Running { ui.painter().rect_filled(r, 8.0, p.control); }
                let c = pos2(r.min.x + 18.0, r.center().y);
                match state {
                    StepState::Running => ui::spinner(ui, c, 6.5, p.accent),
                    StepState::Done => ui::check_mark(ui, c, 12.0, p.text),
                    StepState::Failed => ui::cross_mark(ui, c, 12.0, p.danger),
                    StepState::Waiting => { ui.painter().circle_filled(c, 2.5, p.tertiary); }
                }
                let ink = match state { StepState::Waiting => p.tertiary, StepState::Failed => p.danger, _ => p.text };
                let f = if state == StepState::Running { Font::Semibold } else { Font::Regular };
                ui::paint_text(ui, pos2(r.min.x + 36.0, r.center().y), egui::Align2::LEFT_CENTER, title, f, ui::size::BODY - 0.5, ink);
            }
            ui::gap(ui, (steps_h - 30.0 * n - 8.0).max(8.0));
        });
        if self.show_details {
            ui::gap(ui, 12.0);
            let log_h = (room - steps_h - 12.0).max(80.0);
            egui::Frame::new().fill(p.surface).corner_radius(14).inner_margin(egui::Margin::symmetric(14, 10)).show(ui, |ui| {
                ui.set_width(width - 28.0);
                egui::ScrollArea::both().id_salt("log").max_height(log_h - 20.0).auto_shrink([false, false]).stick_to_bottom(true).show(ui, |ui| {
                    for l in &self.log { ui.add(egui::Label::new(egui::RichText::new(l).font(ui::font(Font::Regular, ui::size::DETAIL)).color(p.secondary)).extend()); }
                });
            });
        }

        let y = start.y + height - 36.0;
        let at = |x: f32, w: f32| Rect::from_min_size(pos2(x, y), vec2(w, 36.0));
        let details = if self.show_details { "Hide details" } else { "Show details" };
        let dw = ui::text_size(ui, details, Font::Semibold, ui::size::BODY, None).x + 40.0;
        if ui.scope_builder(UiBuilder::new().max_rect(at(start.x, dw)), |ui| ui::button(ui, details, Kind::Ghost, 0.0, true, 36.0)).inner {
            self.show_details = !self.show_details;
        }
        if running {
            if ui.scope_builder(UiBuilder::new().max_rect(at(start.x + width - 120.0, 120.0)), |ui| ui::button(ui, "Cancel", Kind::Secondary, 120.0, !self.cancel.load(Ordering::Relaxed), 36.0)).inner {
                self.cancel.store(true, Ordering::Relaxed);
                self.log.push("cancelling...".into());
            }
        } else {
            if ui.scope_builder(UiBuilder::new().max_rect(at(start.x + width - 120.0 - 10.0 - 110.0, 110.0)), |ui| ui::button(ui, "Back", Kind::Ghost, 110.0, true, 36.0)).inner {
                self.progress_view = false;
                self.result.clear();
            }
            if ui.scope_builder(UiBuilder::new().max_rect(at(start.x + width - 120.0, 120.0)), |ui| ui::button(ui, "Try again", Kind::Primary, 120.0, true, 36.0)).inner {
                self.start_install();
            }
        }
    }

    // ---------- Settings ----------
    fn settings_page(&mut self, ui: &mut egui::Ui, width: f32) {
        Self::heading(ui, "Settings", "Choices for the game and the launcher. Game choices apply the next time the game starts; F10 opens more options while playing.", width);
        let p = ui::palette(ui.ctx());
        let before = self.prefs.clone();

        ui::group_title(ui, "Game");
        ui::card(ui, width, |ui| {
            let labels: Vec<String> = ["Off", "2x (120 Hz)", "3x (180 Hz)", "4x (240 Hz)"].iter().map(|s| s.to_string()).collect();
            let mut index = (self.prefs.smooth_motion as usize).min(3);
            ui::row(ui, width, "Smooth motion", "Draws extra in-between frames for high refresh rate screens. The game itself stays at 60. Adds a little delay and can show small glitches.", 200.0, 36.0, |ui| {
                if ui::dropdown(ui, "smooth", &labels, &mut index, 200.0) { self.prefs.smooth_motion = index as u32; }
            });
            ui::inset_separator(ui, width);
            ui::row(ui, width, "Performance window", "Opens the stats window next to the game.", 42.0, 24.0, |ui| { ui::toggle(ui, &mut self.prefs.show_stats); });
        });
        ui::gap(ui, 28.0);

        ui::group_title(ui, "Installation");
        let busy = self.busy();
        let data = self.layout().data();
        let mut change_folder = false;
        ui::card(ui, width, |ui| {
            let cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(8).max(1);
            let labels: Vec<String> = (1..=cores).map(|n| n.to_string()).collect();
            let mut index = self.prefs.jobs.clamp(1, cores) - 1;
            ui::row(ui, width, "Compile jobs", "How many parts of the game are compiled at once. Each needs about 3 GB of memory.", 200.0, 36.0, |ui| {
                if ui::dropdown(ui, "jobs", &labels, &mut index, 200.0) { self.prefs.jobs = index + 1; }
            });
            ui::inset_separator(ui, width);
            ui::row(ui, width, "Offline mode", "Use only the tools already in the downloads folder.", 42.0, 24.0, |ui| { ui::toggle(ui, &mut self.prefs.offline); });
            ui::inset_separator(ui, width);
            let folder = ui::fit_start(ui, &self.prefs.install_dir.display().to_string(), Font::Regular, ui::size::DETAIL, width - 36.0 - 100.0 - 24.0);
            ui::row(ui, width, "Install folder", &folder, 100.0, 36.0, |ui| { change_folder = ui::button(ui, "Change", Kind::Secondary, 100.0, !busy, 36.0); });
            ui::inset_separator(ui, width);
            let shown = ui::fit_start(ui, &data.display().to_string(), Font::Regular, ui::size::DETAIL, width - 36.0 - 120.0 - 24.0);
            ui::row(ui, width, "Saves and settings", &shown, 120.0, 36.0, |ui| {
                if ui::button(ui, "Open folder", Kind::Secondary, 120.0, data.exists(), 36.0) { let _ = open::that(&data); }
            });
        });
        if change_folder {
            if let Some(d) = rfd::FileDialog::new().pick_folder() {
                self.prefs.install_dir = if d.file_name().is_some_and(|n| n == "WiiPartyRust") { d } else { d.join("WiiPartyRust") };
                self.disc_line = None;
            }
        }
        ui::gap(ui, 28.0);

        ui::group_title(ui, "Launcher");
        ui::card(ui, width, |ui| {
            let themes = ["system", "light", "dark"];
            let labels: Vec<String> = ["Match system", "Light", "Dark"].iter().map(|s| s.to_string()).collect();
            let mut index = themes.iter().position(|t| *t == self.prefs.theme).unwrap_or(0);
            ui::row(ui, width, "Appearance", "", 200.0, 36.0, |ui| {
                if ui::dropdown(ui, "theme", &labels, &mut index, 200.0) { self.prefs.theme = themes[index].into(); }
            });
            ui::inset_separator(ui, width);
            ui::row(ui, width, "Project page", GITHUB, 100.0, 36.0, |ui| { if ui::button(ui, "Open", Kind::Secondary, 100.0, true, 36.0) { let _ = open::that(GITHUB); } });
        });
        let _ = p;
        if serde_json::to_string(&before).ok() != serde_json::to_string(&self.prefs).ok() { self.prefs.save(); }
    }

    // ---------- Controls ----------
    fn controls_page(&mut self, ui: &mut egui::Ui, width: f32) {
        Self::heading(ui, "Controls", "Keyboard and mouse play as Wii Remote 1. Game controllers act as Wii Remotes 1 to 4. Key bindings can be changed in the game options (F10).", width);
        let rows: &[(&str, &[&str])] = &[
            ("Point", &["Mouse"]), ("A", &["Enter", "Space"]), ("B", &["Backspace"]), ("A + B on the title screen", &["Enter", "Backspace"]),
            ("D-pad", &["W", "A", "S", "D", "Arrows"]), ("HOME", &["H"]), ("Shake", &["Left Shift", "Middle mouse"]),
            ("Swing", &["T", "G", "Wheel"]), ("Tilt", &["Q", "E", "R", "F"]), ("Grip", &["Tab"]), ("Options", &["F10"]), ("Full screen", &["F11"]),
        ];
        ui::card(ui, width, |ui| {
            for (i, (action, keys)) in rows.iter().enumerate() {
                if i > 0 { ui::inset_separator(ui, width); }
                ui::row(ui, width, action, "", 300.0, 24.0, |ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        for (k, key) in keys.iter().rev().enumerate() {
                            if k > 0 { ui.add_space(6.0); }
                            ui::keycap(ui, key);
                        }
                    });
                });
            }
        });
    }

    // ---------- Licenses ----------
    fn licenses_page(&mut self, ui: &mut egui::Ui, width: f32, height: f32) {
        let p = ui::palette(ui.ctx());
        let start = ui.cursor().min;
        Self::heading(ui, "Licenses", "Wii Party (Rust) is free software under the GNU GPL 3.0. It is built on arelkair's wiiparty-recomp and the projects below.", width);
        let names = ["Credits", "This project", "Third-party notices", "Inter typeface"];
        let list_w = 210.0;
        let top = ui.cursor().min;
        let room = start.y + height - top.y;
        for (i, n) in names.iter().enumerate() {
            let r = Rect::from_min_size(top + vec2(0.0, i as f32 * 36.0), vec2(list_w, 34.0));
            if ui::list_item(&mut ui.new_child(UiBuilder::new().max_rect(r).id_salt(("license", i))), n, self.license == i, list_w) { self.license = i; }
        }
        let body = Rect::from_min_size(top + vec2(list_w + 20.0, 0.0), vec2(width - list_w - 20.0, room.max(120.0)));
        ui.painter().rect_filled(body, 14.0, p.surface);
        let inner = body.shrink2(vec2(20.0, 18.0));
        ui.scope_builder(UiBuilder::new().max_rect(inner).id_salt("license-body"), |ui| {
            egui::ScrollArea::vertical().id_salt(("license-scroll", self.license)).auto_shrink([false, false]).show(ui, |ui| {
                ui.set_width(inner.width());
                ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                let para = |ui: &mut egui::Ui, s: &str| { ui::text(ui, s, Font::Regular, ui::size::DETAIL, p.text, Some(inner.width())); ui::gap(ui, 10.0); };
                match self.license {
                    0 => {
                        ui::text(ui, "wiiparty-recomp by arelkair", Font::Semibold, ui::size::BODY, p.text, None);
                        ui::gap(ui, 4.0);
                        para(ui, "The static recompiler, runtime design, research and launcher this project is built on. Wii Party (Rust) would not exist without it.");
                        ui.hyperlink_to(egui::RichText::new(UPSTREAM).font(ui::font(Font::Regular, ui::size::DETAIL)).color(p.accent), UPSTREAM);
                        ui::gap(ui, 18.0);
                        ui::text(ui, "Wii Party (Rust)", Font::Semibold, ui::size::BODY, p.text, None);
                        ui::gap(ui, 4.0);
                        para(ui, &format!("Version {} ({}). A Rust and Bevy port of Wii Party for PC.", env!("CARGO_PKG_VERSION"), payload::REVISION));
                        ui.hyperlink_to(egui::RichText::new(GITHUB).font(ui::font(Font::Regular, ui::size::DETAIL)).color(p.accent), GITHUB);
                        ui::gap(ui, 18.0);
                        ui::text(ui, "Also uses", Font::Semibold, ui::size::BODY, p.text, None);
                        ui::gap(ui, 4.0);
                        para(ui, "Dolphin (DSP and GX references), nod, SDL, Bevy, wgpu, egui and the Inter typeface. See Third-party notices.");
                        para(ui, "Not affiliated with or endorsed by Nintendo. You need your own legally obtained copy of the game.");
                    }
                    1 => para(ui, include_str!("../../LICENSE")),
                    2 => para(ui, &plain_markdown(include_str!("../../THIRD_PARTY_NOTICES.md"))),
                    _ => para(ui, include_str!("../assets/fonts/OFL.txt")),
                }
            });
        });
    }
}

/// Markdown shown as plain text: drop emphasis, code ticks and link targets.
fn plain_markdown(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for line in s.lines() {
        let line = line.trim_start_matches('#').trim_start();
        let mut rest = line.replace("**", "").replace('`', "");
        let mut shown = String::new();
        while let Some(a) = rest.find('[') {
            match (rest[a..].find("]("), rest[a..].find(')')) {
                (Some(m), Some(e)) if m < e => { shown.push_str(&rest[..a]); shown.push_str(&rest[a + 1..a + m]); rest = rest[a + e + 1..].to_string(); }
                _ => { shown.push_str(&rest[..=a]); rest = rest[a + 1..].to_string(); }
            }
        }
        shown.push_str(&rest);
        out.push_str(&shown);
        out.push('\n');
    }
    out
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll();
        if self.busy() { ctx.request_repaint_after(std::time::Duration::from_millis(100)); }
        let dark = match self.prefs.theme.as_str() { "dark" => true, "light" => false, _ => ctx.system_theme() == Some(egui::Theme::Dark) };
        if self.dark != Some(dark) { ui::apply_style(ctx, dark); self.dark = Some(dark); }
        let p = ui::palette(ctx);
        egui::SidePanel::left("sidebar").exact_width(SIDEBAR).resizable(false).show_separator_line(false)
            .frame(egui::Frame::new().fill(p.sidebar)).show(ctx, |ui| self.sidebar(ui));
        egui::CentralPanel::default().frame(egui::Frame::new().fill(p.background)).show(ctx, |ui| self.page(ui));
        ui::window_chrome(ctx);
    }
}
