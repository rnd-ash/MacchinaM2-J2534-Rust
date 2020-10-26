# Cross platform J2534-2 implimentation for Macchina M2 Under the dash

This is experimental code, since this appears to be the first J2534 driver that supports Linux/OSX 

# Repo overview

## Driver
Code to generate the following targets:
* J2534 DLL for windows
* J2534 so for Linux and OSX


## J2534Common
J2534 common library for [OpenVehicleDiag](https://github.com/rnd-ash/OpenVehicleDiag) and this driver


## M2UTD
This contains code that gets uploaded to the M2 UTD Module

## M2_LOOKBACK
**DO NOT USE** This is test code written for a secondary M2 UTD to emulate a bare bones ECU to verify protocols function as they should, by acting as a lookback for data sent
via a M2 UTD
