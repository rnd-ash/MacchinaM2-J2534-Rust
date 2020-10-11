use std::fmt::Formatter;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use winapi::*;

#[cfg(target_family = "windows")]
const LOG_PATH: &str = "C:\\Program Files (x86)\\macchina\\passthru\\driver_log.txt";
#[cfg(target_family = "unix")]
const LOG_PATH: &str = "/usr/share/macchina/passthru/driver_log.txt";

struct Logger {}

#[cfg(target_family = "windows")]
impl Logger {}
