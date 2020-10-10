#[macro_use]
extern crate bitflags;

// SAE J2534 API definition,
// Based on J2534.h

#[derive(Debug)]
#[allow(non_camel_case_types, dead_code)]
enum Protocol {
    J1850VPW = 0x01,
    J1850PWM = 0x02,
    ISO9141 = 0x03,
    ISO14230 = 0x04,
    CAN = 0x05,
    ISO15765 = 0x06,
    SCI_A_ENGINE = 0x07,
    SCI_A_TRANS = 0x08,
    SCI_B_ENGINE = 0x09,
    SCI_B_TRANS = 0x0A
}


#[derive(Debug)]
#[allow(non_camel_case_types, dead_code)]
enum IoctlID {         //   Input           Output
    GET_CONFIG = 0x01, // SCONFIG_LIST      NULL
    SET_CONFIG = 0x02, // SCONFIG_LIST      NULL
    READ_VBATT = 0x03, // NULL              u64
    FIVE_BAUD_INIT = 0x05,
    FAST_INIT = 0x06,
    CLEAR_TX_BUFFER = 0x07,
    CLEAR_RX_BUFFER = 0x08,
    CLEAR_PERIODIC_MSGS = 0x09,
    CLEAR_MSG_FILTERS = 0x0A,
    CLEAR_FUNCT_MSG_LOOKUP_TABLE = 0x0B,
    ADD_TO_FUNCT_MSG_LOOKUP_TABLE = 0x0C,
    DELETE_FROM_FUNCT_MSG_LOOKUP_TABLE = 0x0D,
    READ_PROG_VOLTAGE = 0x0E
}

#[derive(Debug)]
#[allow(non_camel_case_types, dead_code)]
enum IoctlParam {
    DATA_RATE = 0x01,
    LOOPBACK = 0x03,

    NODE_ADDRESS = 0x04,
    NETWORK_LINE = 0x05,
    P1_MIN = 0x06,
    P1_MAX = 0x07,
    P2_MIN = 0x08,
    P2_MAX = 0x09,
    P3_MIN = 0x0A,
    P3_MAX = 0x0B,
    P4_MIN = 0x0C,
    P4_MAX = 0x0D,
    W1 = 0x0E,
    W2 = 0x0F,
    W3 = 0x10,
    W4 = 0x11,
    W5 = 0x12,

    TIDLE = 0x13,
    TINL = 0x14,
    TWUP = 0x15,
    PARITY = 0x16,
    BIT_SAMPLE_POINT = 0x17,
    SYNCH_JUMP_WIDTH = 0x18,
    W0 = 0x19,
    T1_MAX = 0x1A,
    T2_MAX = 0x1B,
    T4_MAX = 0x1C,
    T5_MAX = 0x1D,
    ISO15765_BS = 0x1E,
    ISO15765_STMIN = 0x1F,

    DATA_BITS = 0x20,
    FIVE_BAUD_MOD = 0x21,
    BS_TX = 0x22,
    STMIN_TX = 0x23,
    T3_MAX = 0x24,
    ISO15765_WFT_MAX = 0x25
}

#[derive(Debug)]
#[allow(non_camel_case_types, dead_code)]
enum PassthruError {
    STATUS_NOERROR = 0x00,
    ERR_NOT_SUPPORTED = 0x01,
    ERR_INVALID_CHANNEL_ID = 0x02,
    ERR_INVALID_PROTOCOL_ID = 0x03,
    ERR_NULL_PARAMETER = 0x04,
    ERR_INVALID_IOCTL_VALUE = 0x05,
    ERR_INVALID_FLAGS = 0x06,
    ERR_FAILED = 0x07,
    ERR_DEVICE_NOT_CONNECTED = 0x08,
    ERR_TIMEOUT = 0x09,

    ERR_INVALID_MSG = 0x0A,
    ERR_INVALID_TIME_INTERVAL = 0x0B,
    ERR_EXCEEDED_LIMIT = 0x0C,
    ERR_INVALID_MSG_ID = 0x0D,
    ERR_DEVICE_IN_USE = 0x0E,
    ERR_INVALID_IOCTL_ID = 0x0F,
    ERR_BUFFER_EMPTY = 0x10,
    ERR_BUFFER_FULL = 0x11,
    ERR_BUFFER_OVERFLOW = 0x12,
    ERR_PIN_INVALUD = 0x13,
    ERR_CHANNEL_IN_USE = 0x14,
    ERR_MSG_PROTOCOL_ID = 0x15,

    ERR_INVALID_FILTER_ID = 0x16,
    ERR_NO_FLOW_CONTROL = 0x17,
    ERR_NOT_UNIQUE = 0x18,
    ERR_INVALID_BAUDRATE = 0x19,
    ERR_INVALID_DEVICE_ID = 0x1A
}

#[derive(Debug)]
#[allow(non_camel_case_types, dead_code)]
enum FilterType {
    PASS_FILTER = 0x01,
    BLOCK_FILTER = 0x02,
    FLOW_CONTROL_FILTER = 0x03
}

#[derive(Debug)]
#[allow(non_camel_case_types, dead_code)]
enum LoopBackSetting {
    OFF = 0x00,
    ON = 0x01
}

#[derive(Debug)]
#[allow(non_camel_case_types, dead_code)]
enum DataBits {
    DATA_BITS_8 = 0x00,
    DATA_BITS_7 = 0x01
}

#[derive(Debug)]
#[allow(non_camel_case_types, dead_code)]
enum ParitySetting {
    NO_PARITY = 0x00,
    ODD_PARITY = 0x01,
    EVEN_PARITY = 0x02
}

#[derive(Debug)]
#[allow(non_camel_case_types, dead_code)]
enum J1850PWMNetworkLine {
    BUS_NORMAL = 0x00,
    BUS_PLUS = 0x01,
    BUS_MINUS = 0x02
}

#[derive(Debug)]
#[allow(non_camel_case_types, dead_code)]
enum ConnectFlags {
    CAN_29BIT_ID = 0x00000100,
    ISO9141_NO_CHECKSUM = 0x00000200,
    CAN_ID_BOTH = 0x00000800,
    ISO9141_K_LINE_ONLY = 0x00001000
}


bitflags! {
    pub struct RxFlag: u64 {
        const CAN_29BIT_ID = 0x00000100;
        const ISO15765_ADDR_TYPE = 0x00000080;
        const ISO15765_PADDING_ERROR = 0x00000010;
        const TX_DONE = 0x00000008;
        const RX_BREAK = 0x00000004;
        const ISO15765_FIRST_FRAME = 0x00000002;
        const START_OF_MESSAGE = 0x00000002;
        const TX_MSG_TYPE = 0x00000001;
    }
}

bitflags! {
    pub struct TxFlag: u64 {
        const SCI_TX_VOLTAGE = 0x00800000;
        const SCI_MODE = 0x00400000;
        const BLOCKING = 0x00010000;
        const WAIT_P3_MIN_ONLY = 0x00000200;
        const CAN_29BIT_ID = 0x00000100;
        const CAN_EXTENDED_ID = 0x00000100;
        const ISO15765_ADDR_TYPE = 0x00000080;
        const ISO15765_EXT_ADDR = 0x00000080;
        const ISO15765_FRAME_PAD = 0x00000040;
        const TX_NORMAL_TRANSMIT = 0x00000000;
    }
}



#[derive(Copy, Clone)]
#[repr(C, packed(1))]
pub struct PASSTHRU_MSG {
    pub protocol_id: u64,
    pub rx_status: u64,
    pub tx_flags: u64,
    pub timestamp: u64,
    pub data_size: u64,
    pub extra_data_size: u64,
    pub data: [u8; 4128]
}

#[derive(Copy, Clone)]
#[repr(C, packed(1))]
pub struct SBYTE_ARRAY {
    pub num_of_bytes: u64,
    pub byte_ptr: *const u8
}

#[derive(Copy, Clone)]
#[repr(C, packed(1))]
pub struct SConfig {
    pub parameter: u64,
    pub value: u64
}

#[derive(Copy, Clone)]
#[repr(C, packed(1))]
pub struct SConfigList {
    pub num_of_params: u64,
    pub config_ptr: *const SConfig
}