#include "channel.h"



Channel* canChannel = nullptr; // Channel for physical canbus link
Channel* klineChannel = nullptr; // Channel for physical kline line

void setup_channel(COMM_MSG* msg) {
    if (msg->msg_type != MSG_OPEN_CHANNEL) {
        PCCOMM::respond_err(MSG_OPEN_CHANNEL, ERR_FAILED, "This is NOT a open channel msg!");
    }
    if (msg->arg_size != 16) {
        char buf[65];
        sprintf(buf, "Payload size for OpenChannel is incorrect. Want 16, got %d", msg->arg_size);
        PCCOMM::respond_err(MSG_OPEN_CHANNEL, ERR_FAILED, buf);
    }
    unsigned int id;
    unsigned int protocol;
    unsigned int baud;
    unsigned int flags;
    memcpy(&id, &msg->args[0], 4);
    memcpy(&protocol, &msg->args[4], 4);
    memcpy(&baud, &msg->args[8], 4);
    memcpy(&flags, &msg->args[12], 4);
    switch (id)
    {
        case CAN_CHANNEL_ID:
            if (canChannel != nullptr) {
                PCCOMM::respond_err(MSG_OPEN_CHANNEL, ERR_CHANNEL_IN_USE, nullptr);
            } else {
                create_can_channel(id, protocol, baud, flags);
            }
            break;
        default:
            PCCOMM::respond_err(MSG_OPEN_CHANNEL, ERR_FAILED, "Protocol unsupported");
            break;
    }

}

void create_can_channel(int id, int protocol, int baud, int flags) {
    Channel *c = nullptr;
    if (protocol == ISO15765) { // ISO-TP
        c = new ISO15765Channel();
    } else { // Standard CAN
        c = new CanChannel();
    }
    if (!c->setup(id, protocol, baud, flags)) { // This function will return log the error to driver if any error
        delete c;
        return;
    }
    canChannel = c; // Creation ok!
    PCCOMM::respond_ok(MSG_OPEN_CHANNEL, nullptr, 0); // Tell driver CAN based channel is ready!
}

void remove_channel(COMM_MSG *msg) {
    if (msg->msg_type != MSG_CLOSE_CHANNEL) {
        PCCOMM::respond_err(MSG_CLOSE_CHANNEL, ERR_FAILED, "This is NOT a close channel msg!");
    }
    if (msg->arg_size != 4) {
        char buf[65];
        sprintf(buf, "Payload size for OpenChannel is incorrect. Want 4, got %d", msg->arg_size);
        PCCOMM::respond_err(MSG_CLOSE_CHANNEL, ERR_FAILED, buf);
    }
    unsigned int id;
    memcpy(&id, &msg->args[0], 4);
    switch(id) {
        case CAN_CHANNEL_ID:
            delete_channel(canChannel);
            break;
        default:
            PCCOMM::respond_err(MSG_CLOSE_CHANNEL, ERR_FAILED, "Protocol unsupported");
            break;
    }
}

void delete_channel(Channel*& ptr) {
    if (ptr != nullptr) {
        ptr->destroy();
        delete ptr;
        ptr = nullptr;
        PCCOMM::respond_ok(MSG_CLOSE_CHANNEL, nullptr, 0);
    } else {
        PCCOMM::respond_err(MSG_CLOSE_CHANNEL, ERR_INVALID_CHANNEL_ID, nullptr);
    }
}

void channel_loop() {
    if (canChannel != nullptr) {
        canChannel->update();
    }
    if (klineChannel != nullptr) {
        klineChannel->update();
    }
}

void reset_all_channels() {
     if (canChannel != nullptr) {
        canChannel->destroy();
        delete canChannel;
        canChannel = nullptr;
    }
    if (klineChannel != nullptr) {
        klineChannel->destroy();
        delete klineChannel;
        klineChannel = nullptr;
    }
}

void add_channel_filter(COMM_MSG* msg) {
    unsigned int channel_id;
    unsigned int filter_id;
    unsigned int filter_type;
    unsigned int mask_size;
    unsigned int pattern_size;
    unsigned int flowcontrol_size;
    memcpy(&channel_id, &msg->args[0], 4);
    memcpy(&filter_id, &msg->args[4], 4);
    memcpy(&filter_type, &msg->args[8], 4);
    memcpy(&mask_size, &msg->args[12], 4);
    memcpy(&pattern_size, &msg->args[16], 4);
    memcpy(&flowcontrol_size, &msg->args[20], 4);
    if (filter_type == FLOW_CONTROL_FILTER && flowcontrol_size == 0) {
        PCCOMM::respond_err(MSG_SET_CHAN_FILT, ERR_NULL_PARAMETER, "WTF. ISO15765 FC filter is null? Driver should have checked this!");
        return;
    }
    // Check if the channel is valid?
    if (channel_id != CAN_CHANNEL_ID && channel_id != KLINE_CHANNEL_ID) {
        PCCOMM::respond_err(MSG_SET_CHAN_FILT, ERR_INVALID_CHANNEL_ID, "Channel ID does not exist");
        return;
    }

    // Channel is valid - Create our arrays for filter messages

    // Mask
    char* mask = new char[mask_size];
    memcpy(&mask[0], &msg->args[24], mask_size);

    // Pattern
    char* pattern = new char[pattern_size];
    memcpy(&pattern[0], &msg->args[24+mask_size], pattern_size);

    // This is the only optional filter
    char* flowcontrol = nullptr;
    if (flowcontrol_size > 0) {
        flowcontrol = new char[flowcontrol_size];
        memcpy(&flowcontrol[0], &msg->args[24+mask_size+pattern_size], flowcontrol_size);
    }

    if (channel_id == CAN_CHANNEL_ID) {
        if (canChannel != nullptr) {
            canChannel->addFilter(filter_type, filter_id, mask, pattern, flowcontrol, mask_size, pattern_size, flowcontrol_size);
        } else {
            PCCOMM::respond_err(MSG_SET_CHAN_FILT, ERR_INVALID_CHANNEL_ID, nullptr);
        }
    } else if (channel_id == KLINE_CHANNEL_ID) {
        if (klineChannel != nullptr) {
             klineChannel->addFilter(filter_type, filter_id, mask, pattern, flowcontrol, mask_size, pattern_size, flowcontrol_size);
        } else {
             PCCOMM::respond_err(MSG_SET_CHAN_FILT, ERR_INVALID_CHANNEL_ID, nullptr);
        }
    }
    // Done with these arrays, hardware has applied them, destroy
    delete[] mask;
    delete[] pattern;
    if (flowcontrol != nullptr) {
        delete[] flowcontrol;
    }
}