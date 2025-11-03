use chrono::Local;
use colored::*;

pub struct Logger;

impl Logger {
    pub fn success(message: &str) {
        Self::log("SUCCESS", "[+]", message, Color::Green);
    }

    pub fn error(message: &str) {
        Self::log("ERROR", "[!]", message, Color::Red);
    }

    pub fn info(message: &str) {
        Self::log("INFO", "[!]", message, Color::White);
    }

    fn log(_level: &str, prefix: &str, message: &str, color: Color) {
        let timestamp = Local::now().format("%H:%M:%S");
        let formatted_timestamp = format!("[{}]", timestamp).bright_black();
        let colored_prefix = prefix.color(color);

        println!("{} {} {}", formatted_timestamp, colored_prefix, message);
    }
}
