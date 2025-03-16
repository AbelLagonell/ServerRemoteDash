// SERVER CODE (server.rs)
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio::time::{self, Duration};
use tokio::fs::File;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs::File;
use std::io::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::path::Path;

// Structure to hold system statistics
#[derive(Clone, Debug)]
struct SystemStats {
	timestamp: String,
	cpu_usage: f64,
	memory_usage: f64,
	disk_usage: f64,
	system_load: f64,
	network_rx: u64,
	network_tx: u64,
}

impl Default for SystemStats {
	fn default() -> Self {
		Self {
			timestamp: String::new(),
			cpu_usage: 0.0,
			memory_usage: 0.0,
			disk_usage: 0.0,
			system_load: 0.0,
			network_rx: 0,
			network_tx: 0,
		}
	}
}

// Structure to hold client information
struct ClientInfo {
	id: String,
	last_seen: u64,
	stats: SystemStats,
}

// Named pipe for local system monitoring integration
const PIPE_PATH: &str = "/tmp/system_stats_pipe";

#[tokio::main]
async fn main() {
	// Initialize logger
	env_logger::builder().filter_level(log::LevelFilter::Info).init();

	// Create a shared state for connected clients
	let clients = Arc::new(Mutex::new(HashMap::<String, ClientInfo>::new()));

	// Create a broadcast channel for system stats
	let (tx, _) = broadcast::channel::<(String, SystemStats)>(100);

	// Create a flag for graceful shutdown
	let running = Arc::new(AtomicBool::new(true));
	let r = running.clone();

	// Set up signal handler for graceful shutdown
	tokio::spawn(async move {
		let _ = tokio::signal::ctrl_c().await;
		log::info!("Shutdown signal received");
		r.store(false, Ordering::SeqCst);
	});

	// Create named pipe if it doesn't exist
	if !Path::new(PIPE_PATH).exists() {
		if let Err(e) = tokio::fs::File::create(PIPE_PATH).await {
			log::error!("Failed to create named pipe: {}", e);
		} else {
			log::info!("Created named pipe at {}", PIPE_PATH);
		}
	}

	// Start a task to read from the named pipe
	let pipe_tx = tx.clone();
	let pipe_clients = Arc::clone(&clients);
	tokio::spawn(async move {
		read_from_named_pipe(pipe_tx, pipe_clients).await;
	});

	// Start the cleanup task for inactive clients
	let cleanup_clients = Arc::clone(&clients);
	tokio::spawn(async move {
		let mut interval = time::interval(Duration::from_secs(60));
		while running.load(Ordering::SeqCst) {
			interval.tick().await;
			cleanup_inactive_clients(cleanup_clients.clone()).await;
		}
	});

	// Start TCP server
	let listener = match TcpListener::bind("0.0.0.0:7800").await {
		Ok(l) => {
			log::info!("Server listening on 0.0.0.0:7800");
			l
		},
		Err(e) => {
			log::error!("Failed to bind to address: {}", e);
			return;
		}
	};

	// Accept connections
	loop {
		if !running.load(Ordering::SeqCst) {
			break;
		}

		let accept_result = tokio::select! {
            result = listener.accept() => result,
            _ = time::sleep(Duration::from_millis(100)) => continue,
        };

		match accept_result {
			Ok((socket, addr)) => {
				log::info!("New connection from: {}", addr);

				// Clone the shared state for this connection
				let client_map = Arc::clone(&clients);
				let tx_clone = tx.clone();
				let client_running = running.clone();

				// Spawn a task to handle this connection
				tokio::spawn(async move {
					handle_connection(socket, addr.to_string(), client_map, tx_clone, client_running).await;
				});
			}
			Err(e) => {
				log::error!("Error accepting connection: {}", e);
			}
		}
	}

	log::info!("Server shutting down gracefully");
}

async fn read_from_named_pipe(
	tx: broadcast::Sender<(String, SystemStats)>,
	clients: Arc<Mutex<HashMap<String, ClientInfo>>>
) {
	loop {
		// Open the named pipe for reading
		match tokio::fs::File::open(PIPE_PATH).await {
			Ok(file) => {
				let mut reader = BufReader::new(file);
				let mut line = String::new();

				// Read lines from the pipe
				while let Ok(n) = reader.read_line(&mut line).await {
					if n == 0 {
						break; // End of file
					}

					if let Some(stats) = parse_stats_line(&line) {
						// Update client stats for "local" client
						{
							let mut clients_map = clients.lock().unwrap();
							let client_id = "local".to_string();

							clients_map.insert(client_id.clone(), ClientInfo {
								id: client_id.clone(),
								last_seen: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
								stats: stats.clone(),
							});
						}

						// Broadcast stats to other clients
						let _ = tx.send(("local".to_string(), stats));
					}

					line.clear();
				}
			}
			Err(e) => {
				log::error!("Failed to open named pipe: {}", e);
				time::sleep(Duration::from_secs(5)).await;
			}
		}
	}
}

async fn cleanup_inactive_clients(clients: Arc<Mutex<HashMap<String, ClientInfo>>>) {
	let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
	let mut to_remove = Vec::new();

	// Find inactive clients (last seen > 120 seconds ago)
	{
		let clients_map = clients.lock().unwrap();
		for (id, client) in clients_map.iter() {
			if now - client.last_seen > 120 {
				to_remove.push(id.clone());
			}
		}
	}

	// Remove inactive clients
	if !to_remove.is_empty() {
		let mut clients_map = clients.lock().unwrap();
		for id in to_remove {
			log::info!("Removing inactive client: {}", id);
			clients_map.remove(&id);
		}
	}
}

fn parse_stats_line(line: &str) -> Option<SystemStats> {
	let line = line.trim();
	if !line.contains('|') {
		return None;
	}

	let parts: Vec<&str> = line.split('|').collect();
	if parts.len() < 7 {
		return None;
	}

	Some(SystemStats {
		timestamp: parts[0].to_string(),
		cpu_usage: parts[1].parse().unwrap_or(0.0),
		memory_usage: parts[2].parse().unwrap_or(0.0),
		disk_usage: parts[3].parse().unwrap_or(0.0),
		system_load: parts[4].parse().unwrap_or(0.0),
		network_rx: parts[5].parse().unwrap_or(0),
		network_tx: parts[6].parse().unwrap_or(0),
	})
}

async fn handle_connection(
	socket: TcpStream,
	client_id: String,
	clients: Arc<Mutex<HashMap<String, ClientInfo>>>,
	tx: broadcast::Sender<(String, SystemStats)>,
	running: Arc<AtomicBool>,
) {
	let (reader, mut writer) = socket.into_split();
	let mut reader = BufReader::new(reader);
	let mut line = String::new();

	// Add client to connected clients
	{
		let mut clients_map = clients.lock().unwrap();
		clients_map.insert(client_id.clone(), ClientInfo {
			id: client_id.clone(),
			last_seen: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
			stats: SystemStats::default(),
		});
	}

	// Subscribe to broadcast channel
	let mut rx = tx.subscribe();

	// Spawn a task to send broadcast messages to this client
	let client_id_for_writer = client_id.clone();
	let writer_task = tokio::spawn(async move {
		let mut heartbeat_interval = time::interval(Duration::from_secs(30));

		loop {
			tokio::select! {
                result = rx.recv() => {
                    match result {
                        Ok((sender_id, stats)) => {
                            // Only forward messages from other clients (not from self)
                            if sender_id != client_id_for_writer {
                                let stats_msg = format!(
                                    "STATS|{}|{}|{}|{}|{}|{}|{}\n",
                                    stats.timestamp,
                                    stats.cpu_usage,
                                    stats.memory_usage,
                                    stats.disk_usage,
                                    stats.system_load,
                                    stats.network_rx,
                                    stats.network_tx
                                );

                                if let Err(e) = writer.write_all(stats_msg.as_bytes()).await {
                                    log::error!("Error writing to client {}: {}", sender_id, e);
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("Error receiving broadcast: {}", e);
                            break;
                        }
                    }
                }
                _ = heartbeat_interval.tick() => {
                    // Send a heartbeat message to make sure connection is still alive
                    if let Err(e) = writer.write_all(b"HEARTBEAT\n").await {
                        log::error!("Error sending heartbeat to client {}: {}", client_id_for_writer, e);
                        break;
                    }
                }
                _ = time::sleep(Duration::from_millis(100)), if !running.load(Ordering::SeqCst) => {
                    // Server is shutting down
                    break;
                }
            }
		}
	});

	// Read messages from this client
	while running.load(Ordering::SeqCst) {
		let read_result = tokio::select! {
            result = reader.read_line(&mut line) => result,
            _ = time::sleep(Duration::from_millis(100)), if !running.load(Ordering::SeqCst) => {
                // Server is shutting down
                break;
            }
        };

		match read_result {
			Ok(n) if n == 0 => {
				// EOF, client disconnected
				break;
			}
			Ok(_) => {
				let msg = line.trim();
				log::info!("Received from {}: {}", client_id, msg);

				// Try to parse as system stats
				if let Some(stats) = parse_stats_line(&line) {
					// Update client stats
					{
						if let Ok(mut clients_map) = clients.lock() {
							if let Some(client) = clients_map.get_mut(&client_id) {
								client.last_seen = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
								client.stats = stats.clone();
							}
						}
					}

					// Broadcast stats to other clients
					let _ = tx.send((client_id.clone(), stats));
				} else if msg == "HEARTBEAT" {
					// Update last seen time for heartbeat
					if let Ok(mut clients_map) = clients.lock() {
						if let Some(client) = clients_map.get_mut(&client_id) {
							client.last_seen = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
						}
					}
				} else {
					// Echo back for regular messages
					let response = format!("Echo: {}\n", msg);
					if let Err(e) = writer.write_all(response.as_bytes()).await {
						log::error!("Error writing echo response to {}: {}", client_id, e);
						break;
					}
				}

				line.clear();
			}
			Err(e) => {
				log::error!("Error reading from client {}: {}", client_id, e);
				break;
			}
		}
	}

	// Cancel the writer task
	writer_task.abort();

	// Client disconnected, remove from map
	{
		let mut clients_map = clients.lock().unwrap();
		clients_map.remove(&client_id);
	}

	log::info!("Client {} disconnected", client_id);
}
/*use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// Structure to hold system statistics
#[derive(Clone, Debug)]
struct SystemStats {
	timestamp: String,
	cpu_usage: f64,
	memory_usage: f64,
	disk_usage: f64,
	system_load: f64,
	network_rx: u64,
	network_tx: u64,
}

impl Default for SystemStats {
	fn default() -> Self {
		Self {
			timestamp: String::new(),
			cpu_usage: 0.0,
			memory_usage: 0.0,
			disk_usage: 0.0,
			system_load: 0.0,
			network_rx: 0,
			network_tx: 0,
		}
	}
}

// Structure to hold client information
struct ClientInfo {
	id: String,
	last_seen: u64,
	stats: SystemStats,
}

#[tokio::main]
async fn main() {
	// Initialize logger
	env_logger::builder().filter_level(log::LevelFilter::Info).init();

	// Create a shared state for connected clients
	let clients = Arc::new(Mutex::new(HashMap::<String, ClientInfo>::new()));

	// Create a broadcast channel for system stats
	let (tx, _) = broadcast::channel::<(String, SystemStats)>(100);

	// Start TCP server
	let listener = TcpListener::bind("0.0.0.0:7800").await.expect("Failed to bind to address");
	println!("Server listening on 0.0.0.0:7800");
	log::info!("Server listening on 0.0.0.0:7800");

	// Accept connections
	loop {
		match listener.accept().await {
			Ok((socket, addr)) => {
				println!("New connection from: {}", addr);
				log::info!("New connection from: {}", addr);

				// Clone the shared state for this connection
				let client_map = Arc::clone(&clients);
				let tx_clone = tx.clone();

				// Spawn a task to handle this connection
				tokio::spawn(async move {
					handle_connection(socket, addr.to_string(), client_map, tx_clone).await;
				});
			}
			Err(e) => {
				println!("Error accepting connection: {}", e);
				log::error!("Error accepting connection: {}", e);
			}
		}
	}
}

async fn handle_connection(
	socket: TcpStream,
	client_id: String,
	clients: Arc<Mutex<HashMap<String, ClientInfo>>>,
	tx: broadcast::Sender<(String, SystemStats)>
) {
	let (reader, mut writer) = socket.into_split();
	let mut reader = BufReader::new(reader);
	let mut line = String::new();

	// Add client to connected clients
	{
		let mut clients_map = clients.lock().unwrap();
		clients_map.insert(client_id.clone(), ClientInfo {
			id: client_id.clone(),
			last_seen: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
			stats: SystemStats::default(),
		});
	}

	// Subscribe to broadcast channel
	let mut rx = tx.subscribe();

	// Spawn a task to send broadcast messages to this client
	let client_id_for_writer = client_id.clone(); // Clone here before moving into closure
	let writer_task = tokio::spawn(async move {
		while let Ok((sender_id, stats)) = rx.recv().await {
			// Only forward messages from other clients (not from self)
			if sender_id != client_id_for_writer { // Use the cloned value
				let stats_msg = format!(
					"STATS|{}|{}|{}|{}|{}|{}|{}\n",
					stats.timestamp,
					stats.cpu_usage,
					stats.memory_usage,
					stats.disk_usage,
					stats.system_load,
					stats.network_rx,
					stats.network_tx
				);

				if let Err(e) = writer.write_all(stats_msg.as_bytes()).await {
					println!("Error writing to client {}: {}", sender_id, e);
					log::error!("Error writing to client {}: {}", sender_id, e);
					break;
				}
			}
		}
	});

	// Read messages from this client
	while let Ok(n) = reader.read_line(&mut line).await {
		if n == 0 {
			// EOF, client disconnected
			break;
		}

		let msg = line.trim();
		println!("Received from {}: {}", client_id, msg);
		log::info!("Received from {}: {}", client_id, msg);

		// Try to parse as system stats
		if msg.contains("|") {
			let parts: Vec<&str> = msg.split('|').collect();
			if parts.len() >= 7 {
				// Parse system stats
				let stats = SystemStats {
					timestamp: parts[0].to_string(),
					cpu_usage: parts[1].parse().unwrap_or(0.0),
					memory_usage: parts[2].parse().unwrap_or(0.0),
					disk_usage: parts[3].parse().unwrap_or(0.0),
					system_load: parts[4].parse().unwrap_or(0.0),
					network_rx: parts[5].parse().unwrap_or(0),
					network_tx: parts[6].parse().unwrap_or(0),
				};

				// Update client stats
				{
					if let Ok(mut clients_map) = clients.lock() {
						if let Some(client) = clients_map.get_mut(&client_id) {
							client.last_seen = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
							client.stats = stats.clone();
						}
					}
				}

				// Broadcast stats to other clients
				let _ = tx.send((client_id.clone(), stats)); // Make sure to clone here
			}
		} else {
			// Echo back for regular messages
			let response = format!("Echo: {}\n", msg);
			// Fix the is_finished check
			if writer_task.is_finished() {
				println!("Writer task finished, can't send response");
				break;
			}
			// You'll need a way to send this response - perhaps through another channel
		}

		line.clear();
	}

	// Client disconnected, remove from map
	{
		let mut clients_map = clients.lock().unwrap();
		clients_map.remove(&client_id); // This is the last use, no need to clone
	}

	println!("Client {} disconnected", client_id);
	log::info!("Client {} disconnected", client_id);
}

 */