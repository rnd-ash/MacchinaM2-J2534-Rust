use libc::{c_char, c_long};
use std::ffi::CString;
use J2534Common::*;
use crate::logger;
use crate::comm::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use crate::channels::ChannelComm;
use crate::logger::*;

/// J2534 API Version supported - In this case 04.04
const API_VERSION: &str = "04.04";
/// DLL (Driver) version of this library
const DLL_VERSION: &str = "0.1";

lazy_static! {
    pub static ref LAST_ERROR_STR: Mutex<String> = Mutex::new(String::from(""));
}

#[allow(unused_must_use)]
pub fn set_error_string(input: String) {
    let mut state = LAST_ERROR_STR.lock().unwrap();
    std::mem::replace(&mut *state, input);
}

/// Our device ID that will be returned back to the application (0x1234)
const DEVICE_ID: u32 = 0x1234;

fn copy_str_unsafe(dst: *mut c_char, src: &str) -> bool {
    if dst.is_null() {
        logger::log_info(format!("Error copying '{}' - Source ptr is null", src).as_str());
        return false
    }
    match CString::new(src) {
        Err(_) => {
            logger::log_info(format!("Error copying '{}' - CString creation failed", src).as_str());
            false
        }
        Ok(x) => {
            unsafe { std::ptr::copy_nonoverlapping(x.as_c_str().as_ptr(), dst, src.len()) };
            true
        }
    }
}

/// Copies the API_VERSION, DLL_VERSION and FW_VERSION
/// back to the pointers set by the source application
pub fn passthru_read_version(
    fw_version_ptr: *mut c_char,
    dll_version_ptr: *mut c_char,
    api_version_ptr: *mut c_char
) -> PassthruError {
    let fw_version = run_on_m2(|dev| {
        let msg = COMM_MSG::new(MsgType::GetFwVersion);
        match dev.write_and_read_ptcmd(msg, 250) {
            M2Resp::Ok(args) => { Ok(String::from_utf8(args).unwrap()) },
            M2Resp::Err{status, string} => {
                log_warn(format!("M2 failed to respond to FW_VERSION request: {}", string).as_str());
                Err(status)   
            }
        }
    });
    if fw_version.is_err() {
        return fw_version.unwrap_err();
    }

    if !copy_str_unsafe(fw_version_ptr, fw_version.unwrap().as_str()) {
        set_error_string("FW Version copy failed".to_string());
        return PassthruError::ERR_FAILED
    }
    if !copy_str_unsafe(api_version_ptr, API_VERSION) {
        set_error_string("API Version copy failed".to_string());
        return PassthruError::ERR_FAILED
    }
    if !copy_str_unsafe(dll_version_ptr, DLL_VERSION) {
        set_error_string("DLL Version copy failed".to_string());
        return PassthruError::ERR_FAILED
    }
    PassthruError::STATUS_NOERROR
}

pub fn passthru_get_last_error(dest: *mut c_char) -> PassthruError {
    match copy_str_unsafe(dest, LAST_ERROR_STR.lock().unwrap().as_str()) {
        false => PassthruError::ERR_FAILED,
        true => PassthruError::STATUS_NOERROR
    }
}


pub fn passthru_open(device_id: *mut u32) -> PassthruError {
    logger::log_info("PassthruOpen called");
    if M2.read().unwrap().is_some() {
        return PassthruError::ERR_DEVICE_IN_USE;
    } else {
        match MacchinaM2::open_connection() {
            Ok(dev) => {
                if let Ok(ptr) = M2.write().as_deref_mut() {
                    *ptr = Some(dev);
                    unsafe { std::ptr::write(device_id, DEVICE_ID) };
                    return PassthruError::STATUS_NOERROR;
                }
                set_error_string(format!("Failed to obtain write access to M2"));
                return PassthruError::ERR_FAILED;
            }
            Err(x) => {
                logger::log_error(format!("Cannot open com port. Error: {}", x).as_str());
                set_error_string(format!("COM Port open failed with error {}", x));
                return PassthruError::ERR_DEVICE_NOT_CONNECTED
            }
        }
    }
}

pub fn passthru_close(pDeviceID: u32) -> PassthruError {
    logger::log_info(&format!("PassthruClose called. Device ID: {}", pDeviceID));
    // Device ID which isn't our device ID
    if pDeviceID != DEVICE_ID {
        return PassthruError::ERR_INVALID_DEVICE_ID
    }
    if M2.read().unwrap().is_none() {
        return PassthruError::STATUS_NOERROR;
    } else {
        if let Ok(d) = M2.write().as_deref_mut() {
            match d {
                Some(dev) => dev.stop(),
                None => {return PassthruError::STATUS_NOERROR}
            }
        }

        if let Ok(ptr) = M2.write().as_deref_mut() {
            *ptr = None;
            return PassthruError::STATUS_NOERROR;
        }
        return PassthruError::ERR_FAILED;
    }
}

pub fn passthru_connect(device_id: u32, protocol_id: u32, flags: u32, baud_rate: u32, pChannelID: *mut u32) -> PassthruError {
    if device_id != DEVICE_ID {
        // Diagnostic Software messed up here. Not my device ID!
        set_error_string(format!("Not M2s device ID. Expected {}, got {}", DEVICE_ID, device_id));
        return PassthruError::ERR_DEVICE_NOT_CONNECTED;
    }
    if pChannelID.is_null() {
        logger::log_error(&"Channel destination pointer is null!?".to_string());
        PassthruError::ERR_NULL_PARAMETER;
    }

    match Protocol::from_raw(protocol_id) {
        Some(protocol) => {
            match ChannelComm::create_channel(protocol, baud_rate, flags) {
                Ok(chan_id) => {
                    unsafe { *pChannelID = chan_id };
                    PassthruError::STATUS_NOERROR
                },
                Err(x) => x
            }
        },
        None => {
            logger::log_error(&format!("{} is not recognised as a valid protocol ID!", protocol_id));
            PassthruError::ERR_INVALID_PROTOCOL_ID
        }
    }
}

pub fn passthru_ioctl(
    HandleID: u32,
    IoctlID: u32,
    pInput: *mut libc::c_void,
    pOutput: *mut libc::c_void,
) -> PassthruError {
    if IoctlID == J2534Common::IoctlID::READ_VBATT as u32 {
        logger::log_info(&format!("Getting voltage"));
        match get_batt_voltage() {
            Err(code) => {
                logger::log_warn(&format!("Error retreiving VBatt"));
                return code;
            },
            Ok(v) => {
                logger::log_info(&format!("Reported voltage: {}V", v));
                let output: &mut u32 = unsafe { &mut *(pOutput as *mut u32) };
                *output = v;
                return PassthruError::STATUS_NOERROR;
            }
        }
    }
    PassthruError::STATUS_NOERROR 
}