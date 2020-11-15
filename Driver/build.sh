#!/bin/bash

cargo build --release
cp ../macchina.json ~/.passthru/macchina.so
cp target/release/libm2_driver.so ~/.passthru/macchina.so