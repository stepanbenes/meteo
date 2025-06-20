#!/bin/bash
set -e

PI_IP=192.168.1.60 # morty's IP address

ssh pi@$PI_IP 'sudo journalctl -u meteo-dashboard.service -f'