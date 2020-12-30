use J2534Common::*;
use lazy_static::*;
use crate::logger::*;
use std::collections::VecDeque;
use std::sync::*;
use crate::comm::*;
use byteorder::{LittleEndian, ByteOrder, WriteBytesExt};
use crate::passthru_drv::set_error_string;

lazy_static! {
    static ref CAN_CHANNEL: RwLock<Option<Channel>> = RwLock::new(None);
    static ref KLINE_CHANNEL: RwLock<Option<Channel>> = RwLock::new(None);
    static ref J1850_CHANNEL: RwLock<Option<Channel>> = RwLock::new(None);
    static ref SCI_CHANNEL: RwLock<Option<Channel>> = RwLock::new(None);
}


// Defined in J2534 spec. Each channel can have up to 10 filters
const MAX_FILTERS_PER_CHANNEL: usize = 10;

type Result<T> = std::result::Result<T, PassthruError>;

enum ChannelID {
    Can = 0,
    Kline = 1,
    J1850 = 2,
    Sci = 3
}


impl ChannelID {
    fn get_channel<'a>(&self) -> &'static RwLock<Option<Channel>> {
        match self {
            ChannelID::Can => &CAN_CHANNEL,
            ChannelID::Kline => &KLINE_CHANNEL,
            ChannelID::J1850 => &J1850_CHANNEL,
            ChannelID::Sci => &SCI_CHANNEL
        }
    }

    pub fn from_u32(id: u32) -> Result<Self> {
        match id {
            0 => Ok(ChannelID::Can),
            1 => Ok(ChannelID::Kline),
            2 => Ok(ChannelID::J1850),
            3 => Ok(ChannelID::Sci),
            _ => Err(PassthruError::ERR_INVALID_CHANNEL_ID)
        }
    }

    pub fn from_protocol(id: Protocol) -> Self {
        match id {
            Protocol::ISO15765 | Protocol::CAN => ChannelID::Can,
            Protocol::ISO14230 | Protocol::ISO9141 => ChannelID::Kline,
            Protocol::J1850PWM | Protocol::J1850VPW => ChannelID::J1850,
            Protocol::SCI_A_ENGINE | Protocol::SCI_A_TRANS | Protocol::SCI_B_ENGINE | Protocol::SCI_B_TRANS => ChannelID::Sci
        }
    }
}

pub struct ChannelComm{}

impl ChannelComm {
    /// Attempts to create a new communication channel
    /// # Returns
    /// Channel ID if operation was OK
    pub fn create_channel(protocol: Protocol, baud_rate: u32, flags: u32) -> Result<u32> {
        let protocol_id = ChannelID::from_protocol(protocol);
        match protocol_id.get_channel().write() {
            Ok(mut channel) => {
                if channel.is_some() { // Already occupied!
                    return Err(PassthruError::ERR_CHANNEL_IN_USE)
                }
                Channel::new(protocol_id as u32, protocol, baud_rate, flags) // If ID, create a new channel
                    .and_then(|chan| {
                        // If channel creation OK, set it in the channel list
                        let idx = chan.id;
                        *channel = Some(chan);
                        Ok(idx as u32) // Return the ID
                    })
            }
            Err(e) => {
                set_error_string(format!("Write guard failed: {}", e));
                Err(PassthruError::ERR_FAILED)
            }
        }
    }

    pub fn destroy_channel(channel_id: u32) -> Result<()> {
        match ChannelID::from_u32(channel_id)?.get_channel().write() {
            Ok(mut channel) => {
                if let Some(c) = channel.take() {
                    c.destroy()
                } else {
                    Err(PassthruError::ERR_INVALID_CHANNEL_ID)
                }
            }
            Err(e) => {
                set_error_string(format!("Write guard failed: {}", e));
                Err(PassthruError::ERR_FAILED)
            }
        }
    }
 
    pub fn create_channel_filter(channel_id: u32, filter_type: FilterType, mask_bytes: &[u8], pattern_bytes: &[u8], fc_bytes: &[u8]) -> Result<u32> {
        match ChannelID::from_u32(channel_id)?.get_channel().write() {
            Ok(mut channel) => {
                if let Some(c) = channel.as_mut() {
                    c.add_filter(filter_type, mask_bytes, pattern_bytes, fc_bytes)
                } else {
                    Err(PassthruError::ERR_INVALID_CHANNEL_ID)
                }
            }
            Err(e) => {
                set_error_string(format!("Write guard failed: {}", e));
                Err(PassthruError::ERR_FAILED)
            }
        }
    }

    pub fn write_channel_data(channel_id: u32, msg: &PASSTHRU_MSG, require_response: bool) -> Result<()> {
        match ChannelID::from_u32(channel_id)?.get_channel().read() {
            Ok(channel) => {
                if let Some(c) = channel.as_ref() {
                    c.transmit_data(msg, require_response)
                } else {
                    Err(PassthruError::ERR_INVALID_CHANNEL_ID)
                }
            }
            Err(e) => {
                set_error_string(format!("Write guard failed: {}", e));
                Err(PassthruError::ERR_FAILED)
            }
        }
    }

    pub fn read_channel_data(channel_id: u32) -> Result<Option<PASSTHRU_MSG>> {
        let channel = ChannelID::from_u32(channel_id)?.get_channel();

        if let Some(c) = channel.read().unwrap().as_ref() {
            if c.rx_available() == 0 {
                return Ok(None)
            }
        } else {
            return Err(PassthruError::ERR_INVALID_CHANNEL_ID)
        }

        match channel.write() {
            Ok(mut c) => {
                Ok(c.as_mut().unwrap().pop_rx_queue())
            },
            Err(e) => {
                set_error_string(format!("Write guard failed: {}", e));
                Err(PassthruError::ERR_FAILED)
            }
        }
    }

    /// Used by the receiver thread running on the M2 to write data to our Rx buffer
    pub fn receive_channel_data(msg: &CommMsg) -> Result<()> {
        match ChannelID::from_u32(msg.args[0] as u32)?.get_channel().write() {
            Ok(mut channel) => {
                if let Some(c) = channel.as_mut() {
                    let tx_flags = LittleEndian::read_u32(&msg.args[1..5]);
                    let data = &msg.args[5..msg.arg_size as usize];
                    c.on_receive_data(tx_flags, data)
                }
                Ok(())
            }
            Err(e) => {
                set_error_string(format!("Write guard failed: {}", e));
                Err(PassthruError::ERR_FAILED)
            }
        }
    }
}


const MAX_QUEUE_MSGS: usize = 100;
/// J2534 API Channel
#[derive(Debug, Clone)]
struct Channel {
    id: u32,
    protocol: Protocol,
    baud_rate: u32,
    flags: u32,
    filters: [u8; MAX_FILTERS_PER_CHANNEL],
    tx_data: VecDeque<PASSTHRU_MSG>, // 1000 Tx messages (~4MB)
    rx_data: VecDeque<PASSTHRU_MSG>, // 1000 Rx messages (~4MB)
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
        log_debug(format!("Requesting channel open. ID: {}, Protocol: {:?}, baud: {}, flags: 0x{:04X}", id, protocol, baud_rate, flags));
        let msg = CommMsg::new_with_args(MsgType::OpenChannel, dst.as_mut_slice());
        run_on_m2(|dev |{
            match dev.write_and_read_ptcmd(msg, 100) {
                M2Resp::Ok(_) => {
                    log_debug_str("M2 opened channel!");
                    Ok(Self{
                        id, 
                        protocol, 
                        baud_rate, 
                        flags, 
                        filters: [0x00; MAX_FILTERS_PER_CHANNEL], 
                        tx_data: VecDeque::new(), 
                        rx_data: VecDeque::new(),
                    })
                },
                M2Resp::Err{status, string} => {
                    log_error(format!("M2 failed to open channel {} (Status {:?}): {}", id, status, string));
                    set_error_string(string);
                    Err(status)
                }
            }
        })
    }

    pub fn add_filter(&mut self, filter_type: FilterType, mask_bytes: &[u8], pattern_bytes: &[u8], fc_bytes: &[u8]) -> Result<u32> {
        println!("{:?}", self.filters);
        let free_id = self.filters.iter().enumerate().find(| (_, v) | {**v == 0}).map_or(99, |x| x.0);

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
        for arg in [self.id, free_id as u32, filter_type as u32, mask_bytes.len() as u32, pattern_bytes.len() as u32, fc_bytes.len() as u32].iter() {
            dst.write_u32::<LittleEndian>(*arg).unwrap();
        }
        dst.extend_from_slice(mask_bytes);
        dst.extend_from_slice(pattern_bytes);
        dst.extend_from_slice(fc_bytes);
        log_debug(format!("Setting {} (ID: {}) on channel {}. Mask: {:02X?}, Pattern: {:02X?}, FlowControl: {:02X?}", filter_type, self.id, free_id, mask_bytes, pattern_bytes, fc_bytes));
        let msg = CommMsg::new_with_args(MsgType::SetChannelFilter, dst.as_mut_slice());
        run_on_m2(|dev |{
            match dev.write_and_read_ptcmd(msg, 250) {
                M2Resp::Ok(_) => {
                    log_debug(format!("M2 set filter {} on channel {}!", free_id, self.id));
                    self.filters[free_id] = 1; // Mark it as used
                    Ok(free_id as u32)
                },
                M2Resp::Err{status, string} => {
                    log_error(format!("M2 failed to set filter {} on channel {} (Status {:?}): {}", free_id, self.id, status, string));
                    set_error_string(string);
                    Err(status)
                }
            }
        })
    }

    pub fn remove_filter(&mut self, id: usize) -> Result<()> {
        if self.filters[id] == 0 {
            return Err(PassthruError::ERR_INVALID_MSG_ID)
        }
        let mut dst: Vec<u8> = Vec::new();
        for arg in [self.id, id as u32].iter() {
            dst.write_u32::<LittleEndian>(*arg).unwrap();
        }
        log_debug(format!("Closing channel {} filter {}", self.id, id));
        let msg = CommMsg::new_with_args(MsgType::RemoveChannelFilter, dst.as_mut_slice());
        run_on_m2(|dev |{
            match dev.write_and_read_ptcmd(msg, 100) {
                M2Resp::Ok(_) => {
                    log_debug_str("M2 closed filter OK!");
                    self.filters[id] = 0; // Mark it as used
                    Ok(())
                },
                M2Resp::Err{status, string} => {
                    log_error(format!("M2 failed to set filter {} on channel {} (Status {:?}): {}", id, self.id, status, string));
                    set_error_string(string);
                    Err(status)
                }
            }
        })
    }

    pub fn destroy(&self) -> Result<()> {
        log_debug(format!("Requesting channel destroy. ID: {}", self.id));
        let mut dst: Vec<u8> = Vec::new();
        dst.write_u32::<LittleEndian>(self.id).unwrap();
        let msg = CommMsg::new_with_args(MsgType::CloseChannel, dst.as_mut_slice());
        run_on_m2(|dev |{
            match dev.write_and_read_ptcmd(msg, 250) {
                M2Resp::Ok(_) => Ok(()),
                M2Resp::Err{status, string} => {
                    log_error(format!("M2 failed to close channel {} (Status {:?}): {}", self.id, status, string));
                    set_error_string(string);
                    Err(status)
                }
            }
        })
    }

    pub fn transmit_data(&self, ptmsg: &PASSTHRU_MSG, require_response: bool) -> Result<()> {
        if ptmsg.protocol_id != self.protocol as u32 {
            return Err(PassthruError::ERR_MSG_PROTOCOL_ID);
        }

        // Build Tx message
        let mut dst: Vec<u8> = Vec::new();
        for arg in [self.id, ptmsg.tx_flags].iter() {
            dst.write_u32::<LittleEndian>(*arg).unwrap();
        }
        dst.extend_from_slice(&ptmsg.data[0..ptmsg.data_size as usize]);
        let msg = CommMsg::new_with_args(MsgType::TransmitChannelData, dst.as_mut_slice());
        log_debug(format!("Channel {} writing message: {}. Response required?: {}", self.id, ptmsg, require_response));
        run_on_m2(|dev| {
            if require_response {
                match dev.write_and_read_ptcmd(msg, 100) {
                    M2Resp::Ok(_) => Ok(()),
                    M2Resp::Err{status, string}  => {
                        log_error(format!("M2 failed to write data to channel {} (Status {:?}): {}", self.id, status, string));
                        set_error_string(string);
                        Err(status)
                    }
                }
            } else {
                dev.write_comm_struct(msg)
            }
        })
    }

    pub fn pop_rx_queue(&mut self) -> Option<PASSTHRU_MSG> {
        self.rx_data.pop_front()
    }

    pub fn rx_available(&self) -> usize {
        self.rx_data.len()
    }

    pub fn on_receive_data(&mut self, rx_status: u32, data: &[u8]) {
        if self.rx_data.len() < MAX_QUEUE_MSGS {
            let mut msg = PASSTHRU_MSG::default();
            msg.data_size = data.len() as u32;
            msg.rx_status = rx_status;
            msg.protocol_id = self.protocol as u32;
            msg.data[..data.len()].copy_from_slice(data);
            msg.timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_micros() as u32;
            log_debug(format!("Channel {} buffering message. RxStatus: {:08X}, data: {:02X?}", self.id, rx_status, &data));
            self.rx_data.push_back(msg);
        } else {
            // Data is lost if queue is too big!
            log_warn(format!("Rx queue in channel {} is full. Data has been lost!", self.id));
        }
    }

    pub fn ioctl(&mut self, ioctl_id: IoctlID) -> Result<()> {
        match ioctl_id {
            IoctlID::CLEAR_TX_BUFFER => self.tx_data.clear(),
            IoctlID::CLEAR_RX_BUFFER => self.rx_data.clear(),
            _ => {
                log_error(format!("Unhandled raw IOCTL request for channel {} {:?}", self.id, ioctl_id));
                return Err(PassthruError::ERR_FAILED)
            }
        }
        Ok(())
    }

    pub fn ioctl_set_config(cfg: &SConfig) {

    }

    pub fn ioctl_get_config(cfg: &SConfig) {

    }
}