#!/bin/bash

cargo build --release
sudo cp target/release/libm2_driver.so /usr/share/passthru/macchina.so