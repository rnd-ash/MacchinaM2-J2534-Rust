#ifndef COMM_H_
#define COMM_H_

#include <stdint.h>
#include <Arduino.h>


#define MSG_LOG 0x01
#define MSG_VBATT 0x02

// Reserve ~5Kb of memory for a temp buffer for reading and writing comm messages
#define BUFFER_SIZE 2048

//
struct __attribute__ ((packed)) COMM_MSG {
    uint8_t msg_type;
    uint16_t arg_size;
    uint8_t args[2045];
};

namespace PCCOMM {
    bool read_message(COMM_MSG *msg);
    void send_message(COMM_MSG *msg);
    void log_message(char* msg);
};

#endif