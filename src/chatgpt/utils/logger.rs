use chrono::Local;
use colored::*;
use std::sync::Mutex;

/// Logger with colored output similar to the Python version
pub struct Logger {
    lock: Mutex<()>,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            lock: Mutex::new(()),
        }
    }

    fn get_timestamp() -> String {
        Local::now().format("%H:%M:%S").to_string()
    }

    fn log(&self, _level: &str, prefix: &str, message: &str, color: Color) {
        let _guard = self.lock.lock().unwrap();

        let timestamp = Self::get_timestamp();
        let timestamp_formatted = format!("[{}]", timestamp.magenta());
        let prefix_colored = prefix.color(color).bold();

        println!(
            "{} {} {}",
            timestamp_formatted.bright_black(),
            prefix_colored,
            message
        );
    }

    pub fn success(&self, message: &str) {
        self.log("SUCCESS", "[+]", message, Color::Green);
    }

    pub fn error(&self, message: &str) {
        self.log("ERROR", "[!]", message, Color::Red);
    }

    pub fn info(&self, message: &str) {
        self.log("INFO", "[*]", message, Color::White);
    }

    pub fn warning(&self, message: &str) {
        self.log("WARNING", "[!]", message, Color::Yellow);
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

lazy_static::lazy_static! {
    pub static ref LOG: Logger = Logger::new();
}

/// Convenience macros for logging
#[macro_export]
macro_rules! log_success {
    ($($arg:tt)*) => {
        $crate::chatgpt::utils::logger::LOG.success(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::chatgpt::utils::logger::LOG.error(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::chatgpt::utils::logger::LOG.info(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_warning {
    ($($arg:tt)*) => {
        $crate::chatgpt::utils::logger::LOG.warning(&format!($($arg)*))
    };
}
