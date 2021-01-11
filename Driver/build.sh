#!/bin/bash

path="debug"

if [ $1 = "release" ]; then
    path="release"
    cargo build --release
else
    cargo build
fi

echo "Copying JSON to ~/.passthru/"
cp macchina.json ~/.passthru/macchina.json

echo "Copying library to ~/.passthru/"
cp target/${path}/libm2_driver.so ~/.passthru/macchina.so

echo "Complete"
