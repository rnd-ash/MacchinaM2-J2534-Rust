#ifndef J2534_MINI_H_
#define J2534_MINI_H_

// Just stores relivant J2534 values to channels and IOCTL commands

// Protocols
#define	J1850VPW	 0x01 // J1850 VPW protocol
#define	J1850PWM	 0x02 // K1850 PWM protocol
#define	ISO9141		 0x03 // IOS9141 protocol (Uses K-Line)
#define	ISO14230	 0x04 // ISO14230 protocol (Uses K-Line)
#define	CAN			 0x05 // CAN protocol (Uses CAN-D)
#define	ISO15765	 0x06 // ISO15765 protocol (Uses CAN-D)
#define	SCI_A_ENGINE 0x07 // SCI_A_ENGINE TODO - Not supported ATM
#define	SCI_A_TRANS	 0x08 // SCI_A_TRANS  TODO - Not supported ATM
#define	SCI_B_ENGINE 0x09 // SCI_B_ENGINE TODO - Not supported ATM
#define	SCI_B_TRANS	 0x0A // SCI_B_TRANS  TODO - Not supported ATM

// Error definitions
#define		STATUS_NOERROR			  0x00	// Function completed successfully.
#define		ERR_NOT_SUPPORTED		  0x01	// Function option is not supported.
#define		ERR_INVALID_CHANNEL_ID    0x02	// Channel Identifier or handle is not recognized.
#define		ERR_INVALID_PROTOCOL_ID	  0x03	// Protocol Identifier is not recognized.
#define		ERR_NULL_PARAMETER		  0x04	// NULL pointer presented as a function parameter, NULL is an invalid address.
#define		ERR_INVALID_IOCTL_VALUE	  0x05	// Ioctl GET_CONFIG/SET_CONFIG parameter value is not recognized.
#define		ERR_INVALID_FLAGS		  0x06	// Flags bit field(s) contain(s) an invalid value.
#define		ERR_FAILED			      0x07	// Unspecified error, use PassThruGetLastError for obtaining error text string.
#define		ERR_DEVICE_NOT_CONNECTED  0x08	// PassThru device is not connected to the PC.
#define		ERR_TIMEOUT			      0x09	// Timeout violation. PassThru device is unable to read specified number of messages from the vehicle network. The actual number of messages returned is in NumMsgs.
#define		ERR_INVALID_MSG			  0x0A	// Message contained a min/max length, ExtraData support or J1850PWM specific source address conflict violation.
#define		ERR_INVALID_TIME_INTERVAL 0x0B	// The time interval value is outside the specified range.
#define		ERR_EXCEEDED_LIMIT		  0x0C	// The limit(ten) of filter/periodic messages has been exceeded for the protocol associated the communications channel.
#define		ERR_INVALID_MSG_ID		  0x0D	// The message identifier or handle is not recognized.
#define		ERR_DEVICE_IN_USE		  0x0E	// The specified PassThru device is already in use.
#define		ERR_INVALID_IOCTL_ID	  0x0F	// Ioctl identifier is not recognized.
#define		ERR_BUFFER_EMPTY		  0x10	// PassThru device could not read any messages from the vehicle network.
#define		ERR_BUFFER_FULL			  0x11	// PassThru device could not queue any more transmit messages destined for the vehicle network.
#define		ERR_BUFFER_OVERFLOW		  0x12	// PassThru device experienced a buffer overflow and receive messages were lost.
#define		ERR_PIN_INVALID			  0x13	// Unknown pin number specified for the J1962 connector.
#define		ERR_CHANNEL_IN_USE		  0x14	// An existing communications channel is currently using the specified network protocol.
#define		ERR_MSG_PROTOCOL_ID		  0x15	// The specified protocol type within the message structure is different from the protocol associated with the communications channel when it was opened.
#define		ERR_INVALID_FILTER_ID	  0x16	// Filter identifier is not recognized.
#define		ERR_NO_FLOW_CONTROL		  0x17	// No ISO15765 flow control filter is set, or no filter matches the header of an outgoing message.
#define		ERR_NOT_UNIQUE			  0x18	// An existing filter already matches this header or node identifier.
#define		ERR_INVALID_BAUDRATE	  0x19	// Unable to honor requested Baud rate within required tolerances.
#define		ERR_INVALID_DEVICE_ID	  0x1A	// PassThru device identifier is not recognized.

// Some useful flags (Channel creation)
#define		CAN_29BIT_ID		0x00000100
#define		ISO9141_NO_CHECKSUM	0x00000200
#define		CAN_ID_BOTH		    0x00000800
#define		ISO9141_K_LINE_ONLY	0x00001000

// Filter type
#define PASS_FILTER         0x01
#define BLOCK_FILTER        0x02
#define FLOW_CONTROL_FILTER 0x03

#endif