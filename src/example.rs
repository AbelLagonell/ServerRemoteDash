use iced::{
    color, futures::{stream::Stream, SinkExt}, stream, time,
    widget::{button, column, row, text, Column, Image, Row},
    window::Position, Point,
    Size,
    Subscription,
};
use plotters::prelude::*;
use std::collections::VecDeque;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::net::TcpListener;
const MAX_PRESSURE_VALUES: usize = 300;
/// Receives pressure values from a TCP stream and sends them to an output channel as `BasicAppMessages`.
///
/// This function sets up a TCP listener on a fixed address (127.0.0.1:7800) and waits for incoming
/// connections. When a client connects, it listens for pressure values sent as text lines, parses them
/// as `f32`, and sends the parsed values to a message stream. Each message also contains an incrementing
/// sequence number, stored in an `Arc<Mutex<f32>>` to ensure concurrency safety.
fn receive_and_send_pressure_value() -> impl Stream<Item = BasicAppMessages> {
    use std::sync::Arc;
    use tokio::sync::Mutex;
    stream::channel(2, |output| async move {
        let x = Arc::new(Mutex::new(0.0));
        let output = Arc::new(Mutex::new(output));

        let server_address = "127.0.0.1:7800";
        match TcpListener::bind(server_address).await {
            Ok(listener) => loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        let x = Arc::clone(&x);
                        let output = Arc::clone(&output);
                        tokio::spawn(async move {
                            let mut reader = BufReader::new(stream);
                            let mut line = String::new();
                            while let Ok(bytes_read) = reader.read_line(&mut line).await {
                                if bytes_read == 0 {
                                    log::error!("Connection was closed");
                                    break;
                                }
                                match line.trim().parse::<f32>() {
                                    Ok(value) => {
                                        let mut x_lock = x.lock().await;
                                        let mut output_lock = output.lock().await;
                                        let _res = output_lock
                                            .send(BasicAppMessages::NewPressureValue(
                                                *x_lock, value,
                                            ))
                                            .await;
                                        *x_lock += 1.0;
                                    }
                                    Err(e) => log::error!("Failed to parse value: {}", e),
                                }
                                line.clear();
                            }
                        });
                    }
                    Err(e) => {
                        log::error!("Failed to accept connection: {}", e);
                    }
                }
            },
            Err(e) => {
                log::error!("Failed to bind listener: {}", e);
            }
        }
    })
}
