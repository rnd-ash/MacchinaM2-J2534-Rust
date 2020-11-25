#include "comm_channel.h"

#include "due_can.h"

// -- Base channel --
Channel::Channel(int id) {
    this->id = id;
}




// -- ISO15765 Channel --
ISO15765_Channel::ISO15765_Channel(int id, int mailbox_id, bool isExtended): Channel(id) {
    this->mailbox_id = mailbox_id;
    this->isExtended = isExtended;
}

void ISO15765_Channel::send_message(uint8_t* msg, uint16_t msg_len) {
    
}

void ISO15765_Channel::poll_for_message() {

}

void ISO15765_Channel::ioctl() {

}



// -- CAN Channel -- 