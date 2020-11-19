#include "channel.h"

void respond_err(COMM_MSG *msg, uint8_t err, char* txt) {
    msg->msg_type = MSG_OPEN_CHANNEL;
    msg->arg_size = 1 + strlen(txt);
    msg->args[0] = err;
    memcpy(&msg->args[1], txt, strlen(txt));
    PCCOMM::send_message(msg);
}

void respond_ok(COMM_MSG *msg) {
    msg->arg_size = 0x01;
    msg->msg_type = MSG_OPEN_CHANNEL;
    msg->args[0] = STATUS_NOERROR;
    PCCOMM::send_message(msg);
}

void setup_channel(COMM_MSG* msg) {
    if (msg->msg_type != MSG_OPEN_CHANNEL) {
        respond_err(msg, ERR_FAILED, "This is NOT a open channel msg!");
    }
    if (msg->arg_size != 16) {
        respond_err(msg, ERR_FAILED, "Payload size for OpenChannel is incorrect");
    }
    unsigned int id;
    unsigned int protocol;
    unsigned int baud;
    unsigned int flags;
    memcpy(&id, &msg->args[0], 4);
    memcpy(&protocol, &msg->args[4], 4);
    memcpy(&baud, &msg->args[8], 4);
    memcpy(&flags, &msg->args[12], 4);
    switch (protocol)
    {
        case ISO15765:
        case CAN:
        case ISO9141:
        case ISO14230:
        case J1850PWM:
        case J1850VPW:
        case SCI_A_ENGINE:
        case SCI_B_ENGINE:
        case SCI_A_TRANS:
        case SCI_B_TRANS:
            respond_err(msg, ERR_NOT_SUPPORTED, "Protocol not implimented yet");
            break;
        default:
            respond_err(msg, ERR_INVALID_PROTOCOL_ID, "Unrecognised protocol");
            break;
    }
}


int CanChannelhandler::curr_baud = 0;