use std::time::{Duration, Instant};

use iced::{
    Alignment, Element, Length,
    alignment::{Horizontal, Vertical},
    widget::{Column, Scrollable, Space, Text},
};

use super::{
    message::{AppMessage, BasicMessage},
    server_chart::ServerChart,
};

const SAMPLE_EVERY: Duration = Duration::from_millis(1000);

pub struct MonitorChart {
    //holds the server charts
    servers: Vec<(u8, ServerChart)>,
    last_sample_time: Instant,
}

impl Default for MonitorChart {
    fn default() -> Self {
        Self {
            last_sample_time: Instant::now(),
            servers: Default::default(),
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
        //Add any new Server
        if !self.servers.iter().any(|e| e.0 == msg.stress_tester) {
            let new_server = ServerChart::default();
            self.servers.append(&mut vec![(msg.server_id, new_server)]);
        }

        //Update the servers
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

        self.last_sample_time = Instant::now();
        for (_, server) in &mut self.servers {
            server.update();
        }
    }

    pub fn view(&self) -> Element<'_, AppMessage> {
        if !self.is_initialized() {
            Text::new("Loading...")
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .into()
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
}
