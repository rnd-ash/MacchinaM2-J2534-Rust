use J2534Common::*;
use lazy_static::*;
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
}


/// J2534 API Channel
#[derive(Debug, Copy, Clone)]
struct Channel {
    id: u32,
    protocol: Protocol,
    baud_rate: u32,
    flags: u32   
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
                    Ok(Self{id, protocol, baud_rate, flags})
                },
                M2Resp::Err{status, string} => {
                    log_error(format!("M2 failed to open channel {} (Status {:?}): {}", id, status, string).as_str());
                    set_error_string(string);
                    Err(status)
                }
            }
        })
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