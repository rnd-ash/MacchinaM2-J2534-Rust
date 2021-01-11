#!/usr/bin/env bash

path="release"

echo "Macchina M2 Driver Installer"

UNAME=$(uname)
if [ `UNAME` = Darwin ]; then
    echo "Building the cargo"
    cargo build --release
    echo "Copying JSON to ~/.passthru/"
    cp macchina.json ~/.passthru/macchina.json
    echo "Copying the driver to ~/.passthru/"
    cp target/${path}/libm2_driver.dylib ~/.passthru/macchina.so
    
else
    echo "Building the cargo"
    cargo build --release
    echo "Copying JSON to ~/.passthru/"
    cp macchina.json ~/.passthru/macchina.json
    echo "Copying the driver to ~/.passthru/"
    cp target/${path}/libm2_driver.so ~/.passthru/macchina.so
fi

echo "Driver install is complete. Happy car hacking!"


