#ifndef COMM_CHANNEL_H_
#define COMM_CHANNEL_H_

#include "j2534_mini.h"
#include <Arduino.h>

class Channel {
    public:
        Channel(int id);
        virtual void poll_for_message() = 0;
        virtual void send_message(uint8_t* msg, uint16_t msg_len) = 0;
        virtual void ioctl() = 0;
    protected:
        int id;
};

class ISO15765_Channel : public Channel {
    public:
        ISO15765_Channel(int id, int mailbox_id, bool isExtended);
        void poll_for_message();
        void send_message(uint8_t* msg, uint16_t msg_len);
        void ioctl();
    private:
        bool isExtended;
        int mailbox_id;
};

#endif