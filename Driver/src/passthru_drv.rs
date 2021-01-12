use libc::{c_char};
use std::{ffi::CString, time::Instant};
use J2534Common::*;
use crate::{channels, ioctl, logger};
use crate::comm::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use crate::channels::ChannelComm;
use crate::logger::*;

/// J2534 API Version supported - In this case 04.04
const API_VERSION: &str = "04.04";
/// DLL (Driver) version of this library
const DLL_VERSION: &str = env!("CARGO_PKG_VERSION");

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
        logger::log_info(format!("Error copying '{}' - Source ptr is null", src));
        return false
    }
    match CString::new(src) {
        Err(_) => {
            logger::log_info(format!("Error copying '{}' - CString creation failed", src));
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
        let mut msg = CommMsg::new(MsgType::GetFwVersion);
        match dev.write_and_read_ptcmd(&mut msg, 250) {
            M2Resp::Ok(args) => { Ok(String::from_utf8(args).unwrap()) },
            M2Resp::Err{status, string} => {
                log_warn(format!("M2 failed to respond to FW_VERSION request: {}", string));
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
    logger::log_info_str("PassthruOpen called");
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
                logger::log_error(format!("Cannot open com port. Error: {}", x));
                set_error_string(format!("COM Port open failed with error {}", x));
                return PassthruError::ERR_DEVICE_NOT_CONNECTED
            }
        }
    }
}

pub fn passthru_close(device_id: u32) -> PassthruError {
    logger::log_info(format!("PassthruClose called. Device ID: {}", device_id));
    // Device ID which isn't our device ID
    if device_id != DEVICE_ID {
        return PassthruError::ERR_INVALID_DEVICE_ID
    }
    if let Ok(d) = M2.write().as_deref_mut() {
        match d {
            Some(dev) => {
                // Kill all open channels if any exist
                dev.stop();
                channels::ChannelComm::force_destroy_all_channels();
                *d = None;
                return PassthruError::STATUS_NOERROR;
            },
            None => {return PassthruError::STATUS_NOERROR}
        }
    } else {
        return PassthruError::ERR_FAILED;
    }
}

pub fn passthru_connect(device_id: u32, protocol_id: u32, flags: u32, baud_rate: u32, channel_id_ptr: *mut u32) -> PassthruError {
    if device_id != DEVICE_ID {
        // Diagnostic Software messed up here. Not my device ID!
        set_error_string(format!("Not M2s device ID. Expected {}, got {}", DEVICE_ID, device_id));
        return PassthruError::ERR_DEVICE_NOT_CONNECTED;
    }
    if channel_id_ptr.is_null() {
        logger::log_error_str("Channel destination pointer is null!?");
        return PassthruError::ERR_NULL_PARAMETER;
    }

    match Protocol::from_raw(protocol_id) {
        Some(protocol) => {
            match ChannelComm::create_channel(protocol, baud_rate, flags) {
                Ok(chan_id) => {
                    unsafe { *channel_id_ptr = chan_id };
                    PassthruError::STATUS_NOERROR
                },
                Err(x) => x
            }
        },
        None => {
            logger::log_error(format!("{} is not recognised as a valid protocol ID!", protocol_id));
            PassthruError::ERR_INVALID_PROTOCOL_ID
        }
    }
}

pub fn passthru_disconnect(channel_id: u32) -> PassthruError {
    match ChannelComm::destroy_channel(channel_id as u32) {
        Ok(_) => PassthruError::STATUS_NOERROR,
        Err(e) => e
    }
}

pub fn passthru_ioctl(
    channel_id: u32,
    ioctl_id: u32,
    input_ptr: *mut libc::c_void,
    output_ptr: *mut libc::c_void,
) -> PassthruError {
    let ioctl_opt = match IoctlID::from_raw(ioctl_id) {
        Some(p) => p,
        None => {
            log_error(format!("IOCTL Param {:08X} is invalid", ioctl_id));
            return PassthruError::ERR_INVALID_IOCTL_ID
        }
    };

    match ioctl_opt {
        IoctlID::READ_VBATT => {
            if output_ptr.is_null() {
                log_error_str("Cannot read battery voltage. Output ptr is null");
                return PassthruError::ERR_NULL_PARAMETER 
            }
            ioctl::read_vbatt(output_ptr as *mut u32)
        },
        IoctlID::READ_PROG_VOLTAGE => {
            if output_ptr.is_null() {
                log_error_str("Cannot read programming voltage. Output ptr is null");
                return PassthruError::ERR_NULL_PARAMETER 
            }
            ioctl::read_prog_voltage(output_ptr as *mut u32)
        }
        IoctlID::SET_CONFIG => {
            if input_ptr.is_null() {
                log_error_str("Cannot set config. Input ptr is null");
                return PassthruError::ERR_NULL_PARAMETER 
            }
            ioctl::set_config(channel_id, unsafe { (input_ptr as *mut SConfigList).as_ref().unwrap() })
        }

        IoctlID::GET_CONFIG => {
            if input_ptr.is_null() {
                log_error_str("Cannot get config. Input ptr is null");
                return PassthruError::ERR_NULL_PARAMETER 
            }
            ioctl::get_config(channel_id, unsafe { (input_ptr as *mut SConfigList).as_ref().unwrap() })
        }
        IoctlID::FIVE_BAUD_INIT => {
            if input_ptr.is_null() {
                log_error_str("Cannot run five baud init. Input ptr is null");
                return PassthruError::ERR_NULL_PARAMETER 
            }
            if output_ptr.is_null() {
                log_error_str("Cannot run five baud init. Input ptr is null");
                return PassthruError::ERR_NULL_PARAMETER 
            }
            ioctl::five_baud_init(
                channel_id, 
                unsafe { (input_ptr as *mut SBYTE_ARRAY).as_mut().unwrap() },
                unsafe { (input_ptr as *mut SBYTE_ARRAY).as_mut().unwrap() }
            )
        },
        IoctlID::FAST_INIT => {
            if input_ptr.is_null() {
                log_error_str("Cannot run fast init. Input ptr is null");
                return PassthruError::ERR_NULL_PARAMETER 
            }
            if output_ptr.is_null() {
                log_error_str("Cannot run fast init. Input ptr is null");
                return PassthruError::ERR_NULL_PARAMETER 
            }
            ioctl::fast_init(
                channel_id, 
                unsafe { (input_ptr as *mut PASSTHRU_MSG).as_mut().unwrap() },
                unsafe { (input_ptr as *mut PASSTHRU_MSG).as_mut().unwrap() }
            )
        },
        IoctlID::CLEAR_TX_BUFFER => ioctl::clear_tx_buffer(channel_id),
        IoctlID::CLEAR_RX_BUFFER => ioctl::clear_rx_buffer(channel_id),
        IoctlID::CLEAR_PERIODIC_MSGS => ioctl::clear_periodic_msgs(channel_id),
        IoctlID::CLEAR_MSG_FILTERS => ioctl::clear_msg_filters(channel_id),
        IoctlID::CLEAR_FUNCT_MSG_LOOKUP_TABLE => ioctl::clear_funct_msg_lookup_table(channel_id),
        IoctlID::ADD_TO_FUNCT_MSG_LOOKUP_TABLE => {
            if input_ptr.is_null() {
                log_error_str("Cannot add to function message lookup table. Input ptr is null");
                return PassthruError::ERR_NULL_PARAMETER 
            }
            ioctl::add_to_funct_msg_lookup_table(channel_id, unsafe { (input_ptr as *mut SBYTE_ARRAY).as_mut().unwrap() })
        },
        IoctlID::DELETE_FROM_FUNCT_MSG_LOOKUP_TABLE => {
            if input_ptr.is_null() {
                log_error_str("Cannot delete from function message lookup table. Input ptr is null");
                return PassthruError::ERR_NULL_PARAMETER 
            }
            ioctl::delete_from_funct_msg_lookup_table(channel_id, unsafe { (input_ptr as *mut SBYTE_ARRAY).as_mut().unwrap() })
        }
    }
}

/// Sets a channel filter
/// # Params
/// * channel_id - Target channel to add filter to
/// * filter_type - Type of filter
///
pub fn set_channel_filter(channel_id: u32, filter_type: FilterType, mask_ptr: *const PASSTHRU_MSG, pattern_ptr: *const PASSTHRU_MSG, fc_ptr: *const PASSTHRU_MSG, msg_id_ptr: *mut u32) -> PassthruError {
    if mask_ptr.is_null() || pattern_ptr.is_null() {
        log_error_str("Mask or pattern is null!?");
        return PassthruError::ERR_NULL_PARAMETER
    }
    
    // Error - Filter is flow control yet the specified flow control message is null!?
    if filter_type == FilterType::FLOW_CONTROL_FILTER && fc_ptr.is_null() {
        return PassthruError::ERR_NULL_PARAMETER
    }

    fn log_filter(name: &str, msg: *const PASSTHRU_MSG) {
        let ptr = unsafe { msg.as_ref() };
        ptr.map(|msg| {
            logger::log_debug(format!("Filter specified. Type: {}, Data: {:?}", name, &msg.data[0..msg.data_size as usize]))
        });
    }
    log_filter("Mask filter", mask_ptr);
    log_filter("Pattern filter", pattern_ptr);
    log_filter("Flow control filter", fc_ptr);

    fn get_filter_bytes(msg: *const PASSTHRU_MSG) -> Vec<u8> {
        match unsafe { msg.as_ref() } {
            None => Vec::new(),
            Some(msg) => msg.data[0..msg.data_size as usize].to_vec()
        }
    }


    let mask: Vec<u8> = get_filter_bytes(mask_ptr);
    let pattern: Vec<u8> = get_filter_bytes(pattern_ptr);
    let flowcontrol: Vec<u8> = get_filter_bytes(fc_ptr);

    match channels::ChannelComm::create_channel_filter(channel_id, filter_type, mask.as_slice(), pattern.as_slice(), flowcontrol.as_slice()) {
        Ok(filter_id) => {
            // Assign the filter ID
            unsafe { *msg_id_ptr = filter_id };
            PassthruError::STATUS_NOERROR
        },
        Err(e) => e
    }
}

/// Stops a channel filter
/// # Params
/// * channel_id - Target channel
/// * filter_type - Filter ID
///
pub fn del_channel_filter(channel_id: u32, filter_id: u32) -> PassthruError {
    match channels::ChannelComm::remove_filter(channel_id, filter_id) {
        Ok(_) => PassthruError::STATUS_NOERROR,
        Err(e) => e
    }
}

pub fn write_msgs(channel_id: u32, msg_ptr: *const PASSTHRU_MSG, num_msg_ptr: *mut u32, timeout_ms: u32) -> PassthruError {
    if msg_ptr.is_null() || num_msg_ptr.is_null() {
        return PassthruError::ERR_NULL_PARAMETER
    }

    let max_msgs = *unsafe { num_msg_ptr.as_ref() }.unwrap() as usize;
    // Set num_msg_ptr to 0, we will increment it as reading to keep track how many messages have been written
    unsafe { *num_msg_ptr = 0 };
    let start_time = Instant::now();
    for i in 0..max_msgs {
        if timeout_ms != 0 && start_time.elapsed().as_millis() > timeout_ms as u128 { // Timeout!
            return PassthruError::ERR_TIMEOUT
        }
        let curr_msg = match unsafe { msg_ptr.offset(i as isize).as_ref() } {
            Some(m) => m,
            None => return PassthruError::ERR_NULL_PARAMETER
        };
        match channels::ChannelComm::write_channel_data(channel_id, curr_msg, timeout_ms != 0) {
            Ok(()) => {}, // Continue
            Err(e) => return e // Stop sending and return the error to the application
        }
        unsafe { *num_msg_ptr += 1 };
    }
    return PassthruError::STATUS_NOERROR
}

pub fn read_msgs(channel_id: u32, msg_ptr: *mut PASSTHRU_MSG, num_msg_ptr: *mut u32, timeout_ms: u32) -> PassthruError {
    if msg_ptr.is_null() || num_msg_ptr.is_null() {
        return PassthruError::ERR_NULL_PARAMETER
    }
    let max_msgs = *unsafe { num_msg_ptr.as_ref() }.unwrap() as usize;
    // Set num_msg_ptr to 0, we will increment it as reading to keep track how many messages have been read
    unsafe { *num_msg_ptr = 0 };
    let start_time = Instant::now();
    for i in 0..max_msgs {
        if timeout_ms != 0 && start_time.elapsed().as_millis() > timeout_ms as u128 { // Timeout!
            return PassthruError::ERR_TIMEOUT
        }
        match channels::ChannelComm::read_channel_data(channel_id) {
            Ok(opt) => {
                match opt {
                    Some(msg) => {
                        //log_debug(format!("Channel {} sending data back to application! {}", channel_id, msg));
                        unsafe { *msg_ptr.offset(i as isize) = msg; }
                        unsafe { *num_msg_ptr += 1 };
                    }
                    None => {
                        if timeout_ms == 0 {
                            return PassthruError::ERR_BUFFER_EMPTY
                        }
                    }
                }
            }
            Err(e) => return e
        }
    }
    return PassthruError::STATUS_NOERROR
}