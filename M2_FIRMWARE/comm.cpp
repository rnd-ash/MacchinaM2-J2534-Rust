#include "comm.h"


namespace PCCOMM {
    char tempbuf[BUFFER_SIZE];
    uint16_t read_count = 0;

    bool read_message(COMM_MSG *msg) {
        if(SerialUSB.available() > 0) { // Is there enough data in the buffer for

            // Calculate how many bytes to read (min of avaliable bytes, or left to read to complete the data)
            uint16_t maxRead = min(SerialUSB.available(), sizeof(COMM_MSG)-read_count);
            digitalWrite(DS7_BLUE, LOW);
            SerialUSB.readBytes(&tempbuf[read_count], maxRead);
            digitalWrite(DS7_BLUE, HIGH);
            read_count += maxRead;

            // Size OK now, full payload received
            if(read_count == sizeof(COMM_MSG)) {
                memcpy(msg, &tempbuf, sizeof(COMM_MSG));
                read_count = 0;
                memset(tempbuf, 0x00, sizeof(tempbuf)); // Reset buffer
                //lastID = msg->msg_id; // Set this for response
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

    void log_message(char* msg) {
        COMM_MSG res = {0x00};
        res.msg_type = MSG_LOG;
        res.arg_size = min(strlen(msg), 2045);
        memcpy(&res.args[0], msg, res.arg_size);
        send_message(&res);
    }
}