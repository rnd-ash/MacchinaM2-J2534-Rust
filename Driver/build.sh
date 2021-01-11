#!/usr/bin/env bash

path="debug"

if [ $1 = "release" ]; then
    path="release"
    cargo build --release
else
    cargo build
fi


UNAME=$(UNAME)
if [ `UNAME` = Darwin ]; then
    echo "Copying JSON to ~/.passthru/"
    cp ../macchina.json ~/.passthru/macchina.json
    echo "Copying the driver to ~/.passthru/"
    cp target/${path}/libm2_driver.dylib ~/.passthru/macchina.dylib
    
else
    echo "Copying JSON to ~/.passthru/"
    cp ../macchina.json ~/.passthru/macchina.json
    echo "Copying the driver to ~/.passthru/"
    cp target/${path}/libm2_driver.so ~/.passthru/macchina.so
fi

echo "Driver install is complete. Happy car hacking!"


