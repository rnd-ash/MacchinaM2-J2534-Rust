use J2534Common::*;
use lazy_static::*;
use libc::free;
use crate::logger::*;
use std::sync::*;
use crate::comm::*;
use byteorder::{LittleEndian, ByteOrder, WriteBytesExt};
use crate::passthru_drv::set_error_string;

const MAX_CHANNELS: usize = 4;

// Same physical 
const USE_CAN_CHAN_ID: usize = 0;
// Same physical
const USE_KLINE_CHAN_ID: usize = 1;
// Same physical
const USE_J1850_CHAN_ID: usize = 2;
// Same physical
const USE_SCI_CHAN_ID: usize = 3;

// Defined in J2534 spec. Each channel can have up to 10 filters
const MAX_FILTERS_PER_CHANNEL: usize = 10;

lazy_static! {
    static ref GLOBAL_CHANNELS: RwLock<[Option<Channel>; MAX_CHANNELS]> = RwLock::new([None; MAX_CHANNELS]);
}

type Result<T> = std::result::Result<T, PassthruError>;

pub struct ChannelComm{}

impl ChannelComm {
    /// Attempts to create a new communication channel
    /// # Returns
    /// Channel ID if operation was OK
    pub fn create_channel(protocol: Protocol, baud_rate: u32, flags: u32) -> Result<u32> {
        match GLOBAL_CHANNELS.write() {
            Ok(mut channels) => {
                let channel_id = match protocol {
                    Protocol::ISO15765 | Protocol::CAN => USE_CAN_CHAN_ID,
                    Protocol::ISO14230 | Protocol::ISO9141 => USE_KLINE_CHAN_ID,
                    Protocol::J1850PWM | Protocol::J1850VPW => USE_J1850_CHAN_ID,
                    Protocol::SCI_A_ENGINE | Protocol::SCI_A_TRANS | Protocol::SCI_B_ENGINE | Protocol::SCI_B_TRANS => USE_SCI_CHAN_ID
                };
                if channels[channel_id].is_none() {
                    Channel::new(channel_id as u32, protocol, baud_rate, flags) // If ID, create a new channel
                    .and_then(|chan| {
                        // If channel creation OK, set it in the channel list
                        let idx = chan.id;
                        channels[idx as usize] = Some(chan);
                        Ok(idx as u32) // Return the ID
                    })
                } else {
                    Err(PassthruError::ERR_CHANNEL_IN_USE)
                }
            },
            Err(e) => {
                set_error_string(format!("Write guard failed: {}", e));
                Err(PassthruError::ERR_FAILED)
            }
        }
    }

    pub fn destroy_channel(id: i32) -> Result<()> {
        match GLOBAL_CHANNELS.write() {
            Ok(mut channels) => {
                let res = match channels[id as usize] {
                    Some(mut channel) => channel.destroy(),
                    None => Err(PassthruError::ERR_INVALID_CHANNEL_ID)
                };
                if res.is_ok() {
                    channels[id as usize] = None; // Drop the old channel if we succeeded in destroying it
                }
                res
            },
            Err(e) => {
                set_error_string(format!("Write guard failed: {}", e));
                Err(PassthruError::ERR_FAILED)
            }
        }
    }

    pub fn create_channel_filter(channel_id: u32, filter_type: FilterType, mask_bytes: &[u8], pattern_bytes: &[u8], fc_bytes: &[u8]) -> Result<u32> {
        match GLOBAL_CHANNELS.write() {
            Ok(channels) => {
                match channels[channel_id as usize] {
                    Some(mut channel) => channel.add_filter(filter_type, mask_bytes, pattern_bytes, fc_bytes),
                    None => Err(PassthruError::ERR_INVALID_CHANNEL_ID)
                }
            },
            Err(e) => {
                set_error_string(format!("Write guard failed: {}", e));
                Err(PassthruError::ERR_FAILED)
            }
        } 
    }
}


/// J2534 API Channel
#[derive(Debug, Copy, Clone)]
struct Channel {
    id: u32,
    protocol: Protocol,
    baud_rate: u32,
    flags: u32,
    filters: [bool; MAX_FILTERS_PER_CHANNEL],
}

impl Channel {
    pub fn new(id: u32, protocol: Protocol, baud_rate: u32, flags: u32) -> Result<Self> {
        // First arg id (u32)
        // Second arg protocol (RAW)
        // Third arg baud rate
        // fourth arg flags
        let mut dst: Vec<u8> = Vec::new();
        for arg in [id, protocol as u32, baud_rate, flags].iter() {
            dst.write_u32::<LittleEndian>(*arg).unwrap();
        }
        log_debug(format!("Requesting channel open. ID: {}, Protocol: {:?}, baud: {}, flags: 0x{:04X}", id, protocol, baud_rate, flags).as_str());
        let msg = COMM_MSG::new_with_args(MsgType::OpenChannel, dst.as_mut_slice());
        run_on_m2(|dev |{
            match dev.write_and_read_ptcmd(msg, 100) {
                M2Resp::Ok(_) => {
                    log_debug("M2 opened channel!");
                    Ok(Self{id, protocol, baud_rate, flags, filters: [false; MAX_FILTERS_PER_CHANNEL]})
                },
                M2Resp::Err{status, string} => {
                    log_error(format!("M2 failed to open channel {} (Status {:?}): {}", id, status, string).as_str());
                    set_error_string(string);
                    Err(status)
                }
            }
        })
    }

    pub fn add_filter(&mut self, filter_type: FilterType, mask_bytes: &[u8], pattern_bytes: &[u8], fc_bytes: &[u8]) -> Result<u32> {
        let mut free_id = 99;
        for i in 0..MAX_FILTERS_PER_CHANNEL {
            if self.filters[i] == false {
                free_id = i as u32;
                break;
            }
        }

        if free_id == 99 {
            return Err(PassthruError::ERR_EXCEEDED_LIMIT)
        }

        // Mask and pattern MUST be present, Flow control is only if FilterType is ISO15765
        // Create our args
        // First arg: channel id (u32)
        // Second arg: specified filter ID (u32)
        // Third arg: Filter type (u32)
        // fourth arg: mask size (u32)
        // fifth arg: pattern size (u32)
        // sixth arg: flow control size (Can be 0) (u32)
        let mut dst: Vec<u8> = Vec::new();
        for arg in [self.id, free_id, filter_type as u32, mask_bytes.len() as u32, pattern_bytes.len() as u32, fc_bytes.len() as u32].iter() {
            dst.write_u32::<LittleEndian>(*arg).unwrap();
        }
        let msg = COMM_MSG::new_with_args(MsgType::SetChannelFilter, dst.as_mut_slice());
        run_on_m2(|dev |{
            match dev.write_and_read_ptcmd(msg, 100) {
                M2Resp::Ok(_) => {
                    log_debug(format!("M2 set filter {} on  channel {}!", free_id, self.id).as_str());
                    self.filters[free_id as usize] = true; // Mark it as used
                    Ok(free_id)
                },
                M2Resp::Err{status, string} => {
                    log_error(format!("M2 failed to set filter {} on channel {} (Status {:?}): {}", free_id, self.id, status, string).as_str());
                    set_error_string(string);
                    Err(status)
                }
            }
        })
    }

    pub fn remove_filter(&mut self, id: usize) -> Result<()> {
        if self.filters[id] == false {
            return Err(PassthruError::ERR_INVALID_MSG_ID)
        }
        self.filters[id] = false;
        Ok(())
    }

    pub fn destroy(&mut self) -> Result<()> {
        log_debug(format!("Requesting channel destroy. ID: {}", self.id).as_str());
        let mut dst: Vec<u8> = Vec::new();
        dst.write_u32::<LittleEndian>(self.id).unwrap();
        let msg = COMM_MSG::new_with_args(MsgType::CloseChannel, dst.as_mut_slice());
        run_on_m2(|dev |{
            match dev.write_and_read_ptcmd(msg, 100) {
                M2Resp::Ok(_) => Ok(()),
                M2Resp::Err{status, string} => {
                    log_error(format!("M2 failed to close channel {} (Status {:?}): {}", self.id, status, string).as_str());
                    set_error_string(string);
                    Err(status)
                }
            }
        })
    }
}

#[test]
fn test_channel_args() {
    let c = Channel::new(0x01, Protocol::ISO15765, 500_000, 0);
}