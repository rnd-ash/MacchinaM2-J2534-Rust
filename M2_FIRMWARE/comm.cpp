#include "comm.h"
#include <HardwareSerial.h>



namespace PCCOMM {
    uint8_t last_id = 0;
    char tempbuf[BUFFER_SIZE];
    uint16_t read_count = 0;

    bool read_message(COMM_MSG *msg) {
        if(SerialUSB.available() > 0) { // Is there enough data in the buffer for
            digitalWrite(DS7_BLUE, LOW);
            // Calculate how many bytes to read (min of avaliable bytes, or left to read to complete the data)
            uint16_t maxRead = min(SerialUSB.available(), sizeof(COMM_MSG)-read_count);
            SerialUSB.readBytes(&tempbuf[read_count], maxRead);
            read_count += maxRead;

            // Size OK now, full payload received
            if(read_count == sizeof(COMM_MSG)) {
                memcpy(msg, &tempbuf, sizeof(COMM_MSG));
                read_count = 0;
                memset(tempbuf, 0x00, sizeof(tempbuf)); // Reset buffer
                if (msg->msg_id != 0x00) {
                    last_id = msg->msg_id;
                }
                digitalWrite(DS7_BLUE, HIGH);
                return true;
            }
        }
        return false;
    }

    void send_message(COMM_MSG *msg) {
        digitalWrite(DS7_RED, LOW);
        SerialUSB.write((char*)msg, sizeof(COMM_MSG));
        SerialUSB.flush();
        digitalWrite(DS7_RED, HIGH);
    }

    // This is used for log_message, respond_ok and respond_err
    COMM_MSG res = {0x00};

    void log_message(char* msg) {
        res.msg_type = MSG_LOG;
        res.arg_size = min(strlen(msg), 4095);
        memcpy(&res.args[0], msg, res.arg_size);
        send_message(&res);
    }

    void respond_ok(uint8_t op, uint8_t* args, uint16_t arg_size) {
        res.msg_type = op;
        res.arg_size = 1 + arg_size;
        res.msg_id = last_id;
        res.args[0] = 0x00; // STATUS_NOERROR
        if (arg_size != 0) {
            memcpy(&res.args[1], args, min(arg_size, COMM_MSG_ARG_SIZE));
        }
        send_message(&res);
    }

    void respond_err(uint8_t op, uint8_t error_id, char* txt) {
        res.msg_type = op;
        res.arg_size = 1 + strlen(txt);
        res.args[0] = error_id;
        res.msg_id = last_id;
        memcpy(&res.args[1], txt, strlen(txt));
        send_message(&res);
    }
}