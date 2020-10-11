use J2534Common::*;
use libc::{c_char, c_long};
use std::ffi::CString;
use libc;
mod logger;

const API_VERSION: &str = "04.04";
const DLL_VERSION: &str = "0.1";
const FW_VERSION: &str = "0.1";

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn PassThruOpen(pName: *mut libc::c_void, pDeviceID: *mut i32) -> i32 {
    PassthruError::STATUS_NOERROR as i32
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn PassThruClose(pDeviceID: i32) -> i32 {
    PassthruError::STATUS_NOERROR as i32
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn PassThruConnect(DeviceID: i32, ProtocolID: i32, Flags: i32, BaudRate: i32, pChannelID: *mut i32) -> i32 {
    PassthruError::STATUS_NOERROR as i32
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn PassThruDisconnect(ChannelID: i32) -> i32 {
    PassthruError::STATUS_NOERROR as i32
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn PassThruReadVersion(DeviceID: i32, pFirmwareVersion: *mut c_char, pDllVersion: *mut c_char, pApiVersion: *mut c_char) -> i32 {
    let fw = CString::new(FW_VERSION).unwrap();
    let api = CString::new(API_VERSION).unwrap();
    let dll = CString::new(DLL_VERSION).unwrap();
    unsafe {
        std::ptr::copy_nonoverlapping( fw.as_c_str().as_ptr(), pFirmwareVersion, FW_VERSION.len());
        std::ptr::copy_nonoverlapping( api.as_c_str().as_ptr(), pApiVersion, API_VERSION.len());
        std::ptr::copy_nonoverlapping( dll.as_c_str().as_ptr(), pDllVersion, DLL_VERSION.len());
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
pub extern "system" fn PassThruReadMsgs(ChannelID: i32, pMsg: *mut PASSTHRU_MSG, pNumMsgs: *mut i32, Timeout: i32) -> i32 {
    PassthruError::STATUS_NOERROR as i32
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn PassThruStartMsgFilter(ChannelID: i32, FilterType: i32, pMaskMsg: *const PASSTHRU_MSG, pPatternMsg: *const PASSTHRU_MSG, pFlowControlMsg: *const PASSTHRU_MSG, pMsgID: *mut i32) -> i32 {
    PassthruError::STATUS_NOERROR as i32
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn PassThruStopMsgFilter(ChannelID: i32, MsgID: i32) -> i32 {
    PassthruError::STATUS_NOERROR as i32
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn PassThruWriteMsgs(ChannelID: i32, pMsg: *const PASSTHRU_MSG, pNumMsgs: *const i32, Timeout: i32) -> i32 {
    PassthruError::STATUS_NOERROR as i32
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn PassThruStartPeriodicMsg(ChannelID: i32, pMsg: *const PASSTHRU_MSG, pMsgID: *const i32, TimeInterval: i32) -> i32 {
    PassthruError::STATUS_NOERROR as i32
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn PassThruStopPeriodicMsg(ChannelID: i32, MsgID: i32) -> i32 {
    PassthruError::STATUS_NOERROR as i32
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn PassThruIoctl(HandleID: i32, IoctlID: i32, pInput: *mut libc::c_void, pOutput: *mut libc::c_void) -> i32 {
    PassthruError::STATUS_NOERROR as i32
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn PassThruSetProgrammingVoltage(DeviceID: i32, PinNumber: i32, Voltage: i32) -> i32 {
    PassthruError::STATUS_NOERROR as i32
}


#[test]
fn test_read_version() {
    let mut api_version: [u8; 80] = [0; 80];
    let mut dll_version: [u8; 80] = [0; 80];
    let mut fw_version:  [u8; 80] = [0; 80];
    let res = PassThruReadVersion(
        0, 
        fw_version.as_mut_ptr() as *mut libc::c_char, 
        dll_version.as_mut_ptr() as *mut libc::c_char,
        api_version.as_mut_ptr() as *mut libc::c_char
    );

    assert!(res == PassthruError::STATUS_NOERROR as i32);

    unsafe {
        let api = String::from(std::ffi::CStr::from_ptr(api_version.as_mut_ptr() as *mut libc::c_char).to_str().unwrap());
        assert!(api.as_str() == API_VERSION);

        let dll = String::from(std::ffi::CStr::from_ptr(dll_version.as_mut_ptr() as *mut libc::c_char).to_str().unwrap());
        assert!(dll.as_str() == DLL_VERSION);

        let fw = String::from(std::ffi::CStr::from_ptr(fw_version.as_mut_ptr() as *mut libc::c_char).to_str().unwrap());
        assert!(fw.as_str() == FW_VERSION);
    }


}