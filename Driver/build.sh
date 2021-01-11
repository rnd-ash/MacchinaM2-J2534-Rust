#!/bin/bash

path="debug"

if [ $1 = "release" ]; then
    path="release"
    cargo build --release
else
    cargo build
fi

echo "Copying JSON to ~/.passthru/"
cp ../macchina.json ~/.passthru/macchina.so

#This is for macOS, uncomment out for Linux"
echo "Copying library to ~/.passthru/"
#cp target/${path}/libm2_driver.so ~/.passthru/macchina.so
cp target/${path}/libm2_driver.dylib ~/.passthru/macchina.dylib
echo "Driver install is complete. Happy car hacking!"
