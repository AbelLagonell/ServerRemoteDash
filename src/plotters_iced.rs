// CLIENT CODE (client.rs)
use rand::Rng;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use std::sync::{Arc, Mutex};
use iced::{
    Alignment, Element, Length, Task,
    widget::{Column, Container, Text, Row},
};
use std::process::Command;
use std::thread;

mod stressapp;
use stressapp::message::AppMessage;
use stressapp::monitor_chart::MonitorChart;
use stressapp::message::BasicMessage;

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

// Shared data structure that's thread-safe
struct SharedData {
    received_data: Vec<String>,
    system_stats: SystemStats,
}

struct State {
    server_chart: MonitorChart,
    cpu_chart: MonitorChart,
    memory_chart: MonitorChart,
    shared_data: Arc<Mutex<SharedData>>,
    current_stats: SystemStats,
}

impl State {
    fn new(shared_data: Arc<Mutex<SharedData>>) -> (Self, Task<AppMessage>) {
        (
            Self {
                server_chart: Default::default(),
                cpu_chart: Default::default(),
                memory_chart: Default::default(),
                shared_data,
                current_stats: SystemStats::default(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: AppMessage) {
        match message {
            AppMessage::NewDataPoint(basic_message) => {
                // Update the servers here
                self.server_chart.send_message(basic_message);
            }
            AppMessage::Tick => {
                self.server_chart.update();
                self.cpu_chart.update();
                self.memory_chart.update();

                // Check for new data from TCP client
                if let Ok(mut data) = self.shared_data.lock() {
                    // Process new system stats
                    self.current_stats = data.system_stats.clone();

                    // Update charts with system stats
                    if self.current_stats.cpu_usage > 0.0 {
                        let message = stressapp::message::BasicMessage {
                            server_id: 0,  // Choose appropriate value
                            stress_tester: 0,  // Choose appropriate value
                            percentage: self.current_stats.cpu_usage as f32,
                            timestamp: chrono::Utc::now(),
                        };
                        self.cpu_chart.send_message(message);
                    }

                    if self.current_stats.memory_usage > 0.0 {
                        let message = stressapp::message::BasicMessage {
                            server_id: 0,  // Choose appropriate value
                            stress_tester: 1,  // You might want to use different values for memory
                            percentage: self.current_stats.memory_usage as f32,
                            timestamp: chrono::Utc::now(),
                        };
                        self.memory_chart.send_message(message);
                    }

                    // Process any other received data
                    if !data.received_data.is_empty() {
                        // Process other data...
                        data.received_data.clear();
                    }
                }
            }
        }
    }

    fn view(&self) -> Element<'_, AppMessage> {
        let system_stats_view = Column::new()
            .spacing(10)
            .push(Text::new(format!("CPU Usage: {:.1}%", self.current_stats.cpu_usage)))
            .push(Text::new(format!("Memory Usage: {:.1}%", self.current_stats.memory_usage)))
            .push(Text::new(format!("Disk Usage: {:.1}%", self.current_stats.disk_usage)))
            .push(Text::new(format!("System Load: {:.2}", self.current_stats.system_load)))
            .push(Text::new(format!("Network RX: {} B/s", self.current_stats.network_rx)))
            .push(Text::new(format!("Network TX: {} B/s", self.current_stats.network_tx)));

        let charts = Row::new()
            .spacing(20)
            .push(self.cpu_chart.view())
            .push(self.memory_chart.view());

        let content = Column::new()
            .spacing(20)
            .align_x(Alignment::Start)
            .width(Length::Fill)
            .height(Length::Fill)
            .push(Text::new("System Monitor").size(24))
            .push(system_stats_view)
            .push(charts)
            .push(Text::new("Server Data"))
            .push(self.server_chart.view());

        Container::new(content)
            .padding(20)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn main() {
    // Initialize logger with default level of INFO
    env_logger::builder().filter_level(log::LevelFilter::Info).init();

    // Create shared data structure
    let shared_data = Arc::new(Mutex::new(SharedData {
        received_data: Vec::new(),
        system_stats: SystemStats::default(),
    }));

    // Clone for TCP client
    let tcp_shared_data = Arc::clone(&shared_data);

    // Start the bash monitoring script in separate process
    thread::spawn(|| {
        // Path to the monitoring script - ensure it's executable
        let script_path = "/path/to/system_monitor.sh";

        // Execute the bash script
        let status = Command::new("bash")
            .arg(script_path)
            .status()
            .expect("Failed to start monitoring script");

        println!("Monitoring script exited with status: {}", status);
    });

    // Start the TCP client in a separate OS thread
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            run_tcp_client(tcp_shared_data).await;
        });
    });

    // Run the Iced UI in the main thread
    iced::application("System Monitor", State::update, State::view)
        .antialiasing(true)
        .subscription(|_| {
            const FPS: u64 = 50;
            iced::time::every(Duration::from_millis(1000 / FPS)).map(|_| AppMessage::Tick)
        })
        .run_with(|| State::new(shared_data))
        .unwrap();
}

async fn run_tcp_client(shared_data: Arc<Mutex<SharedData>>) {
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

                // Clone shared data for reader task
                let reader_shared_data = Arc::clone(&shared_data);

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

                        let response = line.trim().to_string();
                        println!("Server response: {}", response);
                        log::info!("Server response: {}", response);

                        // Try to parse the response as system stats
                        if response.contains("|") {
                            let parts: Vec<&str> = response.split('|').collect();
                            if parts.len() >= 7 {
                                if let Ok(mut data) = reader_shared_data.lock() {
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
                                    data.system_stats = stats;
                                }
                            }
                        } else {
                            // Store regular data
                            if let Ok(mut data) = reader_shared_data.lock() {
                                data.received_data.push(response.clone());
                            }
                        }

                        line.clear();
                    }
                });

                // Send random values to the server (keeping this from original code)
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
/*use rand::Rng;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use std::sync::{Arc, Mutex};
use iced::{
    Alignment, Element, Length, Task,
    widget::{Column, Container, Text},
};

mod stressapp;
use stressapp::message::AppMessage;
use stressapp::monitor_chart::MonitorChart;

// Shared data structure that's thread-safe
struct SharedData {
    received_data: Vec<String>,
}

struct State {
    server_chart: MonitorChart,
    shared_data: Arc<Mutex<SharedData>>,
}

impl State {
    fn new(shared_data: Arc<Mutex<SharedData>>) -> (Self, Task<AppMessage>) {
        (
            Self {
                server_chart: Default::default(),
                shared_data,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: AppMessage) {
        match message {
            AppMessage::NewDataPoint(basic_message) => {
                // Update the servers here
                self.server_chart.send_message(basic_message);
            }
            AppMessage::Tick => {
                self.server_chart.update();

                // Check for new data from TCP client
                if let Ok(mut data) = self.shared_data.lock() {
                    // Process any new data here
                    // For example, you could convert data points and send them to the chart
                    if !data.received_data.is_empty() {
                        // Process data...
                        data.received_data.clear();
                    }
                }
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
            .padding(5)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}

fn main() {
    // Initialize logger with default level of INFO
    env_logger::builder().filter_level(log::LevelFilter::Info).init();

    // Create shared data structure
    let shared_data = Arc::new(Mutex::new(SharedData {
        received_data: Vec::new(),
    }));

    // Clone for TCP client
    let tcp_shared_data = Arc::clone(&shared_data);

    // Start the TCP client in a separate OS thread
    // This avoids Send trait issues with the Tokio runtime
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            run_tcp_client(tcp_shared_data).await;
        });
    });

    // Run the Iced UI in the main thread
    iced::application("CPU Monitor Example", State::update, State::view)
        .antialiasing(true)
        .subscription(|_| {
            const FPS: u64 = 50;
            iced::time::every(Duration::from_millis(1000 / FPS)).map(|_| AppMessage::Tick)
        })
        .run_with(|| State::new(shared_data))
        .unwrap();
}

async fn run_tcp_client(shared_data: Arc<Mutex<SharedData>>) {
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

                // Clone shared data for reader task
                let reader_shared_data = Arc::clone(&shared_data);

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

                        let response = line.trim().to_string();
                        println!("Server response: {}", response);
                        log::info!("Server response: {}", response);

                        // Store the data in shared storage
                        if let Ok(mut data) = reader_shared_data.lock() {
                            data.received_data.push(response.clone());
                        } else {
                            log::error!("Failed to acquire lock for storing TCP data");
                        }

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
*/