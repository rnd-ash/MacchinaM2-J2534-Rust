#ifndef COMM_CHANNEL_H_
#define COMM_CHANNEL_H_

#include "j2534_mini.h"
#include "due_can.h"
#include <Arduino.h>

class Channel {
    public:
        Channel(int id);
        virtual void send_message(uint8_t* msg, uint16_t msg_len) = 0;
        virtual void update();
        //virtual void set_filter();
        virtual void ioctl() = 0;
        virtual void remove() = 0;
    protected:
        int id;
};

class ISO15765_Channel : public Channel {
    public:
        ISO15765_Channel(int id, int mailbox_id, bool isExtended);
        void poll_for_message();
        void send_message(uint8_t* msg, uint16_t msg_len);
        void ioctl();
        void remove();
        void update();
    private:
        bool isExtended;
        int mailbox_id;
        CAN_FRAME* frame;
};

#endif