use std::{sync::RwLock};
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::sync::{Arc, Mutex};
use lazy_static::*;
use std::sync::mpsc::{channel, Sender, Receiver};

#[cfg(windows)]
const LOG_PATH: &str = "C:\\Program Files (x86)\\macchina\\passthru\\macchina_log.txt";

#[cfg(unix)]
const LOG_PATH: &str = "macchina_log.txt";

lazy_static! {
    static ref LOGGER : RwLock<Logger> = RwLock::new(Logger::new());
}

/// Logs an info message
pub fn log_debug(msg: String) {
    LOGGER.read().unwrap().queue_msg(format!("[DEBUG] - {}", msg))
}

pub fn log_debug_str(msg: &str) {
    log_debug(msg.to_string())
}

pub fn log_error(msg: String) {
    LOGGER.read().unwrap().queue_msg(format!("[ERROR] - {}", msg))
}

pub fn log_error_str(msg: &str) {
    log_error(msg.to_string())
}

pub fn log_warn(msg: String) {
    LOGGER.read().unwrap().queue_msg(format!("[WARN ] - {}", msg))
}

pub fn log_warn_str(msg: &str) {
    log_warn(msg.to_string())
}

pub fn log_info(msg: String) {
    LOGGER.read().unwrap().queue_msg(format!("[INFO ] - {}", msg))
}

pub fn log_info_str(msg: &str) {
    log_info(msg.to_string())
}

pub fn log_m2_msg(msg: String) {
    LOGGER.read().unwrap().queue_msg(format!("[M2LOG] - {}", msg))
}


pub struct Logger {
    tx_queue: Arc<Mutex<Sender<String>>>
}

impl Logger {
    fn new() -> Self {
        let (tx, rx): (Sender<String>, Receiver<String>) = channel();
        std::thread::spawn(move||{
            loop {
                if let Ok(s) = rx.recv() {
                    Logger::write_to_file(s);
                }
            }
        });
        Logger{
            tx_queue: Arc::new(Mutex::new(tx))
        }
    }

    #[allow(unused_must_use)]
    pub fn queue_msg(&self, msg: String) {
        self.tx_queue.lock().unwrap().send(msg);
    }


    #[cfg(not(test))]
    // Not test mode - Write to file
    fn write_to_file(txt: String) {
        if !Path::exists(Path::new(LOG_PATH)) {
            if let Err(x) = File::create(LOG_PATH) {
                eprintln!("LOG FILE CREATE ERROR! [{}]", x);
            }
        }
        println!("{}", txt);
        let mut ops = std::fs::OpenOptions::new()
            .write(true)
            .append(true)
            .create(false)
            .open(LOG_PATH)
            .unwrap();

        if let Err(e) = writeln!(ops, "{}", txt) {
            eprintln!("WRITE ERROR! [{}] - '{}'", e, txt);
        }
        // Mutex gets unlocked at end of scope
    }

    #[cfg(test)]
    // In test mode we print to stdout
    fn write_to_file(txt: String) {
        println!("{}", txt);
    }
}
