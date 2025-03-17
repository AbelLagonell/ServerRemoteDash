use iced::{
    alignment::{Horizontal, Vertical}, widget::{Row, Space, Text}, Alignment,
    Element,
    Length,
};
use crate::stressapp::message::parse_message;
use super::{
    message::{AppMessage, BasicMessage},
    util_chart::UtilChart,
};

pub struct ServerChart {
    //holds the various charts
    util_charts: Vec<(u8, UtilChart)>,
    chart_height: f32,
    pending_messages: Vec<BasicMessage>,
}

impl Default for ServerChart {
    fn default() -> Self {
        Self {
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
        !self.is_initialized() || !self.pending_messages.is_empty()
    }

    pub fn add_message(&mut self, basic_msg: BasicMessage) {
        self.pending_messages.push(basic_msg);
    }

    pub fn update(&mut self) {
        if !self.should_update() {
            return;
        }

        for msg in &self.pending_messages {
            if !self.util_charts.iter().any(|e| e.0 == msg.stress_tester) {
                //Add Missing chart
                let new_chart = UtilChart::new((msg.timestamp, msg.percentage));
                self.util_charts
                    .append(&mut vec![(msg.stress_tester, new_chart)]);
            }
        }

        //Updates each utility chart based on message
        for (stress, chart) in &mut self.util_charts {
            for msg in &self.pending_messages {
                if stress == &msg.stress_tester {
                    chart.push_data(msg.timestamp, msg.percentage);
                }
            }
        }

        self.pending_messages.clear();
    }

    pub fn view(&self) -> Element<AppMessage> {
        if !self.is_initialized() {
            Text::new("Loading...")
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .into()
        } else {
            let chart_height = self.chart_height;
            let mut title: &str;

            let mut row = Row::new()
                .spacing(15)
                .padding(20)
                .width(Length::Fill)
                .height(Length::Shrink)
                .align_y(Alignment::Center);

            //Add the UtilChart
            for (stressor, chart) in &self.util_charts {
                match stressor {
                    0 => title = "cpu",
                    1 => title = "ip",
                    2 => title = "network",
                    3 => title = "fs",
                    4 => title = "memory",
                    _ => title = "",
                }
                row = row.push(chart.view(title.to_string(), chart_height));
                row = row.push(Space::new(Length::Fill, Length::Fixed(50.0)));
            }

            Element::new(row).into()
        }
    }
}
