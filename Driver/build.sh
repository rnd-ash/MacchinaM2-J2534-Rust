#!/bin/bash

cargo build --release
sudo cp ../macchina.json ~/.passthru/macchina.so
sudo cp target/release/libm2_driver.so ~/.passthru/macchina.so