#include "due_can.h"
#include <M2_12VIO.h>
#include "comm.h"

#define FW_TEST

CAN_FRAME input;
M2_12VIO M2IO;

void setup() {
  SerialUSB.begin(115200); // 500 kbps 
  pinMode(DS6, OUTPUT); // Green
  pinMode(DS5, OUTPUT); // Yellow
  pinMode(DS4, OUTPUT); // Yellow
  pinMode(DS3, OUTPUT); // Yellow
  pinMode(DS2, OUTPUT); // Red
  pinMode(DS7_GREEN, OUTPUT); // RGB (Green)
  pinMode(DS7_BLUE, OUTPUT);  // RGB (Blue)
  pinMode(DS7_RED, OUTPUT);   // RGB (Red)
  digitalWrite(DS2, LOW); // At startup assume no PC
  digitalWrite(DS6, HIGH);
  digitalWrite(DS5, HIGH);
  digitalWrite(DS4, HIGH);
  digitalWrite(DS3, HIGH);
  digitalWrite(DS7_GREEN, HIGH);
  digitalWrite(DS7_BLUE, HIGH);
  digitalWrite(DS7_RED, HIGH);
  set_status_led(0x00); // No connection on powerup
  M2IO.Init_12VIO();
}

#define MACCHINA_V4

// https://github.com/kenny-macchina/M2VoltageMonitor/blob/master/M2VoltageMonitor_V4/M2VoltageMonitor_V4.ino
float getVoltage() {
  float voltage=M2IO.Supply_Volts();
  voltage /= 1000;
  voltage=.1795*voltage*voltage-2.2321*voltage+14.596; //calibration curve determined with DSO, assumed good
  //additional correction for M2 V4
#ifdef MACCHINA_V4
  voltage=-.0168*voltage*voltage+1.003*voltage+1.3199; //calibration curve determined with DMM, assumed good (M2 V4 only!)
#endif
  return voltage;
}
COMM_MSG msg = {0x00};
CAN_FRAME f;

void send_v_batt(COMM_MSG *msg) {
  msg->arg_size = 4;
  msg->msg_type = MSG_READ_BATT;
  unsigned long v_batt = getVoltage() * 1000;
  memcpy(&msg->args[0], &v_batt, 4);
  PCCOMM::send_message(msg);
}

void set_status_led(uint8_t status) {
  if (status == 0x00) {
    digitalWrite(DS6, HIGH); // Green Off
    digitalWrite(DS2, LOW); // Red On
  } else {
    digitalWrite(DS6, LOW); // Green On
    digitalWrite(DS2, HIGH); // Red Off
  }
}

void loop() {
  if (PCCOMM::read_message(&msg)) {
    switch (msg.msg_type)
    {
#ifdef FW_TEST
    case MSG_TEST:
      PCCOMM::send_message(&msg); // Test Message type - Should just loop back response
      break;
#endif
    case MSG_STATUS:
      set_status_led(msg.args[0]);
      break;
    case MSG_READ_BATT:
      send_v_batt(&msg);
      break;
    default:
      break;
    }
  }
}
 