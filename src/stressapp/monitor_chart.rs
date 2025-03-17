use std::time::{Duration, Instant};

use iced::{
    alignment::{Horizontal, Vertical}, widget::{Column, Scrollable, Space, Text}, Alignment,
    Element,
    Length,
};

use super::{
    message::{AppMessage, BasicMessage},
    server_chart::ServerChart,
};
use crate::stressapp::message::parse_message;
use std::path::Path;
use std::{fs, fs::{File, OpenOptions}, io::{self, BufRead, Write}};

const SAMPLE_EVERY: Duration = Duration::from_millis(1000);

pub struct MonitorChart {
    //holds the server charts
    servers: Vec<(u8, ServerChart)>,
    last_sample_time: Instant,
    directory: String,
}

impl Default for MonitorChart {
    fn default() -> Self {
        Self {
            last_sample_time: Instant::now(),
            servers: Default::default(),
            directory: String::new() + "tcp_logs",
        }
    }
}

impl MonitorChart {
    #[inline]
    fn is_initialized(&self) -> bool {
        !self.servers.is_empty()
    }

    #[inline]
    fn should_update(&self) -> bool {
        !self.is_initialized() || self.last_sample_time.elapsed() > SAMPLE_EVERY
    }

    pub fn send_message(&mut self, msg: BasicMessage) {
        // Add any new server or update existing servers
        if !self.servers.iter().any(|e| e.0 == msg.stress_tester) {
            let new_server = ServerChart::default();
            self.servers.push((msg.stress_tester, new_server));
        }

        for (server_id, server) in &mut self.servers {
            if *server_id == msg.server_id {
                server.add_message(msg.clone());
            }
        }
    }

    pub fn update(&mut self) {
        if !self.should_update() {
            return;
        }

        // Process files in the directory
        if let Err(e) = self.read_files_in_directory() {
            eprintln!("Error reading files from directory: {}", e);
        }

        self.last_sample_time = Instant::now();

        for (_, server) in &mut self.servers {
            server.update();
        }
    }

    pub fn view(&self) -> Element<'_, AppMessage> {
        if !self.is_initialized() {
            Text::new("Loading...").align_x(Horizontal::Center).align_y(Vertical::Center).into()
        } else {
            let mut col = Column::new()
                .spacing(15)
                .width(Length::Fill)
                .height(Length::Shrink)
                .align_x(Alignment::Center);

            for (id, server) in &self.servers {
                col = col.push(Text::new(format!("Server {}", id)));
                col = col.push(server.view());
                col = col.push(Space::new(Length::Fixed(50.0), Length::Fill));
            }

            Scrollable::new(col).height(Length::Shrink).into()
        }
    }

    pub fn read_and_process_file(&mut self, file_path: &str) -> io::Result<()> {
        // Open the file for reading and writing
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(file_path)?;
        let mut lines = io::BufReader::new(file).lines();

        // Check if there is a line to read
        if let Some(Ok(line)) = lines.next() {
            // Parse the message from the line
            if let Some(msg) = parse_message(&line) {
                // Send the parsed message
                self.send_message(msg);

                // Get the file's content and remove the first line
                let remaining_lines: Vec<String> = lines.filter_map(Result::ok).collect();

                // Rewrite the file with the remaining lines (removes the first line)
                let mut file = File::create(file_path)?;
                for remaining_line in remaining_lines {
                    writeln!(file, "{}", remaining_line)?;
                }
            }
        }

        Ok(())
    }

    fn read_file(&mut self, path: &Path) -> io::Result<()> {
        let file = File::open(path)?;
        let reader = io::BufReader::new(file);

        for line in reader.lines() {
            if let Ok(message) = line {
                if let Some(parsed_message) = parse_message(&message) {
                    self.send_message(parsed_message);
                }
            }
        }

        Ok(())
    }

    fn clear_file(&self, path: &Path) -> io::Result<()> {
        // Clear the contents of the file after processing
        std::fs::write(path, "")?;
        Ok(())
    }

    fn read_files_in_directory(&mut self) -> io::Result<()> {
        // Read all files in the directory
        let entries = fs::read_dir(&self.directory)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // Check if it's a file and has a .txt extension
            if path.is_file() && path.extension() == Some("log".as_ref()) {
                // Process the file
                self.read_file(&path)?;
                // Clear the file after reading
                self.clear_file(&path)?;
            }
        }

        Ok(())
    }
}
