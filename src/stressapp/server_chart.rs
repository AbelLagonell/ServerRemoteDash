use std::time::{Duration, Instant};

use iced::{
    Alignment, Element, Length,
    alignment::{Horizontal, Vertical},
    futures::stream::pending,
    widget::{Column, Scrollable, Text},
};

use super::{
    message::{AppMessage, BasicMessage},
    util_chart::UtilChart,
};

const SAMPLE_EVERY: Duration = Duration::from_millis(1000);

pub struct ServerChart {
    //holds the various charts
    server: i8,
    last_sample_time: Instant,
    util_charts: Vec<(u8, UtilChart)>,
    chart_height: f32,
    pending_messages: Vec<BasicMessage>,
}

impl Default for ServerChart {
    fn default() -> Self {
        Self {
            server: -1,
            last_sample_time: Instant::now(),
            util_charts: Default::default(),
            chart_height: 300.0,
            pending_messages: Default::default(),
        }
    }
}

impl ServerChart {
    #[inline]
    fn is_initialized(&self) -> bool {
        !self.util_charts.is_empty()
    }

    #[inline]
    fn should_update(&self) -> bool {
        !self.is_initialized() || self.last_sample_time.elapsed() > SAMPLE_EVERY
    }

    pub fn add_message(&mut self, basic_msg: BasicMessage) {
        self.pending_messages.push(basic_msg);
    }

    pub fn update(&mut self) {
        if !self.should_update() {
            return;
        }

        self.last_sample_time = Instant::now();

        for msg in &self.pending_messages {
            if self.util_charts.iter().any(|e| e.0 == msg.stress_tester) {
                //Add Missing chart
                let mut new_chart = UtilChart::new((msg.timestamp, msg.percentage));
                self.util_charts
                    .append(&mut vec![(msg.stress_tester, new_chart)]);
            } else {
                //Adds it to the util chart
            }
        }

        //Updates each utility chart based on message
    }

    pub fn view(&self) -> Element<AppMessage> {
        if !self.is_initialized() {
            Text::new("Loading...")
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .into()
        } else {
            let mut _col = Column::new()
                .width(Length::Fill)
                .height(Length::Shrink)
                .align_x(Alignment::Center);

            let _chart_height = self.chart_height;
            let mut _title = "";

            //Add the UtilChart

            Scrollable::new(_col).height(Length::Shrink).into()
        }
    }
}
