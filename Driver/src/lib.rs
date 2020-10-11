use J2534Common::*;
use libc;
mod logger;

#[no_mangle]
pub extern "system" fn PassThruOpen(pName: *mut libc::c_void, pDeviceID: *mut u64) -> u64 {
    PassthruError::STATUS_NOERROR as u64
}

#[no_mangle]
pub extern "system" fn PassThruClose(pDeviceID: u64) -> u64 {
    PassthruError::STATUS_NOERROR as u64
}

#[no_mangle]
pub extern "system" fn PassthruConnect(DeviceID: u64, ProtocolID: u64, Flags: u64, BaudRate: u64, pChannelID: *mut u64) -> u64 {
    PassthruError::STATUS_NOERROR as u64
}

#[no_mangle]
pub extern "system" fn PassThruDisconnect(ChannelID: u64) -> u64 {
    PassthruError::STATUS_NOERROR as u64
}

#[no_mangle]
pub extern "system" fn PassThruReadVersion(DeviceID: u64, pFirmwareVersion: *const char, pDllVersion: *const char, pApiVersion: *const char) -> u64 {
    PassthruError::STATUS_NOERROR as u64
}

#[no_mangle]
pub extern "system" fn PassThruGetLastError(pErrorDescription: *const char) -> u64 {
    PassthruError::STATUS_NOERROR as u64
}

#[no_mangle]
pub extern "system" fn PassThruReadMsgs(ChannelID: u64, pMsg: *mut PASSTHRU_MSG, pNumMsgs: *mut u64, Timeout: u64) -> u64 {
    PassthruError::STATUS_NOERROR as u64
}

#[no_mangle]
pub extern "system" fn PassThruStartMsgFilter(ChannelID: u64, FilterType: u64, pMaskMsg: *const PASSTHRU_MSG, pPatternMsg: *const PASSTHRU_MSG, pFlowControlMsg: *const PASSTHRU_MSG, pMsgID: *mut u64) -> u64 {
    PassthruError::STATUS_NOERROR as u64
}

#[no_mangle]
pub extern "system" fn PassThruStopMsgFilter(ChannelID: u64, MsgID: u64) -> u64 {
    PassthruError::STATUS_NOERROR as u64
}

#[no_mangle]
pub extern "system" fn PassThruWriteMsgs(ChannelID: u64, pMsg: *const PASSTHRU_MSG, pNumMsgs: *const u64, Timeout: u64) -> u64 {
    PassthruError::STATUS_NOERROR as u64
}

#[no_mangle]
pub extern "system" fn PassThruStartPeriodicMsg(ChannelID: u64, pMsg: *const PASSTHRU_MSG, pMsgID: *const u64, TimeInterval: u64) -> u64 {
    PassthruError::STATUS_NOERROR as u64
}

#[no_mangle]
pub extern "system" fn PassThruStopPeriodicMsg(ChannelID: u64, MsgID: u64) -> u64 {
    PassthruError::STATUS_NOERROR as u64
}

#[no_mangle]
pub extern "system" fn PassThruIoctl(HandleID: u64, IoctlID: u64, pInput: *mut libc::c_void, pOutput: *mut libc::c_void) -> u64 {
    PassthruError::STATUS_NOERROR as u64
}

#[no_mangle]
pub extern "system" fn PassThruSetProgrammingVoltage(DeviceID: u64, PinNumber: u64, Voltage: u64) -> u64 {
    PassthruError::STATUS_NOERROR as u64
}