#!/bin/bash

# check if a file argument is given
if [ -z "$1" ]; then
    echo "Usage: $0 <can_log_file>"
    exit 1
fi

LOGFILE="$1"

# check if file exists
if [ ! -f "$LOGFILE" ]; then
    echo "Error: File '$LOGFILE' not found!"
    exit 1
fi

# load vcan module if not already loaded
if ! lsmod | grep -q '^vcan'; then
    sudo modprobe vcan
fi

# create vcan0 if it doesn't exist
if ! ip link show vcan0 > /dev/null 2>&1; then
    sudo ip link add dev vcan0 type vcan
fi

# bring up vcan0
sudo ip link set up vcan0

# start playback
echo "Playing back CAN log '$LOGFILE' on vcan0..."
cat "$LOGFILE" | canplayer vcan0=can0
