#!/bin/bash

# Function to check if stress-ng is installed
check_and_install_stress_ng() {
    if ! command -v stress-ng &> /dev/null; then
        echo "stress-ng not found, installing..."
        # Check if the system is using apt (Debian/Ubuntu) or yum (CentOS/RHEL)
        if command -v apt &> /dev/null; then
            sudo apt update && sudo apt install -y stress-ng
        elif command -v yum &> /dev/null; then
            sudo yum install -y stress-ng
        elif command -v dnf &> /dev/null; then
            sudo dnf install -y stress-ng
        else
            echo "Package manager not supported, please install stress-ng manually."
            exit 1
        fi
    else
        echo "stress-ng is already installed."
    fi
}

# Run the check and installation process
check_and_install_stress_ng

# Now run the stress-ng command
stress-ng --cpu 2 --fifo 2 --vm 2 --timeout 5s
