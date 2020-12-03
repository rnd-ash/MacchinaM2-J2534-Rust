#ifndef CHANNEL_H_
#define CHANNEL_H_

#include <Arduino.h>
#include "due_can.h"
#include "comm.h"
#include "j2534_mini.h"
#include "comm_channel.h"

#define MAX_CHANNELS 10
extern Channel* channels[MAX_CHANNELS];

void setup_channel(COMM_MSG* msg);
void remove_channel(COMM_MSG *msg);
void channel_loop();

/**
 * This function is ran when disconnect is called.
 * This removes all channels, returning the M2
 * back to its idle state
 */
void reset_all_channels();


// Based on Rx mailboxes
#define MAX_CAN_CHANNELS_EXT 3 // Rx boxes 0-3 are extended frames
#define MAX_CAN_CHANNELS_STD 4 // Rx boxes 3-7 are standard frames
namespace CanChannelHandler {
    Channel* create_channel(int id, int protocol, int baud, int flags);
    void resetCanInterface();
};

#endif
