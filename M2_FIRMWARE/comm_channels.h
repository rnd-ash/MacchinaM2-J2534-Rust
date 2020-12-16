#ifndef COMM_CHANNELS_H_
#define COMM_CHANNELS_H_

#include "comm.h"
#include "due_can.h"

class Channel {
    public:
        virtual void ioctl(COMM_MSG *msg);
        virtual bool setup(int id, int protocol, int baud, int flags);
        virtual void addFilter(int type, int filter_id, char* mask, char* pattern, char* flowcontrol, int mask_len, int pattern_len, int flowcontrol_len);
        virtual void removeFilter(int id);
        virtual void sendMsg(char* data, int data_size);
        virtual void destroy();
        virtual void update();
    protected:
        int channel_id;
};

class CanChannel : public Channel {
    public:
        void ioctl(COMM_MSG *msg);
        bool setup(int id, int protocol, int baud, int flags);
        void addFilter(int type, int filter_id, char* mask, char* pattern, char* flowcontrol, int mask_len, int pattern_len, int flowcontrol_len);
        void removeFilter(int id);
        void destroy();
        void sendMsg(char* data, int data_size);
        void update();
    private:
        bool isExtended = false;
        CAN_FRAME f;
        bool used_mailboxes[7] = {false};
        bool blocking_filters[7] = {false};
        bool blocking_filter_masks[7] = {false};
        uint32_t blocking_ids[7] = {0x00};
};

struct isoPayload {
    char* payload;
    int payloadSize;
    int payloadPos;
};

class ISO15765Channel : public Channel {
    public:
        void ioctl(COMM_MSG *msg);
        bool setup(int id, int protocol, int baud, int flags);
        void addFilter(int type, int filter_id, char* mask, char* pattern, char* flowcontrol, int mask_len, int pattern_len, int flowcontrol_len);
        void removeFilter(int id);
        void destroy();
        void sendMsg(char* data, int data_size);
        void update();
    private:
        bool used_mailboxes[7] = {false};
        uint32_t flowcontrol_ids[7] = {0x00};
        bool isExtended;
        bool isSending = false;
        bool isReceiving = false;
        isoPayload rxPayload = {0x00}; // For receiving
        isoPayload txPayload = {0x00}; // For sending

};

#endif