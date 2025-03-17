import random
import socket
import time
import threading
import psutil
import argparse
import sys

def main():
    # Parse command-line arguments
    parser = argparse.ArgumentParser(description="Cloud Server Monitoring Client")
    parser.add_argument("--ip", help="IP address of the central server", default="127.0.0.1")
    parser.add_argument("--port", help="Port of the central server", type=int, default=8888)
    parser.add_argument("--server-id", help="ID for this server instance", type=int, default=0)
    args = parser.parse_args()
    
    # Configuration
    server_address = (args.ip, args.port)
    server_id = args.server_id
    reconnect_delay = 5  # seconds
    
    print(f"Cloud server {server_id} starting up...")
    print(f"Target central server: {server_address[0]}:{server_address[1]}")
    
    # Continuously try to connect and send data
    while True:
        print(f"Attempting to connect to central server at {server_address[0]}:{server_address[1]}")
        try:
            # Create a socket connection
            with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
                sock.connect(server_address)
                print("Connected to central server!")
                
                # Start sending monitoring data
                print("Starting to send monitoring data...")
                
                # Keep sending data until connection fails
                while True:
                    # Get system metrics
                    metrics = collect_system_metrics(server_id)
                    
                    # Send each metric in a loop
                    for message in metrics:
                        try:
                            # Send the message
                            sock.sendall(message.encode())
                            print(f"Sent: {message.strip()}")
                            
                        except socket.error as e:
                            print(f"Error sending data: {e}")
                            break
                    
                    # Sleep before sending next batch of metrics
                    time.sleep(1)
                    
                print("Lost connection to central server. Will try to reconnect...")
                
        except socket.error as e:
            print(f"Failed to connect to central server: {e}")
        
        # Wait before trying to reconnect
        print(f"Waiting {reconnect_delay} seconds before reconnecting...")
        time.sleep(reconnect_delay)

def collect_system_metrics(server_id):
    """Collect actual system metrics and return them as a list of formatted strings."""
    metrics = []
    current_time = get_formatted_time()
    
    # CPU utilization (type 0)
    cpu_percent = psutil.cpu_percent(interval=0.1)
    metrics.append(f"{server_id}-0-{cpu_percent:.1f}-{current_time}\n")
    
    # Memory utilization (type 4)
    memory = psutil.virtual_memory()
    memory_percent = memory.percent
    metrics.append(f"{server_id}-4-{memory_percent:.1f}-{current_time}\n")
    
    # Disk usage (type 3)
    disk = psutil.disk_usage('/')
    disk_percent = disk.percent
    metrics.append(f"{server_id}-3-{disk_percent:.1f}-{current_time}\n")
    
    # Network I/O (type 2)
    # This is a simplified metric - for demonstration, we use a random value
    network_percent = random.uniform(0, 100)
    metrics.append(f"{server_id}-2-{network_percent:.1f}-{current_time}\n")
    
    # IP performance (type 1)
    # Using a random value for demonstration
    ip_percent = random.uniform(0, 100)
    metrics.append(f"{server_id}-1-{ip_percent:.1f}-{current_time}\n")
    
    return metrics

def get_formatted_time():
    """Get current time formatted as HH:MM:SS."""
    now = time.time()
    total_seconds = int(now) % (24 * 3600)
    hours = total_seconds // 3600
    minutes = (total_seconds % 3600) // 60
    seconds = total_seconds % 60
    return f"{hours:02d}:{minutes:02d}:{seconds:02d}"

if __name__ == "__main__":
    main()
