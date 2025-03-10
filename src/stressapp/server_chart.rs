use std::time::{Duration, Instant};

use iced::{
    Alignment, Element, Length,
    alignment::{Horizontal, Vertical},
    widget::{Column, Scrollable, Text},
};

use super::{message::AppMessage, util_chart::UtilChart};

const SAMPLE_EVERY: Duration = Duration::from_millis(1000);

pub struct ServerChart {
    //holds the various charts
    server: i8,
    last_sample_time: Instant,
    items_per_row: usize,
    util_charts: Vec<UtilChart>,
    chart_height: f32,
}

impl Default for ServerChart {
    fn default() -> Self {
        Self {
            server: -1,
            last_sample_time: Instant::now(),
            items_per_row: 4,
            util_charts: Default::default(),
            chart_height: 300.0,
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

    pub fn update(&mut self) {
        if !self.should_update() {
            return;
        }

        self.last_sample_time = Instant::now();
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
