import random
import socket
import time
import threading

def main():
    # Configuration
    server_address = ("127.0.0.1", 8888)  # Change to your central server address
    server_id = 0  # Change this for each cloud server instance
    reconnect_delay = 5  # seconds
    
    print(f"Cloud server {server_id} starting up...")
    
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
                    # Generate random monitoring data
                    message = generate_random_monitoring_data(server_id)
                    
                    try:
                        # Send the message
                        sock.sendall(message.encode())
                        print(f"Sent: {message.strip()}")
                        
                    except socket.error as e:
                        print(f"Error sending data: {e}")
                        break
                    
                    # Sleep before sending next message
                    time.sleep(1)
                    
                print("Lost connection to central server. Will try to reconnect...")
                
        except socket.error as e:
            print(f"Failed to connect to central server: {e}")
        
        # Wait before trying to reconnect
        print(f"Waiting {reconnect_delay} seconds before reconnecting...")
        time.sleep(reconnect_delay)

def generate_random_monitoring_data(server_id):
    # Generate random values
    metric_type = random.randint(0, 4)  # 0=CPU, 1=IP, 2=Network, 3=FS, 4=Memory
    utilization = random.uniform(0.0, 100.0)
    
    # Get current time
    now = time.time()
    total_seconds = int(now) % (24 * 3600)
    hours = total_seconds // 3600
    minutes = (total_seconds % 3600) // 60
    seconds = total_seconds % 60
    time_str = f"{hours:02d}:{minutes:02d}:{seconds:02d}"
    
    # Format the message according to the specified format
    return f"{server_id}-{metric_type}-{utilization:.1f}-{time_str}\n"

if __name__ == "__main__":
    main()
