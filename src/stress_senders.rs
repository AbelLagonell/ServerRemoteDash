use rand::Rng;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
#[tokio::main]
async fn main() {
	env_logger::init();
	let server_address = "127.0.0.1:7800";
	log::info!("Start at {}", server_address);
	loop {
		// Attempt to connect to the TCP server
		match TcpStream::connect(server_address).await {
			Ok(mut stream) => {
				log::info!("Connected to server at {}", server_address);
				let mut rng = rand::thread_rng();
				loop {
					// Generate a random float
					let random_float: f64 = rng.gen_range(5.0..9.0);
					let random_string = random_float.to_string();
					// Send the random float as a string over the TCP connection
					if let Err(e) = stream.write_all(random_string.as_bytes()).await {
						log::error!("Failed to send data: {}", e);
						break;
					}
					if let Err(e) = stream.write_all(b"\n").await {
						log::error!("Failed to send newline: {}", e);
						break;
					}
					log::info!("Sent: {}", random_string);
					// Sleep for a short duration before sending the next value
					tokio::time::sleep(Duration::from_secs(1)).await;
				}
			}
			Err(e) => {
				log::error!("Failed to connect to server: {}", e);
			}
		}
		// Sleep and try later for the next connect
		tokio::time::sleep(Duration::from_secs(10)).await;
	}
}
