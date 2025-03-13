// CLIENT CODE (client.rs)
use rand::Rng;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    // Initialize logger with default level of INFO
    env_logger::builder().filter_level(log::LevelFilter::Info).init();

    let server_address = "127.0.0.1:7800";
    println!("Starting client, will connect to server at {}", server_address);
    log::info!("Starting client, will connect to server at {}", server_address);

    loop {
        println!("Attempting to connect to server...");
        // Attempt to connect to the TCP server
        match TcpStream::connect(server_address).await {
            Ok(stream) => {
                println!("Connected to server at {}", server_address);
                log::info!("Connected to server at {}", server_address);
                let (reader, mut writer) = stream.into_split();
                let mut reader = BufReader::new(reader);
                let mut rng = rand::thread_rng();

                // Spawn a task to handle server responses
                let read_handle = tokio::spawn(async move {
                    let mut line = String::new();
                    while let Ok(n) = reader.read_line(&mut line).await {
                        if n == 0 {
                            // EOF, server closed the connection
                            println!("Server disconnected");
                            log::info!("Server disconnected");
                            break;
                        }
                        println!("Server response: {}", line.trim());
                        log::info!("Server response: {}", line.trim());
                        line.clear();
                    }
                });

                // Send random values to the server
                loop {
                    // Generate a random float
                    let random_float: f64 = rng.gen_range(5.0..9.0);
                    let random_string = random_float.to_string();

                    println!("Sending: {}", random_string);
                    // Send the random float as a string over the TCP connection
                    if let Err(e) = writer.write_all(format!("{}\n", random_string).as_bytes()).await {
                        println!("Failed to send data: {}", e);
                        log::error!("Failed to send data: {}", e);
                        break;
                    }

                    println!("Sent: {}", random_string);
                    log::info!("Sent: {}", random_string);

                    // Sleep for a short duration before sending the next value
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }

                // Wait for the read task to complete
                if let Err(e) = read_handle.await {
                    println!("Error in read task: {}", e);
                    log::error!("Error in read task: {}", e);
                }
            }
            Err(e) => {
                println!("Failed to connect to server: {}", e);
                log::error!("Failed to connect to server: {}", e);
                println!("Will try again in 10 seconds...");
            }
        }

        // Sleep and try later for the next connect
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}



/*mod stressapp;

use std::time::Duration;

use iced::{
    Alignment, Element, Length, Task,
    widget::{Column, Container, Text},
};
use stressapp::message::AppMessage;
use stressapp::monitor_chart::MonitorChart;

struct State {
    server_chart: MonitorChart,
}

impl State {
    fn new() -> (Self, Task<AppMessage>) {
        (
            Self {
                server_chart: Default::default(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: AppMessage) {
        match message {
            AppMessage::NewDataPoint(basic_message) => {
                //Update the servers here
                self.server_chart.send_message(basic_message);
            }
            AppMessage::Tick => {
                self.server_chart.update();
            }
        }
    }

    fn view(&self) -> Element<'_, AppMessage> {
        let content = Column::new()
            .spacing(20)
            .align_x(Alignment::Start)
            .width(Length::Fill)
            .height(Length::Fill)
            .push(Text::new("Server"))
            .push(self.server_chart.view());

        Container::new(content)
            //.style(style::Container)
            .padding(5)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}

fn main() {
    iced::application("CPU Monitor Example", State::update, State::view)
        .antialiasing(true)
        .subscription(|_| {
            const FPS: u64 = 50;
            iced::time::every(Duration::from_millis(1000 / FPS)).map(|_| AppMessage::Tick)
        })
        .run_with(State::new)
        .unwrap();
}
*/