use std::fmt::Formatter;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::sync::{Arc, Mutex};
use lazy_static::*;

#[cfg(windows)]
const LOG_PATH: &str = "C:\\Program Files (x86)\\macchina\\passthru\\driver_log.txt";

#[cfg(unix)]
const LOG_PATH: &str = "driver_log.txt";

lazy_static! {
    static ref LOGGER : Mutex<Logger> = Mutex::new(Logger::new());
}

/// Logs an info message
pub fn info(msg: String) {
    LOGGER.lock().unwrap().write_to_file(format!("[INFO]  - {}", msg));
}

/// Logs a warn message
pub fn warn(msg: String) {
    LOGGER.lock().unwrap().write_to_file(format!("[WARN]  - {}", msg));
}

/// Logs an error message
pub fn error(msg: String) {
    LOGGER.lock().unwrap().write_to_file(format!("[ERROR] - {}", msg));
}

/// Logs a message coming from the M2 unit itself
pub fn log_m2(msg: String) {
    LOGGER.lock().unwrap().write_to_file(format!("[M2_LG] - {}", msg));
}


pub struct Logger {}

impl Logger {
    fn new() -> Self {
        Logger{}
    }

    #[cfg(not(test))]
    // Not test mode - Write to file
    fn write_to_file(&self, txt: String) {
        if !Path::exists(Path::new(LOG_PATH)) {
            if let Err(x) = File::create(LOG_PATH) {
                eprintln!("LOG FILE CREATE ERROR! [{}]", x);
            }
        }
        println!("{}", txt);

        /*
        let mut ops = std::fs::OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(LOG_PATH)
            .unwrap();

        if let Err(e) = writeln!(ops, "{}", txt) {
            eprintln!("WRITE ERROR! [{}] - '{}'", e, txt);
        }
        */
        // Mutex gets unlocked at end of scope
    }

    #[cfg(test)]
    // In test mode we print to stdout
    fn write_to_file(&self, txt: String) {
        println!("{}", txt);
    }
}
