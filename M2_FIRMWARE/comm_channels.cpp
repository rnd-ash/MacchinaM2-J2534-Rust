#include "comm_channels.h"
#include "j2534_mini.h"

void CanChannel::ioctl(COMM_MSG *msg) {
    PCCOMM::respond_err(MSG_IOCTL, ERR_FAILED,"Not implemented");
}

bool CanChannel::setup(int id, int protocol, int baud, int flags) {
    // Here we go, setup a ISO15765 channel!
    if (Can0.init(baud) == 0) {
         PCCOMM::respond_err(MSG_OPEN_CHANNEL, ERR_FAILED, "CAN Controller setup failed!");
         return false;
    }
    if (flags & CAN_29BIT_ID > 0) { // extended addressing, 
        PCCOMM::log_message("CAN Extended enabled");
        this->isExtended = true;
    }

    // Can is OK, now blank set all mailboxes to a block state by default
    for (int i = 0; i < 7; i++) { // Extended boxes
        Can0.setRXFilter(i, 0x0000, 0xFFFF, isExtended);
    }
    digitalWrite(DS3, LOW); // Enable the light
    this->channel_id = id;
    this->f.length = 0;
    return true;
}

void CanChannel::addFilter(int type, int filter_id, char* mask, char* pattern, char* flowcontrol, int mask_len, int pattern_len, int flowcontrol_len) {
     if (type == FLOW_CONTROL_FILTER) {
        PCCOMM::respond_err(MSG_SET_CHAN_FILT, ERR_FAILED, "CAN Channel cannot use flow control filter");
        return;
    }
    if (mask_len > 4) {
        PCCOMM::respond_err(MSG_SET_CHAN_FILT, ERR_FAILED, "Mask length too big");
        return;
    }
    if (pattern_len > 4) {
        PCCOMM::respond_err(MSG_SET_CHAN_FILT, ERR_FAILED, "Pattern length too big");
        return;
    }
     if (filter_id >= 7) { // Out of mailboxes!
        PCCOMM::respond_err(MSG_SET_CHAN_FILT, ERR_EXCEEDED_LIMIT, nullptr);
        return;
    }
    if (used_mailboxes[filter_id] == true) {
        PCCOMM::respond_err(MSG_SET_CHAN_FILT, ERR_FAILED, "Filter ID in use");
        return;
    }

    uint32_t mask_id = 0x0000;
    uint32_t ptn_id = 0x0000;

    for (int i = 0; i < mask_len; i++) {
        mask_id <<= 8;
        mask_id |= mask[i];
    }

    for (int i = 0; i < pattern_len; i++) {
        ptn_id <<= 8;
        ptn_id |= pattern[i];
    }
    char buf[20];
    sprintf(buf, "%04X %04X", ptn_id, mask_id);
    PCCOMM::log_message(buf);
    if (type == BLOCK_FILTER) { // Block filter. Set the CAN Filter ID to be open, and then we will block it in software
        Can0.setRXFilter(filter_id, 0x0000, 0x0000, isExtended); // Open the mailbox filter to everything
        blocking_filters[filter_id] = true; // Mark this as yes for the update function
        memcpy(&blocking_filters[filter_id], &ptn_id, 4); // Set the pattern ID for blocking
        memcpy(&blocking_filter_masks[filter_id], &mask_id, 4); // Set the mask ID for blocking
    } else { // Pass filter, use hardware filter
        Can0.setRXFilter(filter_id, ptn_id, mask_id, isExtended);
    }
    used_mailboxes[filter_id] = true;
    PCCOMM::respond_ok(MSG_SET_CHAN_FILT, nullptr, 0);
}

void CanChannel::update() {
    for (int i = 0; i < 7; i++) {
        if (used_mailboxes[i] == true) { // We should check this mailbox
            Can0.mailbox_read(i, &f);
            if (f.length != 0) {
                if (blocking_filters[i] == true) { // Blocking filter, do the pattern matching in software
                    if (blocking_filter_masks[i] & f.id != blocking_filters[i]) {
                        char buf[f.length + 4];
                        memcpy(&buf[0], &f.id, 4); // Copy ID
                        memcpy(&buf[4], &f.data.bytes[0], f.length); // Copy data
                        PCCOMM::tx_data(this->channel_id, buf, f.length+4);
                    }
                } else { // Pass filter, simply send it to the PC!
                    char buf[f.length + 4];
                    memcpy(&buf[0], &f.id, 4);
                    memcpy(&buf[4], &f.data.bytes[0], f.length); // Copy ID
                    PCCOMM::tx_data(this->channel_id, buf, f.length+4); // Copy data
                }
                f.length = 0x0; // Reset to this so that we don't read it again and again...
            }
        }
    }
}

void CanChannel::removeFilter(int id) {

}

void CanChannel::destroy() {
    // Set all mailboxes to a blocked state
    for (int i = 0; i < 7; i++) { // Extended boxes
        Can0.setRXFilter(i, 0x0000, 0xFFFF, isExtended);
    }
    Can0.disable(); // Bye bye CAN0
    digitalWrite(DS3, HIGH); // Disable the light
}

/**
 * Macchina will NOT respond to this request, just send and leave it
 */
void CanChannel::sendMsg(char* data, int data_size) {
    // First 4 bytes are CAN ID, followed by the CAN Data
    CAN_FRAME f;
    f.length = data_size - 4;
    memcpy(&f.id, &data[0], 4);
    memcpy(&f.data.bytes[0], &data[4], data_size-4);
    digitalWrite(DS7_GREEN, LOW);
    Can0.sendFrame(f);
    digitalWrite(DS7_GREEN, HIGH);
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
        Can0.setRXFilter(i, 0x0000, 0xFFFF, isExtended);
    }
    digitalWrite(DS3, LOW); // Enable the light
    this->channel_id = id;
    return true;
}

void ISO15765Channel::addFilter(int type, int filter_id, char* mask, char* pattern, char* flowcontrol, int mask_len, int pattern_len, int flowcontrol_len) {
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
    if (filter_id >= 7) {
        PCCOMM::respond_err(MSG_SET_CHAN_FILT, ERR_EXCEEDED_LIMIT, nullptr);
        return;
    }
    if (this->used_mailboxes[filter_id] == true) {
        PCCOMM::respond_err(MSG_SET_CHAN_FILT, ERR_FAILED, "Filter ID already in use");
        return;
    }
    // Filter is free, set it!
    this->used_mailboxes[filter_id] = true;
    this->flowcontrol_ids[filter_id] = flowcontrol_u32;
    Can0.setRXFilter(filter_id, pattern_u32, mask_u32, isExtended);

    // Filter set! respond with the just OK
    PCCOMM::respond_ok(MSG_SET_CHAN_FILT, nullptr, 0);
}

void ISO15765Channel::removeFilter(int id) {
    if (this->used_mailboxes[id] == true) {
        this->used_mailboxes[id] = false;
        this->flowcontrol_ids[id] = 0x00;
        Can0.setRXFilter(id, 0x0000, 0xFFFF, this->isExtended);
        PCCOMM::respond_ok(MSG_REM_CHAN_FILT, nullptr, 0);
    } else {
        PCCOMM::respond_err(MSG_REM_CHAN_FILT, ERR_FAILED, "Filter does not exist!");
    }
}

void ISO15765Channel::destroy() {
    // Set all mailboxes to a blocked state
    for (int i = 0; i < 7; i++) { // Extended boxes
        Can0.setRXFilter(i, 0x0000, 0xFFFF, isExtended);
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