use serialport::*;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread::{spawn, JoinHandle};

struct MacchinaM2 {
    p: Box<dyn SerialPort>,
    rx_queue: Arc<Mutex<VecDeque<COMM_MSG>>>,
    handler: JoinHandle<()>,
    is_running: Arc<Mutex<bool>>,
}

impl MacchinaM2 {
    fn open_connection(port: &str) -> Result<Self> {
        // Connection settings for Macchina M2 (Basically arduino SAM Board)
        let settings = SerialPortSettings {
            baud_rate: 500_000,
            flow_control: FlowControl::None,
            data_bits: DataBits::Eight,
            parity: Parity::None,
            stop_bits: StopBits::One,
            timeout: std::time::Duration::from_millis(250),
        };

        // Create a serialport connection
        let orig_port = open_with_settings(port, &settings)?;
        // Clear Tx and Rx buffers if any stray data exists
        orig_port.clear(ClearBuffer::All)?;

        // Create our Rx queue for incomming messages
        let rx_queue = Arc::new(Mutex::new(VecDeque::new()));

        // Clone the port so it can be used in the listener thread
        let mut port_t = orig_port.try_clone()?;
        // Clone the queue so it can be used in the listener thread
        let queue_t = rx_queue.clone();

        // Set tell the thread to run by default
        let is_running = Arc::new(Mutex::new(true));

        let is_running_t = is_running.clone();

        // Since UNIX has a 4KB Page size, I want to store more data,
        // Use a 8KB Buffer
        let handler = spawn(move || {
            loop {
                if port_t.bytes_to_read().unwrap() >= COMM_MSG_SIZE as u32 {
                    let mut buf: Vec<u8> = vec![0; COMM_MSG_SIZE];
                    port_t.read_exact(buf.as_mut_slice()).unwrap();
                    // Aquire lock before writing to the vec queue
                    let msg = COMM_MSG::from_vec(&buf);
                    if msg.msg_type != 0x01 {
                        queue_t.lock().unwrap().push_back(msg);
                    } else {
                        println!(
                            "M2 LOG -> {}",
                            String::from_utf8(Vec::from(msg.args)).unwrap()
                        );
                    }
                }

                if *is_running_t.lock().unwrap() == false {
                    break;
                }
            }
        });

        Ok(MacchinaM2 {
            p: orig_port,
            rx_queue,
            handler,
            is_running,
        })
    }

    fn write_comm_struct(&mut self, s: COMM_MSG) -> std::io::Result<()> {
        let buffer = s.to_slice();
        self.p.write_all(&buffer)
    }

    fn read_comm_struct(&mut self) -> Option<COMM_MSG> {
        match self.rx_queue.lock() {
            Ok(mut r) => r.pop_front(),
            Err(_) => None,
        }
    }

    fn write_and_read(&mut self, s: COMM_MSG, timeout_ms: u128) -> Option<COMM_MSG> {
        if self.write_comm_struct(s).is_err() {
            return None;
        }
        let start = std::time::Instant::now();
        while start.elapsed().as_millis() < timeout_ms {
            if let Some(x) = self.read_comm_struct() {
                return Some(x);
            }
        }
        return None;
    }

    fn stop(&mut self) {
        *self.is_running.lock().unwrap() = false;
    }
}

// Terminate the thread on Struct Drop
impl Drop for MacchinaM2 {
    fn drop(&mut self) {
        self.stop()
    }
}

const COMM_MSG_SIZE: usize = 2048;
const COMM_MSG_ARG_SIZE: usize = COMM_MSG_SIZE - 3;
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
/// Comm message that is sent and received fro the M2 module
struct COMM_MSG {
    msg_type: u8,                  // Message type
    arg_size: u16,                 // Arg size
    args: [u8; COMM_MSG_ARG_SIZE], // Args
}

impl std::fmt::Display for COMM_MSG {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MSG: TYPE={:02X} - Args=[", self.msg_type)?;
        (0..self.arg_size).for_each(|b| {
            write!(f, "{:02X}", self.args[b as usize]).unwrap();
            if b < self.arg_size - 1 {
                write!(f, " ").unwrap();
            }
        });
        write!(f, "]")
    }
}

impl PartialEq<COMM_MSG> for COMM_MSG {
    fn eq(&self, other: &COMM_MSG) -> bool {
        self.msg_type == other.msg_type && self.arg_size == other.arg_size
    }
}

impl COMM_MSG {
    pub fn from_vec(buf: &[u8]) -> Self {
        let mut args: [u8; COMM_MSG_ARG_SIZE] = [0x00; COMM_MSG_ARG_SIZE];
        (0..COMM_MSG_ARG_SIZE).for_each(|i| {
            args[i] = buf[i + 3];
        });
        COMM_MSG {
            msg_type: buf[0],
            arg_size: ((buf[1] as u16) << 8) | buf[2] as u16,
            args,
        }
    }

    pub fn new(msg_type: u8, arg_size: u16) -> Self {
        COMM_MSG {
            msg_type,
            arg_size,
            args: [0x00; COMM_MSG_ARG_SIZE],
        }
    }

    pub fn new_with_args(msg_type: u8, args_array: &[u8]) -> Self {
        let mut args: [u8; COMM_MSG_ARG_SIZE] = [0x00; COMM_MSG_ARG_SIZE];
        (0..std::cmp::min(args_array.len(), COMM_MSG_ARG_SIZE)).for_each(|i| {
            args[i] = args_array[i];
        });
        COMM_MSG {
            msg_type,
            arg_size: args_array.len() as u16,
            args,
        }
    }

    pub fn to_slice(&self) -> [u8; COMM_MSG_SIZE] {
        let mut msg: [u8; COMM_MSG_SIZE] = [0x00; COMM_MSG_SIZE];
        msg[0] = self.msg_type;
        msg[1] = ((self.arg_size >> 8) & 0xFF) as u8;
        msg[2] = (self.arg_size & 0xFF) as u8;

        (0..COMM_MSG_ARG_SIZE).for_each(|i| {
            msg[i + 3] = self.args[i];
        });
        return msg;
    }
}

#[test]
fn test_fmt() {
    let msg = COMM_MSG::new(0x01, 0x02);
    println!("{}", msg);

    let msg2 = COMM_MSG::new_with_args(0x01, &[0x02, 0x03, 0x04, 0x05]);
    println!("{}", msg2);
}

#[cfg(windows)]
const TEST_PORT: &str = "COM12";

#[cfg(unix)]
const TEST_PORT: &str = "/dev/ttyACM0";

#[cfg(test)]
use rand::Rng;

#[test]
fn test_io_m2() {
    let mut tx_errors = 0;
    let mut rec_fail = 0;
    let mut macchina = MacchinaM2::open_connection(TEST_PORT)
        .expect(format!("Could not open COM port {}!", TEST_PORT).as_str());

    // For this test, once we open the comm port,
    // Fire 100 random COMM messages at the M2 module
    // (Use type 0xFF to tell Macchina to echo back the same message)
    // Assert that each received message is the same as what was sent
    for _ in 0..100 {
        let args: Vec<u8> = (0..100)
            .map(|_| rand::thread_rng().gen_range(0, 0xFF))
            .collect();
        let msg = COMM_MSG::new_with_args(0xFF, &args);
        match macchina.write_and_read(msg, 100) {
            None => rec_fail += 1, // Macchina did not respond to our message
            Some(x) => {
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
