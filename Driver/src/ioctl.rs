
use J2534Common::{IoctlParam, PASSTHRU_MSG, Parsable, PassthruError, SBYTE_ARRAY, SConfigList};
use crate::{channels, comm::*, logger::{log_info_str, log_warn, log_warn_str}};
use crate::logger::{log_debug, log_error, log_info};
use byteorder::{ByteOrder, LittleEndian};

pub fn read_vbatt(output_ptr: *mut u32) -> PassthruError {
    log_info_str("Getting voltage");
    let msg = CommMsg::new(MsgType::ReadBatt);
    run_on_m2(|dev| {
        match dev.write_and_read_ptcmd(msg, 250) {
            M2Resp::Ok(args) => {
                let v = LittleEndian::read_u32(&args);
                unsafe { *output_ptr = v };
                Ok(PassthruError::STATUS_NOERROR)
            },
            M2Resp::Err{status, string} => {
                log_error(format!("Error reading battery voltage (Status {:?}): {}", status, string));
                Ok(status)
            }
        }
    }).unwrap()
}

pub fn read_prog_voltage(output_ptr: *mut u32) -> PassthruError {
    log_warn_str("Read programming voltage unimplemented");
    PassthruError::STATUS_NOERROR
}

pub fn set_config(channel_id: u32, cfg_ptr: &SConfigList) -> PassthruError {
    for i in 0..cfg_ptr.num_of_params as isize {
        match unsafe { cfg_ptr.config_ptr.offset(i).as_ref() } {
            None => return PassthruError::ERR_NULL_PARAMETER,
            Some(param) => {
                if param.parameter >= 0x20 {
                    log_warn(format!("setconfig param name is reserved / tool specific?. Param: {:08X}, value: {:08X}", param.parameter, param.value));
                } else {
                    if let Some(pname) = IoctlParam::from_raw(param.parameter) {
                        if let Err(e) = channels::ChannelComm::ioctl_set_cfg(channel_id, pname, param.value) {
                            return e
                        }
                    } else {
                        log_error(format!("Cannot run setconfig. Invalid IOCTL Param name: {:08X}", param.parameter));
                        return PassthruError::ERR_INVALID_IOCTL_VALUE
                    }
                }
            }
        }
    }
    PassthruError::STATUS_NOERROR
}

pub fn get_config(channel_id: u32, cfg_ptr: &SConfigList) -> PassthruError {
    for i in 0..cfg_ptr.num_of_params as isize {
        match unsafe { cfg_ptr.config_ptr.offset(i).as_mut() } {
            None => return PassthruError::ERR_NULL_PARAMETER,
            Some(mut param) => {
                if param.parameter >= 0x20 {
                    log_warn(format!("getconfig param name is reserved / tool specific?. Param: {:08X}, value: {:08X}", param.parameter, param.value));
                } else {
                    if let Some(pname) = IoctlParam::from_raw(param.parameter) {
                        if let Ok(pvalue) = channels::ChannelComm::ioctl_get_cfg(channel_id, pname) {
                            param.value = pvalue;
                        } else {
                            return PassthruError::ERR_FAILED
                        }
                    } else {
                        log_error(format!("Cannot run getconfig. Invalid IOCTL Param name: {:08X}", param.parameter));
                        return PassthruError::ERR_INVALID_IOCTL_VALUE
                    }
                }
            }
        }
    }
    PassthruError::STATUS_NOERROR
}

pub fn five_baud_init(channel_id: u32, input: &mut SBYTE_ARRAY, output: &mut SBYTE_ARRAY) -> PassthruError {
    log_warn_str("Five baud init unimplemented");
    PassthruError::STATUS_NOERROR
}

pub fn fast_init(channel_id: u32, input: &mut PASSTHRU_MSG, output: &mut PASSTHRU_MSG) -> PassthruError {
    log_warn_str("Five baud init unimplemented");
    PassthruError::STATUS_NOERROR
}

pub fn clear_tx_buffer(channel_id: u32) -> PassthruError {
    channels::ChannelComm::clear_tx_buffer(channel_id)
}

pub fn clear_rx_buffer(channel_id: u32) -> PassthruError {
    channels::ChannelComm::clear_rx_buffer(channel_id)
}

pub fn clear_periodic_msgs(channel_id: u32) -> PassthruError {
    log_warn_str("Clear periodic messages unimplemented");
    PassthruError::STATUS_NOERROR
}

pub fn clear_msg_filters(channel_id: u32) -> PassthruError {
    log_warn_str("Clear message filters unimplemented");
    PassthruError::STATUS_NOERROR
}

pub fn clear_funct_msg_lookup_table(channel_id: u32) -> PassthruError {
    log_warn_str("Clear message lookup table unimplemented");
    PassthruError::STATUS_NOERROR
}

pub fn add_to_funct_msg_lookup_table(channel_id: u32, input: &mut SBYTE_ARRAY) -> PassthruError {
    log_warn_str("Add to function message lookup table unimplemented");
    PassthruError::STATUS_NOERROR
}

pub fn delete_from_funct_msg_lookup_table(channel_id: u32, input: &mut SBYTE_ARRAY) -> PassthruError {
    log_warn_str("Delete ffrom function message lookup table unimplemented");
    PassthruError::STATUS_NOERROR
}