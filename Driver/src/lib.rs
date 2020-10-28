use libc::{c_char, c_long};
use J2534Common::*;
mod logger;
mod comm;
use comm::{MacchinaM2};
mod passthru_drv;
use  passthru_drv::*;

#[cfg(test)]
mod lib_tests;

// Dll Load function (Windows only) - Just return true
#[no_mangle]
#[cfg(windows)]
#[allow(non_snake_case)]
pub extern "system" fn DllMain(_module: u32, _reason: u32, _reserved: *mut std::ffi::c_void) -> bool {
    return true
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn PassThruOpen(_name: *mut libc::c_void, device_id: *mut u32) -> i32 {
    passthru_open(device_id) as i32
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn PassThruClose(pDeviceID: u32) -> i32 {
    passthru_close(pDeviceID) as i32
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
    logger::info(format!(
        "PASSTHRU_CONNECT. Protocol: {:?}, Baudrate: {}",
        prot, BaudRate
    ));
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
    _device_id: u32,
    fw_version_ptr: *mut c_char,
    dll_version_ptr: *mut c_char,
    api_version_ptr: *mut c_char,
) -> i32 {
    passthru_read_version(fw_version_ptr, dll_version_ptr, api_version_ptr) as i32
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
            None => return PassthruError::ERR_INVALID_PROTOCOL_ID as i32,
        };
        let size = ptr.data_size;
        let data = &ptr.data[0..size as usize];
        logger::info(format!(
            "WRITE_MSGS. Protocol: {:?}, Data size: {} {:x?}. Timeout {} ms",
            prot, size, data, Timeout
        ));
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