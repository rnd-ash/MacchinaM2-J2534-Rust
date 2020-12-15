#include "comm_channels.h"
#include "j2534_mini.h"
#include "due_can.h"

void CanChannel::ioctl(COMM_MSG *msg) {

}
bool CanChannel::setup(int id, int protocol, int baud, int flags) {
    PCCOMM::respond_err(MSG_OPEN_CHANNEL, ERR_FAILED, "Not implemented");
    return false;
}

void CanChannel::addFilter(int type, char* mask, char* pattern, char* flowcontrol, int mask_len, int pattern_len, int flowcontrol_len) {
}
void CanChannel::removeFilter(int id) {

}
void CanChannel::destroy() {

}
void CanChannel::update() {

}

void CanChannel::sendMsg(char* data, int data_size) {
}

void ISO15765Channel::ioctl(COMM_MSG *msg) {

}
bool ISO15765Channel::setup(int id, int protocol, int baud, int flags) {
    // Here we go, setup a ISO15765 channel!
    if (Can0.init(baud) == 0) {
         PCCOMM::respond_err(MSG_OPEN_CHANNEL, ERR_FAILED, "CAN Controller setup failed!");
         return false;
    }
    if (flags & CAN_29BIT_ID > 0) { // extended addressing, 
        this->isExtended = true;
    }

    // Can is OK, now blank set all mailboxes to a block state by default
    for (int i = 0; i < 7; i++) { // Extended boxes
        Can0.setRXFilter(i, 0x0000, isExtended);
    }
    digitalWrite(DS3, LOW); // Enable the light
    return true;
}

void ISO15765Channel::addFilter(int type, char* mask, char* pattern, char* flowcontrol, int mask_len, int pattern_len, int flowcontrol_len) {
    if (type != FLOW_CONTROL_FILTER) {
        PCCOMM::respond_err(MSG_SET_CHAN_FILT, ERR_FAILED, "ISO15765 filter not valid type");
        return;
    }
    if (mask_len != 4) {
        PCCOMM::respond_err(MSG_SET_CHAN_FILT, ERR_FAILED, "Mask length not 4");
        return;
    }
    if (pattern_len != 4) {
        PCCOMM::respond_err(MSG_SET_CHAN_FILT, ERR_FAILED, "Pattern length not 4");
        return;
    }
    if (flowcontrol_len != 4) {
        PCCOMM::respond_err(MSG_SET_CHAN_FILT, ERR_FAILED, "Flowcontrol length not 4");
        return;
    }
    uint32_t mask_u32;
    uint32_t pattern_u32;
    uint32_t flowcontrol_u32;
    memcpy(&mask_u32, mask, 4);
    memcpy(&pattern_u32, pattern, 4);
    memcpy(&flowcontrol_u32, flowcontrol, 4);
    bool filter_set = false;
    int filter_id = 0;
    for (int i = 0; i < 7; i++) {
        if (this->used_mailboxes[i] == false) { // Mailbox is free!
            this->used_mailboxes[i] = true;
            this->flowcontrol_ids[i] = flowcontrol_u32;
            Can0.setRXFilter(i, mask_u32, pattern_u32, isExtended);
            filter_set = true;
            filter_id = i;
            break;
        }
    }
    if (!filter_set) {
        // Too many filters - Cannot continue
        PCCOMM::respond_err(MSG_SET_CHAN_FILT, ERR_EXCEEDED_LIMIT, nullptr);
    } else {
        // Filter set! respond with the filter ID
        PCCOMM::respond_ok(MSG_SET_CHAN_FILT, (uint8_t*)filter_id, 4);
    }
}

void ISO15765Channel::removeFilter(int id) {
    if (this->used_mailboxes[id] == true) {
        this->used_mailboxes[id] = false;
        Can0.setRXFilter(id, 0x0000, this->isExtended);
        PCCOMM::respond_ok(MSG_REM_CHAN_FILT, nullptr, 0);
    } else {
        PCCOMM::respond_err(MSG_REM_CHAN_FILT, ERR_FAILED, "Filter does not exist!");
    }
}

void ISO15765Channel::destroy() {
    // Set all mailboxes to a blocked state
    for (int i = 0; i < 7; i++) { // Extended boxes
        Can0.setRXFilter(i, 0x0000, isExtended);
    }
    Can0.disable(); // Bye bye CAN0
    digitalWrite(DS3, HIGH); // Disable the light
}

void ISO15765Channel::update() {

}

void ISO15765Channel::sendMsg(char* data, int data_size) {
    if (this->isSending) {
        PCCOMM::respond_err(MSG_TX_CHAN_DATA, ERR_BUFFER_FULL, nullptr);
        return;
    }
    CAN_FRAME write;
    if (data_size <= 11) { // one frame!
        write.extended = this->isExtended;
        write.length = 8;
        write.data.bytes[0] = data_size - 4; // PCI (-4 as first 4 bytes are CID)
        write.priority = 4; // Balanced priority
        memcpy(&write.id, data, 4); // Copy CID
        memcpy(&write.data.bytes[1], &data[4], data_size-4);
        Can0.sendFrame(write);
        PCCOMM::respond_ok(MSG_TX_CHAN_DATA, nullptr, 0);
    } else {
        // TODO Multi frame data write
        PCCOMM::respond_err(MSG_TX_CHAN_DATA, ERR_FAILED, "Multi frame CAN Not supported");
    }
}