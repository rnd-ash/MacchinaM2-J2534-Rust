#ifndef CUSTOM_CAN_H_
#define CUSTOM_CAN_H_

#include "due_can.h"

namespace CustomCan {

    // Each mailbox has a rxMailbox of 8 frames
    #define MAX_RX_QUEUE 8
    struct rxQueue {
        volatile CAN_FRAME buffer[MAX_RX_QUEUE];
        uint8_t head;
        uint8_t tail;
    };

    // Functions that help with due_can library for the J2534 driver
    /**
     * Sets up the Can0 interface, and blocks all incomming traffic by default
     */
    void enableCanBus(int baud);

    void __delete_check_rx_ring(int i);
    void __create_check_rx_ring(int i);

    void __rx_queue_push_frame(rxQueue &r, CAN_FRAME &f);
    bool __rx_queue_pop_frame(rxQueue &r, CAN_FRAME &f);


    // Callback functions that are ran if a frame is sent to a mailbox within an interrupt
    // Only registered when a rxFilter is set for the mailbox
    void __callback_mb0(CAN_FRAME *f);
    void __callback_mb1(CAN_FRAME *f);
    void __callback_mb2(CAN_FRAME *f);
    void __callback_mb3(CAN_FRAME *f);
    void __callback_mb4(CAN_FRAME *f);
    void __callback_mb5(CAN_FRAME *f);
    void __callback_mb6(CAN_FRAME *f);

    /**
     * Disables the Can0 interface
     */
    void disableCanBus();
    void disableCanFilter(int id);
    void enableCanFilter(int id, uint32_t pattern, uint32_t mask, bool isExtended);
    bool sendFrame(CAN_FRAME *cf);
    bool receiveFrame(int mailbox_id, CAN_FRAME *f);
    void clearMailboxQueue(int mailbox_id);
}

#endif