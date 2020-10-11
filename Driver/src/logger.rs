use std::io;
use std::fs::File;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::fmt::Formatter;

#[cfg(target_family = "windows")]
const LOG_PATH : &str = "C:\\Program Files (x86)\\macchina\\passthru\\driver_log.txt";
#[cfg(target_family = "unix")]
const LOG_PATH : &str = "/usr/share/macchina/passthru/driver_log.txt";

pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR")
        }
    }
}




pub struct Logger {
    logFile: File
}

impl Logger {
    pub fn init() -> std::io::Result<Logger> {
        let file = File::create(LOG_PATH)?;
        Ok(Logger { logFile: file })
    }

    pub fn log_msg(&mut self, level: LogLevel, msg: String) {
        if self.logFile.write_all(format!("[{}]: {}", level, msg).as_bytes()).is_err() {
            println!("Error writing to logfile!. Msg is {}-{}", level, msg)
        }
    }
}