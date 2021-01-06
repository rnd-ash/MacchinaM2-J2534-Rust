use logger::{log_debug, log_error};
use serialport::*;
use std::io::{Write, Read, Error, ErrorKind};
use std::sync::{Arc, Mutex, atomic::AtomicU32, atomic::AtomicBool, atomic::Ordering};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread::{spawn, JoinHandle};
use std::sync::RwLock;
use lazy_static::lazy_static;
use std::collections::hash_map::HashMap;
use crate::{channels, logger};
use J2534Common::{PassthruError, Parsable};
use crate::passthru_drv::set_error_string;
use byteorder::{ByteOrder, WriteBytesExt, LittleEndian};


#[cfg(unix)]
use serde_json;

#[cfg(windows)]
use winreg::{RegKey, RegValue, enums::HKEY_LOCAL_MACHINE};

lazy_static! {
    pub static ref M2: RwLock<Option<MacchinaM2>> = RwLock::new(None);
    static ref MSG_ID: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
}


fn get_id() -> u8 {
    let v = (MSG_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst) & 0xFF) as u8;
    return match v {
        0 => (MSG_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst) & 0xFF) as u8,
        x => x
    }
}

#[derive(Debug, Clone)]
pub enum M2Resp {
    Ok(Vec<u8>),
    Err { status: PassthruError, string: String }
}

pub struct MacchinaM2 {
    rx_queue: Arc<RwLock<HashMap<u8, CommMsg>>>,
    handler: Option<JoinHandle<()>>,
    is_running: Arc<AtomicBool>,
    tx_send_queue: Sender<CommMsg>,
}

unsafe impl Send for MacchinaM2{}
unsafe impl Sync for MacchinaM2{}

type Result<T> = std::io::Result<T>;

#[cfg(unix)]
fn get_comm_port() -> Option<String> {
    if let Ok(content) = std::fs::read_to_string(shellexpand::tilde("~/.passthru/macchina.json").to_string()) {
        return match serde_json::from_str::<serde_json::Value>(content.as_str()) {
            Ok(v) => v["COM-PORT"].as_str().map(String::from),
            Err(_) => None
        }
    }
    None
}

#[cfg(windows)]
fn get_comm_port() -> Option<String> {
    if let Ok(reg) = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey("SOFTWARE\\WOW6432Node\\PassThruSupport.04.04\\Macchina-Passthru") {
        logger::log_info_str("Found regkey");
        return match reg.get_value("COM-PORT") {
            Ok(s) => {
                logger::log_info(format!("Com port is {}", s));
                Some(s)
            },
            Err(_) => None
        }
    }
    None
}

pub type PTResult<T> = std::result::Result<T, PassthruError>;
pub fn run_on_m2<T, F: FnOnce(&MacchinaM2) -> PTResult<T>>(op: F) -> PTResult<T> {
    match M2.read() {
        Ok(d) => {
            match d.as_ref() {
                Some(dev) => op(dev),
                None => Err(PassthruError::ERR_DEVICE_NOT_CONNECTED)
            }
        },
        Err(x) => {
            set_error_string(format!("RWLockGuard on M2 failed to be aquired {}", x));
            Err(PassthruError::ERR_FAILED)
        }
    }
}

impl MacchinaM2 {
    pub fn open_connection() -> Result<Self> {
        match get_comm_port() {
            Some(s) => MacchinaM2::open_conn(s.as_str()),
            None => Err(Error::new(ErrorKind::NotFound, "Cannot find COM-PORT attribute"))
        }
    }


    fn open_conn(port: &str) -> Result<Self> {
        
        let mut port = match serialport::new(port, 500000).open_native() {
            Ok(mut port) => {
                port.set_timeout(std::time::Duration::from_millis(1))?;
                port.set_flow_control(FlowControl::Hardware)?;
                port.set_data_bits(DataBits::Eight)?;
                port.set_parity(Parity::None)?;
                port
            },
            Err(e) => {return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Error opening port {}", e.to_string())));}
        };

        // Create our Rx queue for incomming messages
        let rx_queue = Arc::new(RwLock::new(HashMap::new()));
        let rx_queue_t = rx_queue.clone();

        let (send_tx, send_rx) : (Sender<CommMsg>, Receiver<CommMsg>) = channel();

        // Set tell the thread to run by default
        let is_running = Arc::new(AtomicBool::new(true));
        let is_running_t = is_running.clone();
        // Since UNIX has a 4KB Page size, I want to store more data,
        // Use a 16KB Buffer
        let handler = Some(spawn(move || {
            let mut read_buffer: [u8; COMM_MSG_SIZE * 4] = [0x00; COMM_MSG_SIZE * 4];
            logger::log_debug_str("M2 receiver thread starting!");
            let msg = CommMsg::new_with_args(MsgType::StatusMsg, &[0x01]);
            if port.write_all(&msg.to_slice()).is_err() {
                logger::log_error_str("Timeout writing init struct!");
                is_running_t.store(false, Ordering::Relaxed);
                return;
            }
            let mut read_count = 0;
            while is_running_t.load(Ordering::Relaxed) {
                // Any messages to write?
                if let Ok(m) = send_rx.try_recv() {
                    port.write_all(&m.to_slice());
                }

                let incomming = port.read(&mut read_buffer[read_count..]).unwrap_or(0);
                if incomming > 0 {
                    read_count += incomming;
                    if read_count >= COMM_MSG_SIZE {
                        read_count -= COMM_MSG_SIZE;
                        let msg = CommMsg::from_vec(&read_buffer[0..COMM_MSG_SIZE]);
                        read_buffer.rotate_right(COMM_MSG_SIZE);
                        match msg.msg_type {
                            MsgType::LogMsg => { logger::log_m2_msg(String::from_utf8(Vec::from(msg.args)).unwrap()) },
                            MsgType::ReceiveChannelData => { channels::ChannelComm::receive_channel_data(&msg); },
                            _ => { rx_queue_t.write().unwrap().insert(msg.msg_id, msg); }
                        }
                    }
                } else {
                    std::thread::sleep(std::time::Duration::from_micros(10));
                }
            }
            let msg = CommMsg::new_with_args(MsgType::StatusMsg, &[0x00]);
            port.write_all(&msg.to_slice());
            logger::log_debug_str("M2 receiver thread exiting");
        }));
        std::thread::sleep(std::time::Duration::from_millis(50));
        if is_running.load(Ordering::Relaxed) == false {
            return Err(std::io::Error::new(std::io::ErrorKind::ConnectionAborted, "Error initializing M2"));
        }

        //while is_started.load(Ordering::Relaxed) == false {} // Wait for thread to start
        let m = MacchinaM2 {
            rx_queue,
            handler,
            is_running,
            tx_send_queue: send_tx
        };
        return Ok(m);
    }

    pub fn write_comm_struct(&self, mut s: CommMsg) -> PTResult<()> {
        s.msg_id = 0x00; // Tell M2 it doesn't have to respond to request
        self.tx_send_queue.send(s);
        Ok(())
    }

    pub fn write_and_read_ptcmd(&self, s: CommMsg, timeout_ms: u128) -> M2Resp {
        match self.write_and_read(s, timeout_ms) {
            Err(e) => M2Resp::Err { status: e, string: format!("M2 communication failure: {:?}", e) },
            Ok(resp) => {
                let status = match PassthruError::from_raw(resp.args[0] as u32) {
                    Some(x) => x,
                    None => {
                        return M2Resp::Err{ status: PassthruError::ERR_FAILED, string: format!("Unrecognized status {}", resp.args[0]) }
                    }
                };
                match status {
                    PassthruError::STATUS_NOERROR => {
                        match resp.arg_size {
                            1 => M2Resp::Ok(Vec::new()),
                            _ => M2Resp::Ok(Vec::from(&resp.args[1..resp.arg_size as usize]))
                        }
                    },
                    _ => {
                        let text = if resp.arg_size > 1 {
                            String::from_utf8(Vec::from(&resp.args[1..resp.arg_size as usize])).unwrap()
                        } else {
                            format!("No error given")
                        };
                        M2Resp::Err { status, string: text }
                    }
                }
            }
        }
    }

    /// Writes a message to the M2 unit, and expects a designated response back from the unit
    pub fn write_and_read(&self, mut s: CommMsg, timeout_ms: u128) -> PTResult<CommMsg> {
        let query_id = get_id(); // Set a unique ID, M2 is now forced to respond
        s.msg_id = query_id;
        if let Err(e) = self.tx_send_queue.send(s) {
            log_error(format!("Error writing comm msg to queue {}", e));
            return Err(PassthruError::ERR_FAILED);
        }
        logger::log_debug(format!("Write data: {}", &s));
        let start_time = std::time::Instant::now();
        while start_time.elapsed().as_millis() <= 1000 {
            if let Ok(mut lock) = self.rx_queue.write() {
                if lock.contains_key(&query_id) {
                    log_debug(format!("Command took {}ms to execute", start_time.elapsed().as_millis()));
                    return Ok(lock.remove(&query_id).unwrap());
                }
            }
            std::thread::sleep(std::time::Duration::from_micros(100));
        }
        return Err(PassthruError::ERR_TIMEOUT);
    }

    pub fn stop(&mut self) {
        self.is_running.store(false, Ordering::Relaxed);
        self.handler.take().map(JoinHandle::join);
    }
}

// Terminate the thread on Struct Drop
impl Drop for MacchinaM2 {
    fn drop(&mut self) {
    }
}

const COMM_MSG_SIZE: usize = 4096;
const COMM_MSG_ARG_SIZE: usize = COMM_MSG_SIZE - 4;

#[derive(Debug, Copy, Clone)]
pub enum MsgType {
    Unknown = 0x00,
    LogMsg = 0x01,
    OpenChannel = 0x02,
    CloseChannel = 0x03,
    SetChannelFilter = 0x04,
    RemoveChannelFilter = 0x05,
    TransmitChannelData = 0x06,
    ReceiveChannelData = 0x07,
    ReadBatt = 0x08,
    IoctlSet = 0x09,
    IoctlGet = 0x10,
    StatusMsg = 0xAA,
    GetFwVersion = 0xAB,
    #[cfg(test)]
    TestMessage = 0xFF
}

impl std::cmp::PartialEq for MsgType {
    fn eq(&self, other: &MsgType) -> bool {
        *self as u8 == *other as u8
    }
}

impl MsgType {
    fn from_u8(s: u8) -> MsgType {
        match s {
            0x01 => MsgType::LogMsg,
            0x02 => MsgType::OpenChannel,
            0x03 => MsgType::CloseChannel,
            0x04 => MsgType::SetChannelFilter,
            0x05 => MsgType::RemoveChannelFilter,
            0x06 => MsgType::TransmitChannelData,
            0x07 => MsgType::ReceiveChannelData,
            0x08 => MsgType::ReadBatt,
            0x09 => MsgType::IoctlSet,
            0x10 => MsgType::IoctlGet,
            0xAA => MsgType::StatusMsg,
            0xAB => MsgType::GetFwVersion,
            #[cfg(test)]
            0xFF => MsgType::TestMessage,
            _ => {
                logger::log_warn(format!("Unknown message type {:02X}", s));
                MsgType::Unknown
            }
        }
    }
}
#[derive(Debug, Copy, Clone)]
/// Comm message that is sent and received fro the M2 module
pub struct CommMsg {
    pub msg_id: u8,
    pub msg_type: MsgType,                  // Message type
    pub arg_size: u16,                 // Arg size
    pub args: [u8; COMM_MSG_ARG_SIZE], // Args
}

impl std::fmt::Display for CommMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "COMM_MSG: ID: {:02X} Type: {:?}, Size: {} Args=[", self.msg_id, self.msg_type, self.arg_size)?;
        (0..std::cmp::min(self.arg_size, COMM_MSG_ARG_SIZE as u16)).for_each(|b| {
            write!(f, "{:02X}", self.args[b as usize]).unwrap();
            if b < self.arg_size - 1 {
                write!(f, " ").unwrap();
            }
        });
        write!(f, "]")
    }
}

impl PartialEq<CommMsg> for CommMsg {
    fn eq(&self, other: &CommMsg) -> bool {
        self.msg_type as u8 == other.msg_type as u8 && self.arg_size == other.arg_size
    }
}

impl CommMsg {
    pub fn from_vec(buf: &[u8]) -> Self {
        let mut args: [u8; COMM_MSG_ARG_SIZE] = [0x00; COMM_MSG_ARG_SIZE];
        (0..COMM_MSG_ARG_SIZE).for_each(|i| {
            args[i] = buf[i + 4];
        });
        CommMsg {
            msg_id: buf[0],
            msg_type: MsgType::from_u8(buf[1]),
            arg_size: LittleEndian::read_u16(&buf[2..4]),
            args,
        }
    }

    pub fn new(msg_type: MsgType) -> Self {
        CommMsg {
            msg_type,
            arg_size: 0,
            args: [0x00; COMM_MSG_ARG_SIZE],
            msg_id: 0,
        }
    }

    pub fn new_with_args(msg_type: MsgType, args_array: &[u8]) -> Self {
        let mut args: [u8; COMM_MSG_ARG_SIZE] = [0x00; COMM_MSG_ARG_SIZE];
        (0..std::cmp::min(args_array.len(), COMM_MSG_ARG_SIZE)).for_each(|i| {
            args[i] = args_array[i];
        });
        CommMsg {
            msg_type,
            arg_size: args_array.len() as u16,
            args,
            msg_id: 0,
        }
    }

    pub fn put_args(&mut self, args: &[u8]) {
        if args.len() > COMM_MSG_ARG_SIZE {
            logger::log_warn(format!("Input args is {} larger than payload size, truncating", args.len() - COMM_MSG_ARG_SIZE));
        }
        (0..std::cmp::min(COMM_MSG_ARG_SIZE, args.len())).for_each(|i| {
            self.args[i] = args[i];
        });
    }

    pub fn set_type(&mut self, msg_type: MsgType) {
        self.msg_type = msg_type;
    }

    pub fn to_slice(&self) -> Vec<u8> {
        let mut params: Vec<u8> = Vec::with_capacity(self.arg_size as usize + 4);
        params.write_u16::<LittleEndian>(self.arg_size+2).unwrap(); // 0,1
        params.push(self.msg_id); // 2
        params.push(self.msg_type as u8); // 3
        params.append(&mut self.args[0..self.arg_size as usize].to_vec());
        return params;
    }
}

pub fn get_batt_voltage() -> PTResult<u32> {
    let msg = CommMsg::new(MsgType::ReadBatt);
    run_on_m2(|dev| {
        match dev.write_and_read_ptcmd(msg, 250) {
            M2Resp::Ok(args) => Ok(byteorder::LittleEndian::read_u32(&args)),
            M2Resp::Err{status, string: _} => Err(status)
        }
    })
}


#[cfg(test)]
mod comm_test {
    use rand::Rng;
    use super::*;

    #[cfg(windows)]
    const TEST_PORT: &str = "COM12";

    #[cfg(unix)]
    const TEST_PORT: &str = "/dev/ttyACM0";

    #[test]
    #[ignore]
    fn test_io_m2() {
        let mut tx_errors = 0;
        let mut rec_fail = 0;
        let mut macchina = MacchinaM2::open_conn(TEST_PORT)
            .expect(format!("Could not open COM port {}!", TEST_PORT).as_str());

        // For this test, once we open the comm port,
        // Fire 100 random COMM messages at the M2 module
        // (Use type 0xFF to tell Macchina to echo back the same message)
        // Assert that each received message is the same as what was sent
        for _ in 0..10 {
            let args: Vec<u8> = (0..100)
                .map(|_| rand::thread_rng().gen_range(0, 0xFF))
                .collect();
            let msg = CommMsg::new_with_args(MsgType::TestMessage, &args);
            match macchina.write_and_read(msg, 250) {
                Err(_) => rec_fail += 1, // Macchina did not respond to our message
                Ok(x) => {
                    // Macchina responded!
                    if x != msg {
                        // Check if received payload == sent payload
                        tx_errors += 1; // Some corruption occured!
                    }
                }
            }
        }
        macchina.stop();
        assert!(tx_errors == 0);
        assert!(rec_fail == 0);
    }
}

#[test]
fn test_comm_msg() {
    let mut msg = CommMsg::new_with_args(MsgType::GetFwVersion, &[0x01 as u8, 0x02 as u8, 0x03 as u8]);
    msg.msg_id = 0x01;
    let data = msg.to_slice();
    println!("{:02X?}", data);
    let parsed = CommMsg::from_vec(data.as_slice());
    println!("{}", msg);
    println!("{}", parsed);
}
