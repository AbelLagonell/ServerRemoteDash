use std::net::{TcpListener, TcpStream};
use std::io::Write;
use std::thread;
use std::time::Duration;

fn handle_client(mut stream: TcpStream) {
	for i in 1.. {
		let message = format!("Update {} from server\n", i);
		if stream.write(message.as_bytes()).is_err() {
			println!("Client disconnected.");
			break;
		}
		thread::sleep(Duration::from_secs(2)); // Simulating periodic updates
	}
}

fn main() {
	let listener = TcpListener::bind("127.0.0.1:7878").expect("Could not bind to address");

	println!("Server listening on 127.0.0.1:7878");

	for stream in listener.incoming() {
		match stream {
			Ok(stream) => {
				println!("New client connected.");
				thread::spawn(move || handle_client(stream));
			}
			Err(e) => {
				eprintln!("Connection failed: {}", e);
			}
		}
	}
}
