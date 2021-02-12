#include "comm_channels.h"

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
        if (this->isReceiving) {
            delete [] this->rxPayload.payload;
        }
        this->isReceiving = false;
        this->clear_to_send = false;
        PCCOMM::respond_ok(MSG_REM_CHAN_FILT, nullptr, 0);
    } else {
        PCCOMM::respond_err(MSG_REM_CHAN_FILT, ERR_INVALID_FILTER_ID, "Filter does not exist!");
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
                    rx_single_frame(&f);
                    break;
                case 0x10:
                    send_ff_indication(&f, i);
                    break;
                case 0x20:
                    rx_multi_frame(&f, i);
                    break;
                case 0x30:
                    handle_fc(&f, i);
                    break;
                default:
                    char buf[70];
                    sprintf(buf, "CAN ID %04X invalid ISO-TP PCI: %02X. Discarding frame", f.id, f.data.bytes[0]);
                    PCCOMM::log_message(buf);
                    break;
                }
            }
        }
    }
    if (isSending && clear_to_send) {
        if (millis() >= next_send_time) {
            tx_multi_frame();
            next_send_time = millis() + this->sep_time;
            if (this->tx_frames_sent >= this->block_size_tx) {
                this->clear_to_send = false; // Await flow control again
            }
        }
    }
}

void ISO15765Channel::handle_fc(CAN_FRAME *read, int id) {
    // Here, we honour the ECU's COM Parameters, not ones provided by the diag SW.
    // This is to purely ensure better compatibility!

    // Firstly, see if we are clear to send (0x30)...If it is wait (0x31), send first
    // frame again back to the ECU
    if (read->data.bytes[0] != 0x30) { // TODO Handle not clear to send
        PCCOMM::log_message("Flow Control is NOT 0x30!");
        return;
    }
    this->block_size_tx = read->data.bytes[1];
    this->sep_time_tx = read->data.bytes[2];
    this->clear_to_send = true;
    this->tx_frames_sent = 0;
    this->next_send_time = millis() + this->sep_time;
}

void ISO15765Channel::tx_multi_frame() {
    f.id = txPayload.payload[0] << 24 | txPayload.payload[1] << 16 | txPayload.payload[2] << 8 | txPayload.payload[3];
    f.length = 8;
    uint8_t max_cpy = min(7, txPayload.payloadSize - txPayload.payloadPos);
    f.data.bytes[0] = tx_pci;
    memcpy(&f.data.bytes[1], &txPayload.payload[txPayload.payloadPos], max_cpy);
    txPayload.payloadPos += max_cpy;
    debug_send_frame(f);
    tx_pci++;
    this->tx_frames_sent++;
    this->next_send_time = millis() + this->sep_time_tx;
    // Rollover
    if (tx_pci == 0x30) {
        tx_pci = 0x21;
    }
    if (txPayload.payloadPos >= txPayload.payloadSize) {
        this->clear_to_send = false;
        this->isSending = false;
        delete[] this->txPayload.payload;
    }
}


void ISO15765Channel::rx_single_frame(CAN_FRAME *read) {
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


void ISO15765Channel::rx_multi_frame(CAN_FRAME *read, int id) {
    if (!this->isReceiving) {
        PCCOMM::log_message("Multi frame message received but not start frame!?");
        return;
    }
    uint8_t max_copy = min(rxPayload.payloadSize - rxPayload.payloadPos, 7); // Up to 7 bytes
    memcpy(&rxPayload.payload[rxPayload.payloadPos] ,&read->data.bytes[1], max_copy);

    rxPayload.payloadPos += max_copy;
    this->rx_frame_count++;
    if (rxPayload.payloadPos >= rxPayload.payloadSize) { // Got all our data!
        // Send the payload to the PC
        PCCOMM::send_rx_data(this->channel_id, 0x0000, rxPayload.payload, rxPayload.payloadSize);
        // Now delete the old payload
        delete[] this->rxPayload.payload;
        this->isReceiving = false;
        return;
    }
    if (this->rx_frame_count >= 8) { // ECU hit block limit. Send flow control again!
        this->rx_frame_count = 0;
        // Just send flow control back to ECU
        f.id = this->flowcontrol_ids[id];
        // Now create the flow control frame to send back to the application
        f.length = 8;
        f.data.bytes[0] = 0x30;
        f.data.bytes[1] = 8; // BLOCK SIZE
        f.data.bytes[2] = 0x02; // ST_MIN
        debug_send_frame(f);
        // ECU should now continue sending data...
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
    // Now allocate memory for the buffer!
    int size = ((read->data.bytes[0] & 0x0F) << 8) | (read->data.bytes[1] + 4);
    //char buf[40];
    //sprintf(buf, "Allocating %d bytes", size);
    //PCCOMM::log_message(buf);
    this->rxPayload.payload = new char[size]; // +4 for CAN ID
    this->rxPayload.payloadSize = size;
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
    debug_send_frame(f);
    // Send the first frame indication back to the user application
    // 4 additional bytes should be sent which represents the Can ID of the message
    char* buf2 = new char[4];
    // Copy CAN ID
    buf2[0] = request_id >> 24;
    buf2[1] = request_id >> 16;
    buf2[2] = request_id >> 8;
    buf2[3] = request_id >> 0;
    PCCOMM::send_rx_data(this->channel_id, ISO15765_FIRST_FRAME, buf2, 4);
    this->rx_frame_count = 0;
    delete[] buf2;
}

void ISO15765Channel::sendMsg(uint32_t tx_flags, char* data, int data_size, bool respond) {
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
        if (this->isSending) {
            if (respond) {
                PCCOMM::respond_err(MSG_TX_CHAN_DATA, ERR_BUFFER_FULL, nullptr);
            } else {
                PCCOMM::log_message("Cannot send. IOS15765 is already busy");
            }
            return;
        }
        // TODO Multi frame data write
        f.extended = this->isExtended;
        f.priority = 4; // Balanced priority
        f.length = 8;
        f.id = data[0] << 24 | data[1] << 16 | data[2] << 8 | data[3];
        f.rtr = false;
        f.data.bytes[0] = 0x10 | ((data_size - 4) & 0x0F00) >> 8;
        f.data.bytes[1] = (data_size - 4) & 0xFF; // First byte is the length of the ISO message
        memcpy(&f.data.bytes[2], &data[4], 6); // Copy data to bytes [1] and beyond
        this->txPayload = isoPayload {
            // Just copy the data, ignore the CID
            new char[data_size],
            data_size,
            10 // pos 10 (6 + 4 for CID)
        };
        memcpy(&txPayload.payload[0], &data[0], data_size); // Copy the rest of the payload to our temp buffer
        
        // Set attributes for sending data
        this->clear_to_send = false;
        this->isSending = true;
        this->tx_pci = 0x21;
        debug_send_frame(f);
        if (respond) {
            PCCOMM::respond_ok(MSG_TX_CHAN_DATA, nullptr, 0);
        }
    }
}

void ISO15765Channel::ioctl_get(uint32_t id) {
    uint32_t tmp = 0;
    switch (id)
    {
    case ISO15765_STMIN:
        tmp = this->sep_time;
        PCCOMM::respond_ok(MSG_IOCTL_GET, (uint8_t*)(&tmp), 4);
        break;
    case ISO15765_BS:
        tmp = this->block_size;
        PCCOMM::respond_ok(MSG_IOCTL_GET, (uint8_t*)(&tmp), 4);
        break;
    default:
        PCCOMM::respond_err(MSG_IOCTL_GET, ERR_INVALID_IOCTL_ID, "ISO15765 invalid IOCTL ID");
        break;
    }
}

void ISO15765Channel::ioctl_set(uint32_t id, uint32_t value) {
    switch (id)
    {
    case ISO15765_STMIN:
        this->sep_time = value;
        PCCOMM::respond_ok(MSG_IOCTL_SET, nullptr, 0);
        break;
    case ISO15765_BS:
        this->block_size = value;
        PCCOMM::respond_ok(MSG_IOCTL_SET, nullptr, 0);
        break;
    default:
        PCCOMM::respond_err(MSG_IOCTL_SET, ERR_INVALID_IOCTL_ID, "ISO15765 invalid IOCTL ID");
        break;
    }
}