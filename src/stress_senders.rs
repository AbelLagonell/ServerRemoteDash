// SERVER CODE (server.rs)
use std::io;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
	// Initialize logger with default level of INFO
	env_logger::builder().filter_level(log::LevelFilter::Info).init();

	let server_address = "127.0.0.1:7800";

	// Create a TCP listener bound to the specified address
	match TcpListener::bind(server_address).await {
		Ok(listener) => {
			println!("Server listening on {}", server_address);
			log::info!("Server listening on {}", server_address);

			loop {
				// Accept incoming connection
				match listener.accept().await {
					Ok((mut socket, addr)) => {
						println!("New client connected: {}", addr);
						log::info!("New client connected: {}", addr);

						// Spawn a new task to handle this client
						tokio::spawn(async move {
							let (reader, mut writer) = socket.split();
							let mut reader = BufReader::new(reader);
							let mut line = String::new();

							// Read lines from the client
							loop {
								// Clear line before reading into it
								line.clear();

								match reader.read_line(&mut line).await {
									Ok(0) => {
										// EOF, client closed the connection
										println!("Client {} disconnected", addr);
										log::info!("Client {} disconnected", addr);
										break;
									},
									Ok(_) => {
										println!("Received: {}", line.trim());

										// Process the received line
										if let Ok(value) = line.trim().parse::<f64>() {
											println!("Received value from {}: {}", addr, value);
											log::info!("Received value from {}: {}", addr, value);

											// Echo the value back to client if needed
											if let Err(e) = writer.write_all(format!("Received: {}\n", value).as_bytes()).await {
												println!("Failed to write to client: {}", e);
												log::error!("Failed to write to client: {}", e);
												break;
											}
										} else {
											println!("Received invalid data from {}: {}", addr, line.trim());
											log::warn!("Received invalid data from {}: {}", addr, line.trim());
										}
									},
									Err(e) => {
										println!("Error reading from client: {}", e);
										log::error!("Error reading from client: {}", e);
										break;
									}
								}
							}
						});
					}
					Err(e) => {
						println!("Failed to accept connection: {}", e);
						log::error!("Failed to accept connection: {}", e);
					}
				}
			}
		}
		Err(e) => {
			println!("Failed to bind to address {}: {}", server_address, e);
			log::error!("Failed to bind to address {}: {}", server_address, e);
		}
	}
}