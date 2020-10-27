use std::fmt::Formatter;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::sync::{Arc, Mutex};

#[cfg(windows)]
const LOG_PATH: &str = "C:\\Program Files (x86)\\macchina\\passthru\\driver_log.txt";
#[cfg(unix)]
const LOG_PATH: &str = "/usr/share/macchina/passthru/driver_log.txt";

pub struct Logger {}
pub static LOGGER: Logger = Logger {};

impl Logger {
    pub fn info(&self, msg: String) {
        if let Err(e) = self.write_to_file(format!("[INFO] - {}", msg)) {
            eprintln!("ERROR WRITING TO LOG ({:?})", e)
        }
    }

    fn write_to_file(&self, txt: String) -> std::io::Result<()> {
        if !Path::exists(Path::new(LOG_PATH)) {
            File::create(LOG_PATH)?;
        }
        // Lock the mutex
        //let mut data = self.logMutex.lock().unwrap();
        write!(
            std::fs::OpenOptions::new()
                .write(true)
                .append(true)
                .open(LOG_PATH)?,
            "{}",
            txt
        )
        // Mutex gets unlocked at end of scope
    }
}
