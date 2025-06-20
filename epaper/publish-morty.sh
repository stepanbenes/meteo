#!/bin/bash
set -e

PI_IP=192.168.1.60 # morty's IP address
#TARGET=armv7-unknown-linux-gnueabihf

# build binary
#cargo build --target=armv7-unknown-linux-gnueabihf --release

# upload binary
#echo "updating epaper binary"
#scp -r ./target/$TARGET/release/epaper pi@$PI_IP:/home/pi/epaper

echo "updating assets"
scp -r ./src/. pi@$PI_IP:/home/pi/meteo-dashboard
scp ./run-meteo-dashboard.sh pi@$PI_IP:/home/pi/run-meteo-dashboard.sh
#echo "updating epaper script"
#scp -r ./src/meteo-dashboard.py pi@$PI_IP:/home/pi/epaper-python/examples/epd_7in5_V2_test.py

# execute binary
# echo -n "starting epaper service... "
# ssh pi@$PI_IP 'python3 /home/pi/epaper-python/examples/epd_7in5_V2_test.py'
# echo "done"
