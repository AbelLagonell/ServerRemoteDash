use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::{mpsc::{self, Receiver, Sender}, Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

enum ConnectionEvent {
    NewMessage(String, u8), // Message content and server_id
    Disconnected(u8),
}

// Global channel to receive server events
static MESSAGE_CHANNEL: Lazy<(
    Arc<Mutex<Sender<ConnectionEvent>>>,
    Arc<Mutex<Receiver<ConnectionEvent>>>
)> = Lazy::new(|| {
    let (tx, rx) = mpsc::channel();
    (Arc::new(Mutex::new(tx)), Arc::new(Mutex::new(rx)))
});

// File writer configuration
struct FileWriterConfig {
    enabled: bool,
    directory: String,
    file_prefix: String,
}

// Default configuration
static FILE_WRITER_CONFIG: Lazy<Mutex<FileWriterConfig>> = Lazy::new(|| {
    Mutex::new(FileWriterConfig {
        enabled: true,
        directory: "logs".to_string(),
        file_prefix: "tcpdata".to_string(),
    })
});

struct ServerMonitor {
    listener: TcpListener,
    connections: HashMap<u8, TcpStream>,
}

impl ServerMonitor {
    fn new(address: &str) -> io::Result<Self> {
        let listener = TcpListener::bind(address)?;
        listener.set_nonblocking(true)?;

        Ok(ServerMonitor {
            listener,
            connections: HashMap::new(),
        })
    }

    fn start(&mut self) {
        println!("TCP server started, listening on {}", self.listener.local_addr().unwrap());

        // Create log directory if it doesn't exist
        let config = FILE_WRITER_CONFIG.lock().unwrap();
        if config.enabled {
            if !Path::new(&config.directory).exists() {
                if let Err(e) = std::fs::create_dir_all(&config.directory) {
                    eprintln!("Failed to create log directory: {}", e);
                }
            }
        }
        drop(config);

        loop {
            // Accept new connections
            match self.listener.accept() {
                Ok((stream, addr)) => {
                    println!("New connection from: {}", addr);

                    // Set a unique ID for this connection
                    let server_id = self.connections.len() as u8;

                    // Set the stream to non-blocking
                    if let Err(e) = stream.set_nonblocking(true) {
                        println!("Failed to set stream to non-blocking: {}", e);
                        continue;
                    }

                    // Store the connection
                    self.connections.insert(server_id, stream.try_clone().unwrap());

                    // Spawn a thread to handle this connection
                    let sender = MESSAGE_CHANNEL.0.lock().unwrap().clone();
                    thread::spawn(move || {
                        handle_client(server_id, stream, sender);
                    });
                },
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // No new connections, just continue
                    thread::sleep(Duration::from_millis(100));
                },
                Err(e) => {
                    println!("Error accepting connection: {}", e);
                    break;
                }
            }
        }
    }
}

fn handle_client(server_id: u8, mut stream: TcpStream, sender: Sender<ConnectionEvent>) {
    let mut buffer = [0; 1024];

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                // Connection closed
                println!("Connection closed by client {}", server_id);
                let _ = sender.send(ConnectionEvent::Disconnected(server_id));
                break;
            },
            Ok(size) => {
                // Process received data
                if let Ok(message) = String::from_utf8(buffer[0..size].to_vec()) {
                    for line in message.lines() {
                        if !line.is_empty() {
                            let _ = sender.send(ConnectionEvent::NewMessage(line.to_string(), server_id));
                        }
                    }
                }
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // No data available, just continue
                thread::sleep(Duration::from_millis(100));
            },
            Err(e) => {
                println!("Error reading from client {}: {}", server_id, e);
                let _ = sender.send(ConnectionEvent::Disconnected(server_id));
                break;
            }
        }
    }
}

// Function to write message to a file
fn write_to_file(server_id: u8, message: &str) -> io::Result<()> {
    let config = FILE_WRITER_CONFIG.lock().unwrap();

    if !config.enabled {
        return Ok(());
    }

    // Create timestamp for filename
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    // Use a simple date format YYYYMMDD
    let datetime = chrono::Utc::now();
    let date = datetime.format("%Y%m%d").to_string();

    // Create the directory if it doesn't exist
    if !Path::new(&config.directory).exists() {
        std::fs::create_dir_all(&config.directory)?;
    }

    // Create filename with server ID and date
    let filename = format!("{}/{}_{}_server{}.log", config.directory, config.file_prefix, date, server_id);

    // Open file in append mode, create if doesn't exist
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(&filename)?;

    // Add timestamp to the message
    let timestamped_msg = format!("{}\n", message);

    // Write to file
    file.write_all(timestamped_msg.as_bytes())?;

    Ok(())
}

// Start a message processing thread that writes incoming messages to files
fn start_file_writer() -> thread::JoinHandle<()> {
    thread::spawn(move || {
        // Get the receiver from the global channel
        let receiver = MESSAGE_CHANNEL.1.clone();

        loop {
            // Try to get an event from the receiver
            let event = match receiver.lock() {
                Ok(rx) => match rx.recv() {
                    Ok(event) => event,
                    Err(_) => break, // Channel closed
                },
                Err(_) => {
                    println!("Failed to lock receiver");
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }
            };

            match event {
                ConnectionEvent::NewMessage(msg, server_id) => {
                    // Write the message to file
                    if let Err(e) = write_to_file(server_id, &msg) {
                        eprintln!("Error writing to file: {}", e);
                    } else {
                        // Optional: print confirmation that data was saved
                        println!("Message from server {} saved to file", server_id);
                    }
                },
                ConnectionEvent::Disconnected(id) => {
                    println!("Client {} disconnected", id);
                }
            }
        }
    })
}

// Configure file writing settings
pub fn configure_file_writer(enabled: bool, directory: &str, file_prefix: &str) {
    let mut config = FILE_WRITER_CONFIG.lock().unwrap();
    config.enabled = enabled;
    config.directory = directory.to_string();
    config.file_prefix = file_prefix.to_string();
}

// Initialize the server - returns server and file writer thread handles
pub fn initialize_server(address: &str) -> io::Result<(thread::JoinHandle<()>, thread::JoinHandle<()>)> {
    // Start the file writer thread
    let file_writer_handle = start_file_writer();

    // Create and start the server
    let mut server = ServerMonitor::new(address)?;
    let server_handle = thread::spawn(move || {
        server.start();
    });

    // Return both thread handles
    Ok((server_handle, file_writer_handle))
}

// Example main function
#[allow(dead_code)]
fn main() -> io::Result<()> {
    // Configure the file writer
    configure_file_writer(true, "tcp_logs", "data");

    // Initialize the server and file writer
    let (_server_handle, _file_writer_handle) = initialize_server("0.0.0.0:8888")?;

    println!("Server running. Press Ctrl+C to stop.");

    // Keep the main thread running
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}