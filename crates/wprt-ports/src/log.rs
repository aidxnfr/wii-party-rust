// port of engine/src/core/log.cpp

use std::fs::File;
use std::io::Write;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

struct LogFile {
    file: Option<File>,
    origin: Instant,
}

impl LogFile {
    // port of engine/src/core/log.cpp LogFile::LogFile
    fn new() -> Self {
        let origin = Instant::now();
        let file = std::env::var("WP_LOG_FILE").ok().and_then(|path| File::create(path).ok());
        Self { file, origin }
    }
}

static LOG: OnceLock<Mutex<LogFile>> = OnceLock::new();

// port of engine/src/core/log.cpp instance
fn instance() -> &'static Mutex<LogFile> {
    LOG.get_or_init(|| Mutex::new(LogFile::new()))
}

// port of engine/src/core/log.cpp enabled
pub fn enabled() -> bool {
    instance().lock().unwrap().file.is_some()
}

// port of engine/src/core/log.cpp format_line (the fprintf shape)
pub fn format_line(seconds: f64, category: &str, message: &str) -> String {
    format!("[{:10.3}] {:<7} {}\n", seconds, category, message)
}

// port of engine/src/core/log.cpp write
pub fn write(category: &str, message: &str) {
    let mut log_file = instance().lock().unwrap();
    if log_file.file.is_none() {
        return;
    }
    let seconds = log_file.origin.elapsed().as_secs_f64();
    let line = format_line(seconds, category, message);
    if let Some(file) = log_file.file.as_mut() {
        let _ = file.write_all(line.as_bytes());
        let _ = file.flush();
    }
}
