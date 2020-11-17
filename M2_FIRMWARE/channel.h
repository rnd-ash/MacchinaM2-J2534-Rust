#include <Arduino.h>
#include "due_can.h"
#include "comm.h"

class ChannelHandler {
    
}


class KLineChannel {

};


class CanChannel {
public:
    static uint8_t CHANNELS_LEFT = 4; // Mainboxes 3-7 are normal CAN
    static uint8_t CHANNELS_LEFT_EXT = 3; // Mailboxes 0-3 are extended CAN
    CanChannel(req_msg: &COMM_MSG);
    void destroy_channel(req_msg: &COMM_MSG);
private:

}