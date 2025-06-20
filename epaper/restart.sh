#!/bin/bash
set -e

PI_IP=192.168.1.60 # morty's IP address

echo "Restarting Meteo Dashboard service..."
ssh pi@$PI_IP 'sudo systemctl restart meteo-dashboard.service'