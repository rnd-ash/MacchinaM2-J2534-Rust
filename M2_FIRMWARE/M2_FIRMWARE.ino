#include "due_can.h"
#include <M2_12VIO.h>
#include "comm.h"


CAN_FRAME input;
M2_12VIO M2IO;

void setup() {
  SerialUSB.begin(500000); // 500 kbps 
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
  digitalWrite(DS7_GREEN, HIGH);
  digitalWrite(DS7_BLUE, HIGH);
  digitalWrite(DS7_RED, HIGH);
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


void loop() {
  if (PCCOMM::read_message(&msg)) {
    switch (msg.msg_type)
    {
    case 0xFF:
      PCCOMM::send_message(&msg); // Test Message type - Should just loop back response
      break;
    default:
      break;
    }
  }
}
 