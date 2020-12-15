#include "comm.h"
#include <HardwareSerial.h>



namespace PCCOMM {
    uint8_t last_id = 0;
    char* tempbuf;
    uint16_t read_count = 0;
    bool isReadingMsg = false;
    uint16_t read_target = 0;
    bool read_message(COMM_MSG *msg) {
        if (!isReadingMsg && SerialUSB.available() >= 2) { // Starter of payload
            digitalWrite(DS7_BLUE, LOW);
            isReadingMsg = true;
            read_count = 0;
            char buf[2];
            SerialUSB.readBytes(buf, 2);
            memcpy(&read_target, buf, 2);
            tempbuf = new char[read_target];
            return false;
        } else if(SerialUSB.available() > 0) { // Just reading data
            // Calculate how many bytes to read (min of avaliable bytes, or left to read to complete the data)
            uint16_t maxRead = min(SerialUSB.available(), read_target-read_count);
            SerialUSB.readBytes(&tempbuf[read_count], maxRead);
            read_count += maxRead;

            // Size OK now, full payload received
            if(read_count == read_target) {
                msg->arg_size = read_target - 2;
                isReadingMsg = false;
                msg->msg_id = tempbuf[0];
                msg->msg_type = tempbuf[1];
                memcpy(msg->args, &tempbuf[2], msg->arg_size);
                if (msg->msg_id != 0x00) {
                    last_id = msg->msg_id;
                }
                delete[] tempbuf;
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