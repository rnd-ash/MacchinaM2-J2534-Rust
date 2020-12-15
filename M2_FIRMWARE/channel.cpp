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