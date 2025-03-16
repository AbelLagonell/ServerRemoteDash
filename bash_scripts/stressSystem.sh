#!/bin/bash

# Log file location
LOG_FILE="/var/log/system-stress.log"

# Function to log messages
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a $LOG_FILE
}

log "Starting system stress script"

# Define stress classes
STRESS_CLASSES=("cpu" "io" "vm" "hdd" "network")
CURRENT_CLASS=0

# Clean exit function
clean_exit() {
    log "Received termination signal. Stopping stress processes..."
    pkill -f stress-ng
    exit 0
}

# Set up trap for clean exit
trap clean_exit SIGTERM SIGINT

# Main loop
while true; do
    # Select current stress class
    CLASS=${STRESS_CLASSES[$CURRENT_CLASS]}

    # Determine random duration between 30-120 seconds
    DURATION=$((30 + RANDOM % 90))

    # Determine random intensity between 50-100%
    INTENSITY=$((50 + RANDOM % 50))

    log "Running $CLASS stress test for $DURATION seconds at $INTENSITY% intensity"

    # Run the appropriate stress test based on class
    case $CLASS in
        "cpu")
            # CPU stress test (using percentage of available CPUs)
            CPU_COUNT=$(nproc)
            STRESS_CPUS=$(( CPU_COUNT * INTENSITY / 100 ))
            [ $STRESS_CPUS -lt 1 ] && STRESS_CPUS=1
            stress-ng --cpu $STRESS_CPUS --cpu-method all --timeout ${DURATION}s &
            ;;
        "io")
            # IO stress test
            stress-ng --io $((1 + INTENSITY / 25)) --timeout ${DURATION}s &
            ;;
        "vm")
            # Memory stress test
            MEM_TOTAL=$(free -m | grep "Mem:" | awk '{print $2}')
            MEM_STRESS=$((MEM_TOTAL * INTENSITY / 200)) # Use up to half of memory
            stress-ng --vm 2 --vm-bytes ${MEM_STRESS}M --timeout ${DURATION}s &
            ;;
        "hdd")
            # Disk stress test
            stress-ng --hdd $((1 + INTENSITY / 25)) --hdd-bytes 1G --timeout ${DURATION}s &
            ;;
        "network")
            # Network stress using iperf or netcat if available
            if command -v iperf3 &> /dev/null; then
                # Start iperf server locally if not running
                if ! pgrep -f "iperf3 -s" > /dev/null; then
                    iperf3 -s &> /dev/null &
                fi
                # Connect to local server to generate traffic
                timeout ${DURATION}s iperf3 -c localhost -t $DURATION -b ${INTENSITY}M &
            else
                # Fallback to stress-ng network stressor
                stress-ng --sock 2 --timeout ${DURATION}s &
            fi
            ;;
    esac

    # Wait for the stress test to complete
    sleep $DURATION
    pkill -f stress-ng

    # Sleep for a short recovery period
    RECOVERY=$((10 + RANDOM % 20))
    log "Recovery period for $RECOVERY seconds"
    sleep $RECOVERY

    # Move to next stress class
    CURRENT_CLASS=$(( (CURRENT_CLASS + 1) % ${#STRESS_CLASSES[@]} ))
done