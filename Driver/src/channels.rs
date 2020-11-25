use J2534Common::*;
use lazy_static::*;
use crate::logger::*;
use std::sync::*;
use crate::comm::*;
use byteorder::{LittleEndian, ByteOrder, WriteBytesExt};
use crate::passthru_drv::set_error_string;

const MAX_CHANNELS: usize = 10;

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
                Self::get_free_id(&channels.as_ref()).and_then(|id|
                    Channel::new(id as u32, protocol, baud_rate, flags) // If ID, create a new channel
                ).and_then(|chan| {
                    // If channel creation OK, set it in the channel list
                    let idx = chan.id;
                    channels[idx as usize] = Some(chan);
                    Ok(idx as u32) // Return the ID
                })
            },
            Err(e) => {
                set_error_string(format!("Write guard failed: {}", e));
                Err(PassthruError::ERR_FAILED)
            }
        }
    }

    fn get_free_id(curr_list: &[Option<Channel>]) -> Result<usize> {
        for i in 0..MAX_CHANNELS {
            if curr_list[i].is_none() {
                return Ok(i);
            }
        }
        set_error_string(format!("Exceeded {} channels!", MAX_CHANNELS));
        Err(PassthruError::ERR_FAILED)
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
        Ok(())
    }
}

#[test]
fn test_channel_args() {
    let c = Channel::new(0x01, Protocol::ISO15765, 500_000, 0);
}