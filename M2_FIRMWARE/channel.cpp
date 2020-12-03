#include "channel.h"

Channel* channels[MAX_CHANNELS] = {nullptr};

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
    if (channels[id] != nullptr) {
        return PCCOMM::respond_err(MSG_OPEN_CHANNEL, ERR_CHANNEL_IN_USE, "Channel in use");
    }

    switch (protocol)
    {
        case ISO15765: {
                Channel* chan = CanChannelHandler::create_channel(id, protocol, baud, flags);
                if (chan != nullptr) { // Only handle positive case, if its negative, the setup function already returned the result
                    channels[id] = chan;
                    PCCOMM::respond_ok(MSG_OPEN_CHANNEL, nullptr, 0);
                }
            }
            break;
        case CAN:
            CanChannelHandler::create_channel(id, protocol, baud, flags);
            break;
        case ISO9141:
        case ISO14230:
        case J1850PWM:
        case J1850VPW:
        case SCI_A_ENGINE:
        case SCI_B_ENGINE:
        case SCI_A_TRANS:
        case SCI_B_TRANS:
            PCCOMM::respond_err(MSG_OPEN_CHANNEL, ERR_NOT_SUPPORTED, "Protocol not implimented yet");
            break;
        default:
            char buf[35];
            sprintf(buf, "Unrecognised protocol 0x%02X", msg->args[0]);
            PCCOMM::respond_err(MSG_OPEN_CHANNEL, ERR_INVALID_PROTOCOL_ID, buf);
            break;
    }
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
    if (channels[id] == nullptr) {
        PCCOMM::respond_err(MSG_CLOSE_CHANNEL, ERR_INVALID_CHANNEL_ID, "Non existant channel");
    } else {
        channels[id]->remove();
        delete channels[id];
        channels[id] = nullptr;
        PCCOMM::respond_ok(MSG_CLOSE_CHANNEL, nullptr, 0);
    }
}

void reset_all_channels() {
    for (int i = 0; i < MAX_CHANNELS; i++) {
        if (channels[i] != nullptr) {
            channels[i]->remove();
            delete channels[i];
            channels[i] = nullptr;
        }
    }
    CanChannelHandler::resetCanInterface(); // Reset the CAN Iface on M2
}

void channel_loop() {
    for (int i = 0; i < MAX_CHANNELS; i++) {
        if (channels[i] != nullptr) {
            channels[i]->update();
        }
    }
}


namespace CanChannelHandler {
    bool mailboxes_in_use[7] = {0x00};
    int curr_baud = 0;

    int get_free_mailbox_id(bool is_ext) {
        int start_idx = 4;
        int end_idx = 7;
        if (is_ext) {
            start_idx = 0;
            end_idx = 3;
        }
        for (int i = start_idx; i <= end_idx; i++) {
            if (mailboxes_in_use[i] == false) { // Found a free mailbox!
                return i;
            }
        }
        return -1; // No mailbox found
    }


    Channel* create_channel(int id, int protocol, int baud, int flags) {
        // Current baud is already set, but new channel wants another baud speed
        // Not physically possible on the M2s-Hardware
        if (curr_baud != 0 && baud != curr_baud) {
            PCCOMM::respond_err(MSG_OPEN_CHANNEL, ERR_FAILED, "Cannot run multiple CAN baud speeds on 1 interface!");
            return nullptr;
        }
        // Fresh start - init a new CAN interface
        if (curr_baud == 0) {
            PCCOMM::log_message("Setting up CAN0 interface!");
            Can0.init(baud);
            // Also set all mailboxes to reject all frames, once a channel is set up
            // it then can set up its own Rx mailbox
        }
        int mailbox_id = -1;
        bool use_ext = false;
        if (flags & CAN_29BIT_ID > 0) { // Channel uses 29bit CAN IDs!
            mailbox_id = get_free_mailbox_id(true);
            use_ext = true;
        } else { // Channel uses standard 11bit CAN ID
            mailbox_id = get_free_mailbox_id(false);
        }

        // Out of Rx mailboxes!
        if (mailbox_id == -1) {
            PCCOMM::respond_err(MSG_OPEN_CHANNEL, ERR_FAILED, "No more free CAN mailboxes");
            return nullptr;
        }

        
        
        
        if (protocol == ISO15765) {
            // Create the channel, and set the mailbox as in use
            digitalWrite(DS5, LOW);
            mailboxes_in_use[mailbox_id] = true; // Mailbox is now in use
            return new ISO15765_Channel(id, mailbox_id, use_ext);
        } else if (protocol == CAN) {

        } else {
            PCCOMM::respond_err(MSG_OPEN_CHANNEL, ERR_FAILED, "CAN Channel request but protocolID did not match");
            return nullptr;
        }
        PCCOMM::respond_err(MSG_OPEN_CHANNEL, ERR_FAILED, "Not completed");
        return nullptr;
    }

    void resetCanInterface() {
        Can0.disable();
        curr_baud = 0;
        for (int i = 0; i < MAX_CAN_CHANNELS_EXT+MAX_CAN_CHANNELS_STD; i++) {
            mailboxes_in_use[i] = false;
        }
        digitalWrite(DS5, HIGH);
    }
}