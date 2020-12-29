#ifndef COMM_CHANNELS_H_
#define COMM_CHANNELS_H_

#include "comm.h"
#include "custom_can.h"

class Channel {
    public:
        virtual void ioctl(COMM_MSG *msg);
        virtual bool setup(int id, int protocol, int baud, int flags);
        virtual void addFilter(int type, int filter_id, char* mask, char* pattern, char* flowcontrol, int mask_len, int pattern_len, int flowcontrol_len);
        virtual void removeFilter(int id);
        virtual void sendMsg(uint32_t tx_flags, char* data, int data_size, bool respond);
        virtual void destroy();
        virtual void update();
    protected:
        int channel_id;
};

#define MAX_CAN_BUFFER_SIZE 16
struct CanRingBuffer {
    CAN_FRAME* buffer[MAX_CAN_BUFFER_SIZE];
    uint8_t head;
    uint8_t tail;
    uint8_t count;
};

class CanChannel : public Channel {
    public:
        void ioctl(COMM_MSG *msg);
        bool setup(int id, int protocol, int baud, int flags);
        void addFilter(int type, int filter_id, char* mask, char* pattern, char* flowcontrol, int mask_len, int pattern_len, int flowcontrol_len);
        void removeFilter(int id);
        void destroy();
        void on_frame_receive(CAN_FRAME *f);
        void sendMsg(uint32_t tx_flags, char* data, int data_size, bool respond);
        void update();
    private:
        bool isExtended = false;
        CAN_FRAME f;
        bool used_mailboxes[7] = {false};
        bool blocking_filters[7] = {false};
        bool masks[7] = {false};
        uint32_t patterns[7] = {0x00};
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
        void sendMsg(uint32_t tx_flags, char* data, int data_size, bool respond);
        void update();
    private:
        void tx_single_frame(CAN_FRAME *read);
        void tx_multi_frame(CAN_FRAME *read, int filter_id);
        void send_ff_indication(CAN_FRAME *read, int filter_id);
        CAN_FRAME f;
        bool used_mailboxes[7] = {false};
        uint32_t flowcontrol_ids[7] = {0x00};
        uint32_t mask_ids[7] = {0x00};
        uint32_t pattern_ids[7] = {0x00};
        bool isExtended;
        bool isSending = false;
        bool isReceiving = false;
        isoPayload rxPayload = {0x00}; // For receiving
        isoPayload txPayload = {0x00}; // For sending
        uint8_t block_size;
        uint8_t sep_time;
        unsigned long next_send_time;
        bool clear_to_send = false;
};

#endif