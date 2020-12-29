#include "comm_channels.h"
#include "j2534_mini.h"

// Debug function
bool debug_send_frame(CAN_FRAME &f) {
    char buf[80] = {0x00};
    char *pos = buf;
    pos += sprintf(pos, "Send frame -> %04X (LEN: %d) [", f.id, f.length);
    for (int i = 0; i < f.length; i++) {
        pos+=sprintf(pos, "%02X ", f.data.bytes[i]);
    }
    sprintf(pos-1,"]");
    PCCOMM::log_message(buf);
    digitalWrite(DS7_GREEN, LOW);
    bool res = Can0.sendFrame(f);
    digitalWrite(DS7_GREEN, HIGH);
    return res;
}

void debug_read_frame(CAN_FRAME &f) {
    char buf[80] = {0x00};
    char *pos = buf;
    pos += sprintf(pos, "Read frame -> %04X (LEN: %d) [", f.id, f.length);
    for (int i = 0; i < f.length; i++) {
        pos+=sprintf(pos, "%02X ", f.data.bytes[i]);
    }
    sprintf(pos-1,"]");
    PCCOMM::log_message(buf);
}


void CanChannel::ioctl(COMM_MSG *msg) {
    PCCOMM::respond_err(MSG_IOCTL, ERR_FAILED,"Not implemented");
}

bool CanChannel::setup(int id, int protocol, int baud, int flags) {
    // Here we go, setup a ISO15765 channel!
    if (Can0.init(baud) == 0) {
         PCCOMM::respond_err(MSG_OPEN_CHANNEL, ERR_FAILED, "CAN Controller setup failed!");
         return false;
    }
    //Can0.reset_all_mailbox();
    if (flags & CAN_29BIT_ID > 0) { // extended addressing, 
        PCCOMM::log_message("CAN Extended enabled");
        this->isExtended = true;
    }
    for (int i = 0; i < 7; i++) {
        Can0.setRXFilter(i, 0xFFFF, 0x0000, isExtended);
    }
    // Can is OK, now blank set all mailboxes to a block state by default
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

    if (type == BLOCK_FILTER) { // Block filter. Set the CAN Filter ID to be open, and then we will block it in software
        Can0.setRXFilter(filter_id, 0x0000, 0x0000, isExtended); // Open the mailbox filter to everything
        blocking_filters[filter_id] = true; // Mark this as yes for the update function
    } else { // Pass filter, use hardware filter
        Can0.setRXFilter(filter_id, ptn_id, mask_id, isExtended);
        blocking_filters[filter_id] = false;

    }
    patterns[filter_id] = ptn_id;
    masks[filter_id] = mask_id;
    used_mailboxes[filter_id] = true;
    PCCOMM::respond_ok(MSG_SET_CHAN_FILT, nullptr, 0);
}

void CanChannel::update() {
    if (Can0.read(f)) {
        for (int i = 0; i < 7; i++) { // Check all our filters in use
            if (used_mailboxes[i] == true) { // We should this filter
                bool send_frame = false;
                if (blocking_filters[i] == true) { // Check block filter
                    send_frame = masks[i] & f.id != patterns[i]; // Block filter check
                } else { // Check pass filter
                    send_frame = masks[i] & f.id == patterns[i]; // Pass filter check
                }
                if (send_frame) { // Frame should be sent to the PC
                    char buf[f.length + 4];
                    // TODO - Rx Flags for CAN - Although i don't think they are needed, so leave them 0x0000
                    uint32_t rx_status = 0x0000;

                    memcpy(&buf[0], &f.id, 4); // Copy CAN ID
                    memcpy(&buf[4], &f.data.bytes[0], f.length);  // Copy CAN Data
                    PCCOMM::send_rx_data(this->channel_id, rx_status, buf, f.length+4); // Tx to PC
                }
            }
        }
    }
}

void CanChannel::removeFilter(int id) {
    if (this->used_mailboxes[id] == true) {
        this->used_mailboxes[id] = false;
        this->masks[id] = 0;
        this->patterns[id] = 0;
        this->blocking_filters[id] = false;
        CustomCan::disableCanFilter(id);
        PCCOMM::respond_ok(MSG_REM_CHAN_FILT, nullptr, 0);
    } else {
        PCCOMM::respond_err(MSG_REM_CHAN_FILT, ERR_INVALID_MSG_ID, nullptr);
    }
}

void CanChannel::destroy() {
    
    // Set all mailboxes to a blocked state
    for (int i = 0; i < 7; i++) { // Extended boxes
        Can0.setRXFilter(i, 0xFFFF, 0x0000, isExtended);
        this->used_mailboxes[i] = false;
    }
    Can0.disable(); // Bye bye CAN0
    digitalWrite(DS3, HIGH); // Disable the light
}

void CanChannel::on_frame_receive(CAN_FRAME *f) {
    
}

/**
 * Macchina will NOT respond to this request, just send and leave it
 */
void CanChannel::sendMsg(uint32_t tx_flags, char* data, int data_size, bool respond) {
    // First 4 bytes are CAN ID, followed by the CAN Data
    CAN_FRAME f;
    f.length = data_size - 4;
    f.id = data[0] << 24 | data[1] << 16 | data[2] << 8 | data[3] << 0;
    memcpy(&f.data.bytes[0], &data[4], data_size-4);
    digitalWrite(DS7_GREEN, LOW);
    Can0.sendFrame(f);
    digitalWrite(DS7_GREEN, HIGH);
    if (respond) {
        PCCOMM::respond_ok(MSG_TX_CHAN_DATA, nullptr, 0);
    }
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
        PCCOMM::log_message("Extended CAN detected!");
        this->isExtended = true;
    } else {
        PCCOMM::log_message("Standard CAN detected!");
        this->isExtended = false;
    }

    CustomCan::enableCanBus(baud);
    digitalWrite(DS3, LOW); // Enable the light
    this->channel_id = id;

    this->txPayload = {nullptr, 0, 0};
    this->rxPayload = {nullptr, 0, 0};
    this->isSending = false;
    this->isReceiving = false;
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
    if (filter_id >= 7) {
        PCCOMM::respond_err(MSG_SET_CHAN_FILT, ERR_EXCEEDED_LIMIT, nullptr);
        return;
    }
    if (this->used_mailboxes[filter_id] == true) {
        PCCOMM::respond_err(MSG_SET_CHAN_FILT, ERR_FAILED, "Filter ID already in use");
        return;
    }

    uint32_t mask_u32 = mask[0] << 24 | mask[1] << 16 | mask[2] << 8 | mask[3];
    uint32_t pattern_u32 = pattern[0] << 24 | pattern[1] << 16 | pattern[2] << 8 | pattern[3];
    uint32_t flowcontrol_u32 = flowcontrol[0] << 24 | flowcontrol[1] << 16 | flowcontrol[2] << 8 | flowcontrol[3];
    // Filter is free, set it!
    this->used_mailboxes[filter_id] = true;
    this->mask_ids[filter_id] = mask_u32;
    this->pattern_ids[filter_id] = pattern_u32;
    this->flowcontrol_ids[filter_id] = flowcontrol_u32;
    CustomCan::enableCanFilter(filter_id, pattern_u32, mask_u32, isExtended);
    PCCOMM::respond_ok(MSG_SET_CHAN_FILT, nullptr, 0);
}

void ISO15765Channel::removeFilter(int id) {
    if (this->used_mailboxes[id] == true) {
        this->used_mailboxes[id] = false;
        this->flowcontrol_ids[id] = 0x00;
        CustomCan::disableCanFilter(id);
        PCCOMM::respond_ok(MSG_REM_CHAN_FILT, nullptr, 0);
    } else {
        PCCOMM::respond_err(MSG_REM_CHAN_FILT, ERR_FAILED, "Filter does not exist!");
    }
}

void ISO15765Channel::destroy() {
    CustomCan::disableCanBus();
    digitalWrite(DS3, HIGH); // Disable the light
}

void ISO15765Channel::update() {
    for (int i = 0; i < 7; i++) {
        if (used_mailboxes[i] == true) {
            if (CustomCan::receiveFrame(i, &f)) {
                debug_read_frame(f);
                switch(f.data.bytes[0] & 0xF0) {
                case 0x00:
                    tx_single_frame(&f);
                    break;
                case 0x10:
                    send_ff_indication(&f, i);
                    break;
                case 0x20:
                    tx_multi_frame(&f, i);
                    break;
                case 0x30:
                    PCCOMM::log_message("FIXME. Cannot process incoming flow control messages");
                    break;
                default:
                    char buf[70];
                    sprintf(buf, "CAN ID %04X invalid IOS-TP PCI: %02X. Discarding frame", f.id, f.data.bytes[0]);
                    PCCOMM::log_message(buf);
                    break;
                }
            }
        }
    }
    if (isSending) {

    }
}


void ISO15765Channel::tx_single_frame(CAN_FRAME *read) {
    uint8_t size = read->data.bytes[0] + 4;
    char* buf = new char[size];
    // Copy CAN ID
    buf[0] = read->id >> 24;
    buf[1] = read->id >> 16;
    buf[2] = read->id >> 8;
    buf[3] = read->id >> 0;
    memcpy(&buf[4], &read->data.bytes[1], read->data.bytes[0]);
    PCCOMM::send_rx_data(this->channel_id, 0x0000, buf, size);
    delete[] buf;
}


void ISO15765Channel::tx_multi_frame(CAN_FRAME *read, int id) {
    if (!this->isReceiving) {
        PCCOMM::log_message("Multi frame message received but not start frame!?");
        return;
    }
    uint8_t max_copy = min(rxPayload.payloadSize - rxPayload.payloadPos, 7); // Up to 7 bytes
    memcpy(&rxPayload.payload[rxPayload.payloadPos] ,&read->data.bytes[1], max_copy);

    rxPayload.payloadPos += max_copy;
    if (rxPayload.payloadPos >= rxPayload.payloadSize) { // Got all our data!
        // Send the payload to the PC
        PCCOMM::send_rx_data(this->channel_id, 0x0000, rxPayload.payload, rxPayload.payloadSize);
        // Now delete the old payload
        delete[] this->rxPayload.payload;
        this->isReceiving = false;
    }
}

void ISO15765Channel::send_ff_indication(CAN_FRAME *read, int id) {
    uint32_t request_id = read->id;
    // Send the flow control message back to the ECU
    // TODO OBIDE BY IOCTL BS AND MIN_ST
    f.id = this->flowcontrol_ids[id];
    if (f.id == 0) {
        char buf[45] = {0x00};
        sprintf(buf, "Error. CAN ID %04X has no response ID", request_id);
        PCCOMM::log_message(buf);
        return;
    }
    if (this->isReceiving) {
        // Error already trying to receive another payload!
        PCCOMM::log_message("Already trying to receive another ISO-15765 payload!?");
        return;
    }

    char buf[55] = {0x00};
    sprintf(buf, "Start of MF MSG. CID %04X. Expected size: %d", request_id, read->data.bytes[1]);
    PCCOMM::log_message(buf);
    // Now allocate memory for the buffer!
    this->rxPayload.payload = new char[read->data.bytes[1] + 4]; // +4 for CAN ID
    this->rxPayload.payloadSize = read->data.bytes[1] + 4;
    this->rxPayload.payloadPos = 10; // Always for first frame
    memcpy(&rxPayload.payload[4] ,&read->data.bytes[2], 6); // Copy the first 6 bytes (Start at 4 for CAN ID)
    this->rxPayload.payload[0] = request_id >> 24;
    this->rxPayload.payload[1] = request_id >> 16;
    this->rxPayload.payload[2] = request_id >> 8;
    this->rxPayload.payload[3] = request_id >> 0;
    this->isReceiving = true;


    // Now create the flow control frame to send back to the application
    f.length = 8;
    f.data.bytes[0] = 0x30;
    f.data.bytes[1] = 8; // BLOCK SIZE
    f.data.bytes[2] = 0x02; // ST_MIN
    Can0.sendFrame(f);
    // Send the first frame indication back to the user application
    // 4 additional bytes should be sent which represents the Can ID of the message
    char* buf2 = new char[4];
    // Copy CAN ID
    buf2[0] = request_id >> 24;
    buf2[1] = request_id >> 16;
    buf2[2] = request_id >> 8;
    buf2[3] = request_id >> 0;
    PCCOMM::send_rx_data(this->channel_id, ISO15765_FIRST_FRAME, buf2, 4);
    delete[] buf2;
}

void ISO15765Channel::sendMsg(uint32_t tx_flags, char* data, int data_size, bool respond) {
    if (this->isSending) {
        if (respond) {
            PCCOMM::respond_err(MSG_TX_CHAN_DATA, ERR_BUFFER_FULL, nullptr);
        } else {
            PCCOMM::log_message("Cannot send. IOS15765 is already busy");
        }
        return;
    }
    if (data_size <= 11) { // one frame!
        f.extended = this->isExtended;
        f.priority = 4; // Balanced priority
        f.length = 8;
        f.id = data[0] << 24 | data[1] << 16 | data[2] << 8 | data[3];
        f.rtr = false;
        f.data.bytes[0] = data_size - 4; // First byte is the length of the ISO message
        memcpy(&f.data.bytes[1], &data[4], data_size-4); // Copy data to bytes [1] and beyond
        if (!debug_send_frame(f)) {
            if (respond) {
                PCCOMM::respond_err(MSG_TX_CHAN_DATA, ERR_FAILED, "CAN Tx failed");
            } else {
                PCCOMM::log_message("Error sending ISO-TP frame. Canbus Tx failed");
            }
        } else {
            if (respond) {
                PCCOMM::respond_ok(MSG_TX_CHAN_DATA, nullptr, 0);
            }
        }
        if (respond) {
            
        }
    } else {
        // TODO Multi frame data write
        if (respond) {
            PCCOMM::respond_err(MSG_TX_CHAN_DATA, ERR_FAILED, "Multi frame CAN Not supported");
            f.extended = this->isExtended;
            f.priority = 4; // Balanced priority
            f.length = 8;
            f.id = data[0] << 24 | data[1] << 16 | data[2] << 8 | data[3];
            f.rtr = false;
            f.data.bytes[0] = 0x10;
            f.data.bytes[1] = data_size - 4; // First byte is the length of the ISO message
            memcpy(&f.data.bytes[2], &data[4], 6); // Copy data to bytes [1] and beyond
            this->clear_to_send = false;
            this->txPayload = isoPayload {
                // Just copy the data, ignore the CID
                new char[data_size-4],
                data_size - 4,
                6 // pos 6

            };
            memcpy(&txPayload.payload[0], &data[4], data_size-4); // Copy the rest of the payload to our temp buffer
            digitalWrite(DS7_GREEN, LOW);
            debug_send_frame(f);
            digitalWrite(DS7_GREEN, HIGH);
            if (respond) {
                PCCOMM::respond_ok(MSG_TX_CHAN_DATA, nullptr, 0);
            }
        }
    }
}