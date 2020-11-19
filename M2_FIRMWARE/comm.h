#ifndef COMM_H_
#define COMM_H_

#include <stdint.h>
#include <Arduino.h>


#define MSG_UNKNOWN 0x00
#define MSG_LOG 0x01
#define MSG_OPEN_CHANNEL 0x02  //[ID, Baud, Baud, Protocol]
#define MSG_CLOSE_CHANNEL 0x03
#define MSG_CHANNEL_DATA 0x04
#define MSG_READ_BATT 0x05
#define MSG_STATUS 0xAA // Args: [0] -> 0x00 = Goodbye, 0x01 = Hellow
#define MSG_GET_FW_VERSION 0xAB
#define MSG_TEST 0x0FF

// Reserve ~5Kb of memory for a temp buffer for reading and writing comm messages
#define BUFFER_SIZE 4096

//
struct __attribute__ ((packed)) COMM_MSG {
    uint8_t msg_id;
    uint8_t msg_type;
    uint16_t arg_size;
    uint8_t args[4092]; // 2044
};

namespace PCCOMM {
    bool read_message(COMM_MSG *msg);
    void send_message(COMM_MSG *msg);
    void log_message(char* msg);
};

#endif