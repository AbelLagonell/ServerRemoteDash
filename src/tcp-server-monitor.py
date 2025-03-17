import socket
import threading
import os
import time
from datetime import datetime
from enum import Enum
import queue
from typing import Dict, Tuple

class ConnectionEvent(Enum):
    NEW_MESSAGE = 1  # Message content and server_id
    DISCONNECTED = 2

# Global queue to receive server events
message_queue = queue.Queue()

# File writer configuration
class FileWriterConfig:
    def __init__(self):
        self.enabled = True
        self.directory = "logs"
        self.file_prefix = "tcpdata"

# Default configuration
file_writer_config = FileWriterConfig()

class ServerMonitor:
    def __init__(self, address: str):
        host, port = address.split(":")
        self.listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        self.listener.bind((host, int(port)))
        self.listener.setblocking(False)
        self.connections: Dict[int, socket.socket] = {}

    def start(self):
        self.listener.listen(5)
        print(f"TCP server started, listening on {self.listener.getsockname()}")

        # Create log directory if it doesn't exist
        if file_writer_config.enabled:
            if not os.path.exists(file_writer_config.directory):
                try:
                    os.makedirs(file_writer_config.directory)
                except Exception as e:
                    print(f"Failed to create log directory: {e}")

        while True:
            try:
                # Accept new connections
                client_socket, addr = self.listener.accept()
                print(f"New connection from: {addr}")

                # Set a unique ID for this connection
                server_id = len(self.connections)

                # Set the socket to non-blocking
                client_socket.setblocking(False)

                # Store the connection
                self.connections[server_id] = client_socket

                # Spawn a thread to handle this connection
                client_thread = threading.Thread(
                    target=handle_client,
                    args=(server_id, client_socket),
                    daemon=True
                )
                client_thread.start()
            except BlockingIOError:
                # No new connections, just continue
                time.sleep(0.1)
            except Exception as e:
                print(f"Error accepting connection: {e}")
                break

def handle_client(server_id: int, client_socket: socket.socket):
    buffer_size = 1024
    
    while True:
        try:
            data = client_socket.recv(buffer_size)
            if not data:
                # Connection closed
                print(f"Connection closed by client {server_id}")
                message_queue.put((ConnectionEvent.DISCONNECTED, server_id, None))
                break
            
            # Process received data
            try:
                message = data.decode('utf-8')
                for line in message.splitlines():
                    if line:
                        message_queue.put((ConnectionEvent.NEW_MESSAGE, server_id, line))
            except UnicodeDecodeError:
                print(f"Received non-UTF-8 data from client {server_id}")
                
        except BlockingIOError:
            # No data available, just continue
            time.sleep(0.1)
        except Exception as e:
            print(f"Error reading from client {server_id}: {e}")
            message_queue.put((ConnectionEvent.DISCONNECTED, server_id, None))
            break

# Function to write message to a file
def write_to_file(server_id: int, message: str) -> bool:
    if not file_writer_config.enabled:
        return True

    # Create timestamp for filename
    now = int(time.time())

    # Use a simple date format YYYYMMDD
    date = datetime.now().strftime("%Y%m%d")

    # Create the directory if it doesn't exist
    if not os.path.exists(file_writer_config.directory):
        try:
            os.makedirs(file_writer_config.directory)
        except Exception as e:
            print(f"Failed to create log directory: {e}")
            return False

    # Create filename with server ID and date
    filename = f"{file_writer_config.directory}/{file_writer_config.file_prefix}_{date}_server{server_id}.log"

    try:
        # Open file in append mode, create if doesn't exist
        with open(filename, 'a') as file:
            # Add timestamp to the message
            timestamped_msg = f"{message}\n"
            
            # Write to file
            file.write(timestamped_msg)
        return True
    except Exception as e:
        print(f"Error writing to file: {e}")
        return False

# Start a message processing thread that writes incoming messages to files
def start_file_writer():
    def file_writer_thread():
        while True:
            try:
                # Get an event from the queue
                event_type, server_id, message = message_queue.get()
                
                if event_type == ConnectionEvent.NEW_MESSAGE:
                    # Write the message to file
                    if write_to_file(server_id, message):
                        # Optional: print confirmation that data was saved
                        print(f"Message from server {server_id} saved to file")
                elif event_type == ConnectionEvent.DISCONNECTED:
                    print(f"Client {server_id} disconnected")
                
                message_queue.task_done()
            except Exception as e:
                print(f"Error in file writer thread: {e}")
                time.sleep(0.1)
    
    file_writer = threading.Thread(target=file_writer_thread, daemon=True)
    file_writer.start()
    return file_writer

# Configure file writing settings
def configure_file_writer(enabled: bool, directory: str, file_prefix: str):
    file_writer_config.enabled = enabled
    file_writer_config.directory = directory
    file_writer_config.file_prefix = file_prefix

# Initialize the server - returns server and file writer thread handles
def initialize_server(address: str) -> Tuple[threading.Thread, threading.Thread]:
    # Start the file writer thread
    file_writer_handle = start_file_writer()

    # Create and start the server
    server = ServerMonitor(address)
    server_handle = threading.Thread(target=server.start, daemon=True)
    server_handle.start()

    # Return both thread handles
    return server_handle, file_writer_handle

# Example main function
def main():
    # Configure the file writer
    configure_file_writer(True, "tcp_logs", "data")

    # Initialize the server and file writer
    server_handle, file_writer_handle = initialize_server("0.0.0.0:8888")

    print("Server running. Press Ctrl+C to stop.")

    try:
        # Keep the main thread running
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        print("Server stopping...")

if __name__ == "__main__":
    main()
