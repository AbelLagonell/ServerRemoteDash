#!/bin/bash

# Define the named pipes
PIPE="/tmp/metrics_pipe"

# Create the named pipe if it doesn't exist
[ ! -p "$PIPE" ] && mkfifo "$PIPE"

# Specify the log file for historical data
LOG_FILE="/tmp/system_metrics.log"

collect_metrics() {
    while true; do
        TIMESTAMP=$(date "+%Y-%m-%d %H:%M:%S")
        
        # Collect CPU, memory, IO, filesystem, load average
        CPU=$(top -bn1 | grep "Cpu(s)" | awk '{print $2 + $4}')
        MEM=$(free -m | awk 'NR==2{printf "%s/%s MB (%.2f%%)", $3, $2, $3*100/$2}')
        IO=$(iostat | awk 'NR==4 {print $1}')
        FS=$(df -h / | awk 'NR==2{print $5}')
        LOAD_AVG=$(cat /proc/loadavg | awk '{printf "%.2f", $1}')

        # Construct the metric string
        METRIC="{\"time\": \"$TIMESTAMP\", \"cpu\": \"$CPU%\", \"memory\": \"$MEM\", \"io\": \"$IO\", \"filesystem\": \"$FS\", \"load\": \"$LOAD_AVG\"}"

        # Output metrics to the screen
        echo "$METRIC"

        # Write the metrics to the pipe for other processes to read
        sudo echo "$METRIC" > "$PIPE" &

        # Write the metrics to the log file for history
        echo "$METRIC" >> "$LOG_FILE"

        sleep 5  # Change interval as needed for tests
    done
}

# Start the metric collection
collect_metrics
