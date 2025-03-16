#!/bin/bash

# Configuration
SERVER_HOST="127.0.0.1"
SERVER_PORT="7800"
PIPE_PATH="/tmp/system_stats_pipe"
INTERVAL=1  # Update interval in seconds

# Check if named pipe exists, if not create it
if [ ! -p "$PIPE_PATH" ]; then
    mkfifo "$PIPE_PATH"
    chmod 640 "$PIPE_PATH"  # Only user and group can read/write
fi

# Function to collect system statistics
collect_stats() {
    # Get timestamp
    TIMESTAMP=$(date +"%Y-%m-%d %H:%M:%S")

    # CPU usage (as percentage)
    CPU_USAGE=$(top -bn1 | grep "Cpu(s)" | awk '{print $2 + $4}')

    # Memory usage (as percentage)
    MEM_USAGE=$(free | grep Mem | awk '{print $3/$2 * 100.0}')

    # Disk usage (as percentage)
    DISK_USAGE=$(df -h / | awk 'NR==2 {print $5}' | tr -d '%')

    # System load (1-minute average)
    SYSTEM_LOAD=$(cat /proc/loadavg | awk '{print $1}')

    # Network statistics - bytes received and transmitted
    NET_STATS=$(cat /proc/net/dev | grep -v "lo" | head -n 1 | awk '{print $2 "," $10}')
    NET_RX=$(echo $NET_STATS | cut -d',' -f1)
    NET_TX=$(echo $NET_STATS | cut -d',' -f2)

    # Format as pipe-separated values
    echo "$TIMESTAMP|$CPU_USAGE|$MEM_USAGE|$DISK_USAGE|$SYSTEM_LOAD|$NET_RX|$NET_TX"
}

# Function to send stats to the TCP server
send_stats_to_server() {
    # Collect statistics
    STATS=$(collect_stats)

    # Write to named pipe
    echo "$STATS" > "$PIPE_PATH"

    # Send directly to TCP server as well
    echo "$STATS" | nc -w 1 $SERVER_HOST $SERVER_PORT
}

# Trap to clean up the named pipe on exit
trap 'rm -f "$PIPE_PATH"' EXIT

# Main loop
echo "Starting system monitoring (sending to $SERVER_HOST:$SERVER_PORT)"
while true; do
    send_stats_to_server
    sleep $INTERVAL
done