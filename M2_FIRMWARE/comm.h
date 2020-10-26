#ifndef COMM_H_
#define COMM_H_

#include <stdint.h>


#define MSG_LOG 0x01
#define MSG_VBATT 0x02


typedef struct {
    uint8_t msg_type;
    uint16_t arg_size;
    uint8_t args[4096];
} COMM_MSG;

#endif