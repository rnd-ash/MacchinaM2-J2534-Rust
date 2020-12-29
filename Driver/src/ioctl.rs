
use J2534Common::{IoctlParam, Parsable, PassthruError, SConfigList};
use crate::{comm::*, logger::log_warn};
use crate::logger::{log_debug, log_error, log_info};
use byteorder::{ByteOrder, LittleEndian};

pub fn get_battery(output_ptr: *mut u32) -> PassthruError {
    log_info(&format!("Getting voltage"));
    let msg = COMM_MSG::new(MsgType::ReadBatt);
    run_on_m2(|dev| {
        match dev.write_and_read_ptcmd(msg, 250) {
            M2Resp::Ok(args) => {
                let v = LittleEndian::read_u32(&args);
                unsafe { *output_ptr = v };
                Ok(PassthruError::STATUS_NOERROR)
            },
            M2Resp::Err{status, string} => {
                log_error(format!("Error reading battery voltage (Status {:?}): {}", status, string).as_str());
                Ok(status)
            }
        }
    }).unwrap()
}

pub fn set_config(channel_id: u32, cfg_ptr: &SConfigList) -> PassthruError {
    for i in 0..cfg_ptr.num_of_params as isize {
        match unsafe { cfg_ptr.config_ptr.offset(i).as_ref() } {
            None => return PassthruError::ERR_NULL_PARAMETER,
            Some(param) => {
                if param.parameter >= 0x20 {
                    log_warn(format!("setconfig param name is reserved / tool specific?. Param: {:08X}, value: {:08X}", param.parameter, param.value).as_str());
                } else {
                    if let Some(pname) = IoctlParam::from_raw(param.parameter) {
                        log_error(format!("FIXME: Unhandled set_config operation. Channel: {}, Param name: {} value: {}", channel_id, pname, param.value).as_str());
                    } else {
                        log_error(format!("Cannot run setconfig. Invalid IOCTL Param name: {:08X}", param.parameter).as_str());
                        return PassthruError::ERR_INVALID_IOCTL_VALUE
                    }
                }
            }
        }
    }
    PassthruError::STATUS_NOERROR
}

pub fn get_config(channel_id: u32, cfg_ptr: &SConfigList) -> PassthruError {
    PassthruError::STATUS_NOERROR
}