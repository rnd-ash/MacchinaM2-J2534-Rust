#include "comm_channel.h"
#include "comm.h"

// -- Base channel --
Channel::Channel(int id) {
    this->id = id;
}


// -- ISO15765 Channel --
ISO15765_Channel::ISO15765_Channel(int id, int mailbox_id, bool isExtended): Channel(id) {
    this->mailbox_id = mailbox_id;
    this->isExtended = isExtended;
    this->frame = new CAN_FRAME;
}

void ISO15765_Channel::update() {
    this->poll_for_message();
}

void ISO15765_Channel::send_message(uint8_t* msg, uint16_t msg_len) {
    
}

void ISO15765_Channel::poll_for_message() {
    if (Can0.mailbox_read(this->mailbox_id, this->frame)) {
        PCCOMM::log_message("ISO15765 has CAN Frame!");
    }
}

void ISO15765_Channel::ioctl() {

}

void ISO15765_Channel::remove() {

}



// -- CAN Channel -- 