#include <Arduino.h>
#include "due_can.h"
#include "comm.h"
#include "j2534_mini.h"

void respond_err(COMM_MSG *msg, uint8_t err, char* txt);
void respond_ok(COMM_MSG *msg);
void setup_channel(COMM_MSG* msg);

class CanChannel {
    public:
        CanChannel(int id, int flags);
};

#define MAX_CAN_CHANNELS 3
#define MAX_CAN_CHANNELS_EXT 4

class CanChannelhandler {
    public:
        void new_channel(int id, int baud, int flags);
    private:
        static int curr_baud;
        int channels_active_count = 0;
        int channels_active_ext_count = 0;
        CanChannel* active_channels[MAX_CAN_CHANNELS] = {nullptr};
        CanChannel* active_channels_ext[MAX_CAN_CHANNELS_EXT] = {nullptr};
};