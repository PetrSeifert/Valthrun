use std::sync::Mutex;
use log::{Log, Record, Level, Metadata, SetLoggerError};
use once_cell::sync::Lazy;
use std::io::{self, Write};

static FRAME_LOG_BUFFER: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));
static CONSOLE_HEIGHT: Lazy<Mutex<usize>> = Lazy::new(|| Mutex::new(30)); // default height

pub struct FrameLogger;

impl Log for FrameLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let level_color = match record.level() {
                Level::Error => "\x1B[31m", // Red
                Level::Warn => "\x1B[33m",  // Yellow
                Level::Info => "\x1B[32m",  // Green
                Level::Debug => "\x1B[36m", // Cyan
                Level::Trace => "\x1B[90m", // Bright black (gray)
            };
            let reset = "\x1B[0m";
            let log_entry = format!("{}[{}]{} {}", level_color, record.level(), reset, record.args());
            FRAME_LOG_BUFFER.lock().unwrap().push(log_entry);
        }
    }

    fn flush(&self) {}
}

pub fn init() -> Result<(), SetLoggerError> {
    // Clear screen and scroll buffer
    print!("\x1B[2J\x1B[3J\x1B[H");
    io::stdout().flush().unwrap();
    log::set_logger(&FrameLogger).map(|()| log::set_max_level(log::LevelFilter::Info))
}

pub fn flush_frame_logs() {
    let mut frame_buffer = FRAME_LOG_BUFFER.lock().unwrap();
    let height = *CONSOLE_HEIGHT.lock().unwrap();
    
    // Get the last N lines
    let start = if frame_buffer.len() > height {
        frame_buffer.len() - height
    } else {
        0
    };

    // Move cursor to home position
    print!("\x1B[H");
    io::stdout().flush().unwrap();
    
    // Print each line with explicit positioning and clearing
    for (i, log) in frame_buffer.iter().skip(start).enumerate() {
        // Move to line i and clear it
        print!("\x1B[{};0H\x1B[K{}", i + 1, log);
        io::stdout().flush().unwrap();
    }

    // Clear any remaining lines
    for i in frame_buffer.len()..height {
        print!("\x1B[{};0H\x1B[K", i + 1);
        io::stdout().flush().unwrap();
    }
    
    // Clear the frame buffer
    frame_buffer.clear();
}
