# Cross platform J2534-2 implimentation for Macchina M2 Under the dash

This is experimental code, since this appears to be the first J2534 driver that supports Linux/OSX 

# Repo overview

## Driver
Code to generate the following targets:
* J2534 DLL for windows
* J2534 so for Linux and OSX

## J2534Common
J2534 common library for [OpenVehicleDiag](https://github.com/rnd-ash/OpenVehicleDiag) and this driver

## M2_FIRMWARE
This contains code that gets uploaded to the M2 Module

# Compiling and installing

## Driver
**Rust MUST be installed** [You can install it here](https://forge.rust-lang.org/infra/other-installation-methods.html)

### Windows
1. Create the directory `C:\Program Files (x86)\macchina\passthru\`
2. Open the driver folder
3. In macchina.reg file, replace the COM-PORT attribute value with whatever COM Port the M2 Unit shows up as
4. run build.bat

### Linux and OSX
1. Create the directory `~/.passthru/`
2. Open the driver folder
3. In macchina.json file, replace the COM-PORT attribute value with whatever COM Port the M2 Unit shows up as
4. Run build.sh

## M2 Firmware
**Arduino IDE Must be installed**

1. Open M2_FIRMWARE folder within Arduino IDE
2. Select the Macchina M2 SAM Board as a target, go [here](https://docs.macchina.cc/m2-docs/arduino) to learn how to install the build target
3. Upload the sketch to the M2 module
