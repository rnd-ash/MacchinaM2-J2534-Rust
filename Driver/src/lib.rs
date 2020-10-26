use libc::{c_char, c_long};
use std::ffi::CString;
use J2534Common::*;
mod logger;
use logger::LOGGER;

const API_VERSION: &str = "04.04";
const DLL_VERSION: &str = "0.1";
const FW_VERSION: &str = "0.1";

// Dll Load function (Windows only)
#[no_mangle]
#[cfg(windows)]
#[allow(non_snake_case)]
pub extern "system" fn DllMain(module: u32, reason: u32, reserved: *mut std::ffi::c_void) -> bool {
    match reason {
        0x01 => { // Dll_PROCESS_ATTACH
            // Setup logger and one time things
            LOGGER.info(format!("Dll_PROCESS_ATTACH Called"));
            true
        }
        0x00 => { // DLL_PROCESS_DETACH
            // Destroy logger and one time things
            LOGGER.info(format!("DLL_PROCESS_DETACH Called"));
            true
        }
        0x02 => { // DLL_THREAD_ATTACH
            LOGGER.info(format!("DLL_THREAD_ATTACH Called"));
            true
        }
        0x03 => { // DLL_THREAD_DETACH
            LOGGER.info(format!("DLL_THREAD_DETACH Called"));
            true
        }
        _ => {
            LOGGER.info(format!("WTF Invalid DLL Entry {}", reason));
            false
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn PassThruOpen(name: *mut libc::c_void, device_id: *mut u32) -> i32 {
    PassthruError::STATUS_NOERROR as i32
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn PassThruClose(pDeviceID: u32) -> i32 {
    PassthruError::STATUS_NOERROR as i32
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn PassThruConnect(
    DeviceID: u32,
    ProtocolID: u32,
    Flags: u32,
    BaudRate: u32,
    pChannelID: *mut u32,
) -> i32 {
    let prot = Protocol::fromByte(ProtocolID);
    LOGGER.info(format!("PASSTHRU_CONNECT. Protocol: {:?}, Baudrate: {}", prot, BaudRate));
    PassthruError::STATUS_NOERROR as i32
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn PassThruDisconnect(ChannelID: u32) -> i32 {
    PassthruError::STATUS_NOERROR as i32
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn PassThruReadVersion(
    device_id: u32,
    fw_version_ptr: *mut c_char,
    dll_version_ptr: *mut c_char,
    api_version_ptr: *mut c_char,
) -> i32 {
    match CString::new(FW_VERSION) {
        Err(_) => return PassthruError::ERR_FAILED as i32,
        Ok(s) => unsafe {
            std::ptr::copy_nonoverlapping(s.as_c_str().as_ptr(), fw_version_ptr, FW_VERSION.len())
        },
    }
    match CString::new(DLL_VERSION) {
        Err(_) => return PassthruError::ERR_FAILED as i32,
        Ok(s) => unsafe {
            std::ptr::copy_nonoverlapping(s.as_c_str().as_ptr(), dll_version_ptr, DLL_VERSION.len())
        },
    }
    match CString::new(API_VERSION) {
        Err(_) => return PassthruError::ERR_FAILED as i32,
        Ok(s) => unsafe {
            std::ptr::copy_nonoverlapping(s.as_c_str().as_ptr(), api_version_ptr, API_VERSION.len())
        },
    }
    PassthruError::STATUS_NOERROR as i32
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn PassThruGetLastError(pErrorDescription: *mut c_char) -> i32 {
    PassthruError::STATUS_NOERROR as i32
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn PassThruReadMsgs(
    ChannelID: u32,
    pMsg: *mut PASSTHRU_MSG,
    pNumMsgs: *mut u32,
    Timeout: u32,
) -> i32 {
    PassthruError::STATUS_NOERROR as i32
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn PassThruStartMsgFilter(
    ChannelID: u32,
    FilterType: u32,
    pMaskMsg: *const PASSTHRU_MSG,
    pPatternMsg: *const PASSTHRU_MSG,
    pFlowControlMsg: *const PASSTHRU_MSG,
    pMsgID: *mut u32,
) -> i32 {
    PassthruError::STATUS_NOERROR as i32
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn PassThruStopMsgFilter(ChannelID: u32, MsgID: u32) -> i32 {
    PassthruError::STATUS_NOERROR as i32
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn PassThruWriteMsgs(
    ChannelID: u32,
    pMsg: *const PASSTHRU_MSG,
    pNumMsgs: *const u32,
    Timeout: u32,
) -> i32 {
    if let Some(ptr) = unsafe { pMsg.as_ref() } {
        let prot = match Protocol::fromByte(ptr.protocol_id) {
            Some(p) => p,
            None => return PassthruError::ERR_INVALID_PROTOCOL_ID as i32
        };
        let size = ptr.data_size;
        let data = &ptr.data[0..size as usize];
        LOGGER.info(format!("WRITE_MSGS. Protocol: {:?}, Data size: {} {:x?}. Timeout {} ms", prot, size, data, Timeout));  
    }
    PassthruError::STATUS_NOERROR as i32
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn PassThruStartPeriodicMsg(
    ChannelID: u32,
    pMsg: *const PASSTHRU_MSG,
    pMsgID: *const u32,
    TimeInterval: u32,
) -> i32 {
    PassthruError::STATUS_NOERROR as i32
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn PassThruStopPeriodicMsg(ChannelID: u32, MsgID: u32) -> i32 {
    PassthruError::STATUS_NOERROR as i32
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn PassThruIoctl(
    HandleID: u32,
    IoctlID: u32,
    pInput: *mut libc::c_void,
    pOutput: *mut libc::c_void,
) -> i32 {
    PassthruError::STATUS_NOERROR as i32
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn PassThruSetProgrammingVoltage(
    DeviceID: u32,
    PinNumber: u32,
    Voltage: u32,
) -> i32 {
    PassthruError::STATUS_NOERROR as i32
}

#[test]
fn test_read_version() {
    let mut api_version: [u8; 80] = [0; 80];
    let mut dll_version: [u8; 80] = [0; 80];
    let mut fw_version: [u8; 80] = [0; 80];
    let res = PassThruReadVersion(
        0,
        fw_version.as_mut_ptr() as *mut libc::c_char,
        dll_version.as_mut_ptr() as *mut libc::c_char,
        api_version.as_mut_ptr() as *mut libc::c_char,
    );

    assert!(res == PassthruError::STATUS_NOERROR as i32);

    unsafe {
        let api = String::from(
            std::ffi::CStr::from_ptr(api_version.as_mut_ptr() as *mut libc::c_char)
                .to_str()
                .unwrap(),
        );
        assert!(api.as_str() == API_VERSION);

        let dll = String::from(
            std::ffi::CStr::from_ptr(dll_version.as_mut_ptr() as *mut libc::c_char)
                .to_str()
                .unwrap(),
        );
        assert!(dll.as_str() == DLL_VERSION);

        let fw = String::from(
            std::ffi::CStr::from_ptr(fw_version.as_mut_ptr() as *mut libc::c_char)
                .to_str()
                .unwrap(),
        );
        assert!(fw.as_str() == FW_VERSION);
    }
}
