// SERVER CODE (server.rs)
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
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
