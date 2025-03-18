import random
import socket
import time
import threading
import psutil
import argparse
import sys
import signal

# Global flag to indicate if the program should continue running
running = True

def signal_handler(sig, frame):
    """Handle interrupt signals to gracefully shut down the program."""
    global running
    print("\nShutdown signal received. Closing connections...")
    running = False
    sys.exit(0)

def main():
    # Set up signal handler for graceful termination
    signal.signal(signal.SIGINT, signal_handler)  # Ctrl+C
    signal.signal(signal.SIGTERM, signal_handler)  # Termination signal
    
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
    print("Press Ctrl+C to stop the client")
    
    # Continuously try to connect and send data
    global running
    while running:
        print(f"Attempting to connect to central server at {server_address[0]}:{server_address[1]}")
        try:
            # Create a socket connection
            sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            sock.settimeout(10)  # Set timeout for socket operations
            
            # Try to connect
            sock.connect(server_address)
            print("Connected to central server!")
            
            # Start sending monitoring data
            print("Starting to send monitoring data...")
            
            connection_active = True
            # Keep sending data until connection fails
            while running and connection_active:
                try:
                    # Get system metrics
                    metrics = collect_system_metrics(server_id)
                    
                    # Send each metric in a loop
                    for message in metrics:
                        try:
                            # Send the message
                            sock.sendall(message.encode())
                            print(f"Sent: {message.strip()}")
                            
                        except (socket.error, BrokenPipeError) as e:
                            print(f"Connection lost: {e}")
                            connection_active = False
                            break
                    
                    # Sleep before sending next batch of metrics
                    if connection_active:
                        time.sleep(1)
                        
                except Exception as e:
                    print(f"Error during metrics collection/sending: {e}")
                    connection_active = False
            
            print("Connection terminated. Cleaning up...")
            
        except socket.error as e:
            print(f"Failed to connect to central server: {e}")
        
        finally:
            # Always close the socket properly
            try:
                sock.shutdown(socket.SHUT_RDWR)
            except:
                pass  # Socket might already be closed
            
            try:
                sock.close()
            except:
                pass
                
            print("Socket closed.")
        
        # If we're still supposed to be running, wait before trying to reconnect
        if running:
            print(f"Waiting {reconnect_delay} seconds before reconnecting...")
            time.sleep(reconnect_delay)
        else:
            print("Client shutting down...")
            break

def collect_system_metrics(server_id):
    """Collect actual system metrics and return them as a list of formatted strings."""
    metrics = []
    current_time = get_formatted_time()
    
    # CPU utilization (type 0)
    try:
        cpu_percent = psutil.cpu_percent(interval=0.1)
        metrics.append(f"{server_id}-0-{cpu_percent:.1f}-{current_time}\n")
    except:
        # Fallback if CPU metrics collection fails
        metrics.append(f"{server_id}-0-0.0-{current_time}\n")
    
    # Memory utilization (type 4)
    try:
        memory = psutil.virtual_memory()
        memory_percent = memory.percent
        metrics.append(f"{server_id}-4-{memory_percent:.1f}-{current_time}\n")
    except:
        # Fallback if memory metrics collection fails
        metrics.append(f"{server_id}-4-0.0-{current_time}\n")
    
    # Disk usage (type 3)
    try:
        disk = psutil.disk_usage('/')
        disk_percent = disk.percent
        metrics.append(f"{server_id}-3-{disk_percent:.1f}-{current_time}\n")
    except:
        # Fallback if disk metrics collection fails
        metrics.append(f"{server_id}-3-0.0-{current_time}\n")
    
    # Network I/O (type 2)
    # Using a random value for demonstration
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
