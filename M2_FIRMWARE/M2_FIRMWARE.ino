#include "due_can.h"
#include <M2_12VIO.h>
#include "comm.h"


CAN_FRAME input;
M2_12VIO M2IO;

void setup() {
  SerialUSB.begin(115200);
  while(!SerialUSB){}
  pinMode(DS2, OUTPUT); // Alive (RED) LED
  pinMode(DS3, OUTPUT); // Can Looped (Yellow)
  pinMode(DS4, OUTPUT);
  digitalWrite(DS2, LOW); // On
  digitalWrite(DS3, HIGH); // Off
  digitalWrite(DS4, HIGH);
  Can0.enable(); // Use a 500kbps bus speed
  //Can0.init(500_000);
  Can0.beginAutoSpeed();
  int filter;
  //extended
  for (filter = 0; filter < 3; filter++) {
    Can0.setRXFilter(filter, 0, 0, true);
  }  
  //standard
  for (int filter = 3; filter < 7; filter++) {
    Can0.setRXFilter(filter, 0, 0, false);
  } 
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

unsigned long lastVTime = 0;
bool lon = false;
void loop() {
    digitalWrite(DS3, HIGH); // Off
    if (Can0.available() > 0) {
        Can0.read(input);
        digitalWrite(DS3, LOW); // On
        delay(10);
    }
    if (millis() - lastVTime > 1000) {
        if(lon) {
            digitalWrite(DS4, LOW);
        } else {
            digitalWrite(DS4, HIGH);
        }
        lastVTime = millis();
        lon = !lon;
    }
}
