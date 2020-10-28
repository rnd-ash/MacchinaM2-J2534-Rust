use libc::{c_char, c_long};
use std::ffi::CString;
use J2534Common::*;
use crate::logger;
use crate::comm::*;

/// J2534 API Version supported - In this case 04.04
const API_VERSION: &str = "04.04";
/// DLL (Driver) version of this library
const DLL_VERSION: &str = "0.1";
/// Firmware version of the ODB2 hardware
const FW_VERSION: &str = "0.1";


/// Our device ID that will be returned back to the application (0x1234)
const DEVICE_ID: u32 = 0x1234;

fn copy_str_unsafe(dst: *mut c_char, src: &str) -> bool {
    if dst.is_null() {
        logger::info(format!("Error copying '{}' - Source ptr is null", src));
        return false
    }
    match CString::new(src) {
        Err(_) => {
            logger::info(format!("Error copying '{}' - CString creation failed", src));
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

    if !copy_str_unsafe(fw_version_ptr, FW_VERSION) {
        return PassthruError::ERR_FAILED
    }
    if !copy_str_unsafe(api_version_ptr, API_VERSION) {
        return PassthruError::ERR_FAILED
    }
    if !copy_str_unsafe(dll_version_ptr, DLL_VERSION) {
        return PassthruError::ERR_FAILED
    }
    PassthruError::STATUS_NOERROR
}


pub fn passthru_open(device_id: *mut u32) -> PassthruError {
    logger::info("PassthruOpen called".to_string());
    if M2.read().unwrap().is_some() {
        return PassthruError::ERR_DEVICE_IN_USE;
    } else {
        match MacchinaM2::open_connection() {
            Ok(dev) => {
                if let Ok(ptr) = M2.write().as_deref_mut() {
                    *ptr = Some(dev);
                    unsafe { *device_id = DEVICE_ID };
                    return PassthruError::STATUS_NOERROR;
                }
                return PassthruError::ERR_FAILED;
            }
            Err(x) => return PassthruError::ERR_DEVICE_NOT_CONNECTED
        }
    }
}

pub fn passthru_close(pDeviceID: u32) -> PassthruError {
    logger::info(format!("PassthruClose called. Device ID: {}", pDeviceID));
    // Device ID which isn't our device ID
    if pDeviceID != DEVICE_ID {
        return PassthruError::ERR_INVALID_DEVICE_ID
    }
    if M2.read().unwrap().is_none() {
        return PassthruError::ERR_FAILED;
    } else {
        if let Ok(ptr) = M2.write().as_deref_mut() {
            *ptr = None;
            return PassthruError::STATUS_NOERROR;
        }
        return PassthruError::ERR_FAILED;
    }
}