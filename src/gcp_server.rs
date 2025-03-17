use rand::Rng;
use std::io::{self, Write};
use std::net::TcpStream;
use std::thread;
use std::time::{Duration, SystemTime};

fn main() -> io::Result<()> {
    // Configuration
    let server_address = "127.0.0.1:8888"; // Change to your central server address
    let server_id = 0; // Change this for each cloud server instance
    let reconnect_delay = Duration::from_secs(5);

    println!("Cloud server {} starting up...", server_id);

    // Continuously try to connect and send data
    loop {
        println!(
            "Attempting to connect to central server at {}",
            server_address
        );

        match TcpStream::connect(server_address) {
            Ok(mut stream) => {
                println!("Connected to central server!");

                // Start sending monitoring data
                println!("Starting to send monitoring data...");

                // Keep sending data until connection fails
                loop {
                    // Generate random monitoring data
                    let message = generate_random_monitoring_data(server_id);

                    // Send the message
                    match stream.write_all(message.as_bytes()) {
                        Ok(_) => {
                            println!("Sent: {}", message.trim());
                            // Flush to ensure data is sent
                            if let Err(e) = stream.flush() {
                                println!("Error flushing stream: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            println!("Error sending data: {}", e);
                            break;
                        }
                    }

                    // Sleep before sending next message
                    thread::sleep(Duration::from_millis(1000));
                }

                println!("Lost connection to central server. Will try to reconnect...");
            }
            Err(e) => {
                println!("Failed to connect to central server: {}", e);
            }
        }

        // Wait before trying to reconnect
        println!(
            "Waiting {} seconds before reconnecting...",
            reconnect_delay.as_secs()
        );
        thread::sleep(reconnect_delay);
    }
}

fn generate_random_monitoring_data(server_id: u8) -> String {
    let mut rng = rand::thread_rng();

    // Generate random values
    let metric_type = rng.gen_range(0..5); // 0=CPU, 1=IP, 2=Network, 3=FS, 4=Memory
    let utilization = rng.gen_range(0.0..100.0);

    // Get current time
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0));

    // Format time as hh:mm:ss
    let total_seconds = now.as_secs() % (24 * 3600);
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    let time_str = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);

    // Format the message according to the specified format
    format!(
        "{}-{}-{:.1}-{}\n",
        server_id, metric_type, utilization, time_str
    )
}
