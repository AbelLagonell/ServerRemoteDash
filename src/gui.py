import socket
import threading
import tkinter as tk
from tkinter import ttk
import matplotlib.pyplot as plt
from matplotlib.backends.backend_tkagg import FigureCanvasTkAgg
import datetime
import re
from collections import defaultdict
import queue
import matplotlib.dates as mdates

# Global data structure to store server metrics
# Format: {server_id: {metric_type: [(timestamp, value), ...], ...}, ...}
server_data = defaultdict(lambda: defaultdict(list))

# Lock for thread-safe access to server_data
data_lock = threading.Lock()

# Queue for communication between TCP server and GUI
message_queue = queue.Queue()

# Mapping for metric types
METRIC_TYPES = {0: "CPU", 1: "IP", 2: "Network", 3: "FS", 4: "Memory"}

class ServerMonitoringDashboard:
    def __init__(self, root):
        self.root = root
        self.root.title("Server Monitoring Dashboard")
        self.root.geometry("1200x800")
        
        # Main frame to contain all widgets
        self.main_frame = ttk.Frame(root)
        self.main_frame.pack(fill=tk.BOTH, expand=True)
        
        # Canvas for scrolling
        self.canvas = tk.Canvas(self.main_frame)
        self.scrollbar = ttk.Scrollbar(self.main_frame, orient=tk.VERTICAL, command=self.canvas.yview)
        self.scrollable_frame = ttk.Frame(self.canvas)
        
        self.scrollable_frame.bind(
            "<Configure>",
            lambda e: self.canvas.configure(scrollregion=self.canvas.bbox("all"))
        )
        
        self.canvas.create_window((0, 0), window=self.scrollable_frame, anchor="nw")
        self.canvas.configure(yscrollcommand=self.scrollbar.set)
        
        self.canvas.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        self.scrollbar.pack(side=tk.RIGHT, fill=tk.Y)
        
        # Dictionary to store server frames and charts
        self.server_frames = {}
        self.charts = {}
        
        # Create a label to show status
        self.status_label = ttk.Label(root, text="Server Status: Running - Listening on port 8888")
        self.status_label.pack(side=tk.BOTTOM, fill=tk.X, padx=5, pady=2)
        
        # Start the update loop to check for new messages
        self.check_queue()
        
        # Start the update loop for chart refreshing
        self.update_dashboard()
    
    def create_server_frame(self, server_id):
        """Create a new frame for a server that hasn't been seen before"""
        if server_id in self.server_frames:
            return
        
        print(f"Creating new frame for server {server_id}")
        
        # Create a frame for this server
        server_frame = ttk.LabelFrame(self.scrollable_frame, text=f"Server {server_id}")
        server_frame.pack(fill=tk.X, padx=10, pady=5, expand=True)
        
        # Store the frame
        self.server_frames[server_id] = server_frame
        
        # Create subplots for each metric type
        fig, axes = plt.subplots(1, 5, figsize=(15, 3), dpi=80)
        fig.tight_layout(pad=3.0)
        
        # Create charts for each metric type
        self.charts[server_id] = {}
        for i, metric_type_id in enumerate(range(5)):
            metric_name = METRIC_TYPES[metric_type_id]
            axes[i].set_title(metric_name)
            axes[i].set_ylim(0, 100)
            axes[i].set_xlabel('Time')
            axes[i].set_ylabel('Utilization %')
            axes[i].grid(True)
            
            # Store the axis for updates
            self.charts[server_id][metric_type_id] = axes[i]
        
        # Add the figure to the frame
        canvas = FigureCanvasTkAgg(fig, master=server_frame)
        canvas.draw()
        canvas.get_tk_widget().pack(fill=tk.X, expand=True)
        
        # Store canvas for updates
        self.charts[server_id]['canvas'] = canvas
        
        # Update the scrollregion to include new frame
        self.canvas.configure(scrollregion=self.canvas.bbox("all"))
    
    def check_queue(self):
        """Check the message queue for updates from TCP server"""
        try:
            # Process all available messages
            while not message_queue.empty():
                # Get server_id that has new data
                server_id = message_queue.get_nowait()
                
                # Ensure server frame exists
                self.create_server_frame(server_id)
                
                # Update this server's charts
                self.update_server_charts(server_id)
        except queue.Empty:
            pass
        except Exception as e:
            print(f"Error in check_queue: {e}")
        finally:
            # Schedule next check (every 100ms)
            self.root.after(100, self.check_queue)
    
    def update_server_charts(self, server_id):
        """Update charts for a specific server with the latest data"""
        try:
            with data_lock:
                # Create a copy of the data to work with
                if server_id in server_data:
                    metrics = {m: list(v) for m, v in server_data[server_id].items()}
                else:
                    return
            
            # Update each metric chart
            for metric_type_id in range(5):
                if metric_type_id in metrics and metrics[metric_type_id]:
                    # Get data points
                    time_strings = [t for t, _ in metrics[metric_type_id]]
                    values = [float(v) for _, v in metrics[metric_type_id]]
                    
                    # Clear the axis and replot
                    ax = self.charts[server_id][metric_type_id]
                    ax.clear()
                    
                    # Set title and labels
                    metric_name = METRIC_TYPES[metric_type_id]
                    ax.set_title(metric_name)
                    ax.set_ylim(0, 100)
                    ax.set_xlabel('Time')
                    ax.set_ylabel('Utilization %')
                    ax.grid(True)
                    
                    # Plot data using time strings as x labels
                    x_numeric = list(range(len(time_strings)))
                    ax.plot(x_numeric, values, 'b-')
                    
                    # Set x axis labels to time strings
                    if len(time_strings) > 10:
                        # If too many points, only show some labels
                        step = len(time_strings) // 5
                        ticks = x_numeric[::step]
                        labels = time_strings[::step]
                    else:
                        ticks = x_numeric
                        labels = time_strings
                        
                    ax.set_xticks(ticks)
                    ax.set_xticklabels(labels, rotation=45)
                    ax.set_yticks([0, 25, 50, 75, 100])
            
            # Redraw the canvas
            self.charts[server_id]['canvas'].draw()
            
        except Exception as e:
            print(f"Error updating charts for server {server_id}: {e}")
    
    def update_dashboard(self):
        """Update all dashboard charts periodically"""
        try:
            # Get all server IDs
            with data_lock:
                server_ids = list(server_data.keys())
            
            # Update charts for all servers
            for server_id in server_ids:
                if server_id not in self.server_frames:
                    self.create_server_frame(server_id)
                self.update_server_charts(server_id)
        except Exception as e:
            print(f"Error in update_dashboard: {e}")
        finally:
            # Schedule next full update (every 5 seconds)
            self.root.after(5000, self.update_dashboard)


def handle_client(client_socket, address):
    """Handle incoming TCP connection and parse messages"""
    print(f"Connection from {address} established")
    
    try:
        buffer = ""
        while True:
            # Receive data
            data = client_socket.recv(1024).decode('utf-8')
            if not data:
                break
            
            # Append to buffer and process complete messages
            buffer += data
            
            # Find complete messages (those ending with newline or the end of the buffer)
            lines = buffer.split('\n')
            
            # The last line might be incomplete, so keep it in the buffer
            if data.endswith('\n'):
                buffer = ""
                messages = lines
            else:
                buffer = lines[-1]
                messages = lines[:-1]
            
            # Process complete messages
            for message in messages:
                if message.strip():  # Skip empty messages
                    process_message(message.strip())
                
    except Exception as e:
        print(f"Error handling client {address}: {e}")
    finally:
        client_socket.close()
        print(f"Connection from {address} closed")


def process_message(message):
    """Process a single message from client"""
    # Pattern: server-type-percentage-hh:mm:ss
    pattern = r'(\d+)-(\d+)-([\d.]+)-(\d\d:\d\d:\d\d)'
    match = re.match(pattern, message)
    
    if match:
        server_id = match.group(1)
        metric_type = int(match.group(2))
        percentage = match.group(3)
        timestamp = match.group(4)
        
        # Store in our global data structure (with lock)
        with data_lock:
            # Keep only the most recent 30 data points
            server_data[server_id][metric_type].append((timestamp, percentage))
            if len(server_data[server_id][metric_type]) > 30:
                server_data[server_id][metric_type].pop(0)
        
        # Notify GUI thread about new data
        message_queue.put(server_id)
        
        print(f"Received: Server={server_id}, Metric={METRIC_TYPES.get(metric_type, metric_type)}, "
              f"Value={percentage}%, Time={timestamp}")
    else:
        print(f"Ignoring malformed message: {message}")


def start_tcp_server():
    """Start TCP server to listen for incoming metrics"""
    server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    
    try:
        server.bind(('0.0.0.0', 8888))
        server.listen(5)
        print("TCP Server listening on port 8888")
        
        while True:
            client_socket, address = server.accept()
            client_thread = threading.Thread(target=handle_client, args=(client_socket, address))
            client_thread.daemon = True
            client_thread.start()
    except Exception as e:
        print(f"Server error: {e}")
    finally:
        server.close()


if __name__ == "__main__":
    # Start TCP server in a separate thread
    server_thread = threading.Thread(target=start_tcp_server)
    server_thread.daemon = True
    server_thread.start()
    
    # Start the dashboard
    root = tk.Tk()
    app = ServerMonitoringDashboard(root)
    root.mainloop()
