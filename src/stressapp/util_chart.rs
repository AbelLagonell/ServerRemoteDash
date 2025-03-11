use std::{collections::VecDeque, time::Duration};

use super::message::AppMessage;
use chrono::{DateTime, Utc};
use iced::{
    Alignment, Element, Length, Size,
    widget::{
        Column, Text,
        canvas::{Cache, Frame, Geometry},
    },
};
use plotters_iced::{Chart, ChartBuilder, ChartWidget, DrawingBackend, Renderer};

const PLOT_SECONDS: usize = 60; //min

pub struct UtilChart {
    cache: Cache,
    data_points: VecDeque<(DateTime<Utc>, f32)>,
    limit: Duration,
}

impl UtilChart {
    pub fn new(data: (DateTime<Utc>, f32)) -> Self {
        Self {
            cache: Cache::new(),
            data_points: VecDeque::from([data]),
            limit: Duration::from_secs(PLOT_SECONDS as u64),
        }
    }

    pub fn push_data(&mut self, time: DateTime<Utc>, percentage: f32) {
        let cur_ms = time.timestamp_millis();
        self.data_points.push_front((time, percentage));
        loop {
            if let Some((time, _)) = self.data_points.back() {
                let diff = Duration::from_millis((cur_ms - time.timestamp_millis()) as u64);
                if diff > self.limit {
                    self.data_points.pop_back();
                    continue;
                }
            }
            break;
        }
        self.cache.clear();
    }

    pub fn view(&self, title: String, chart_height: f32) -> Element<AppMessage> {
        Column::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(5)
            .align_x(Alignment::Center)
            .push(Text::new(title))
            .push(ChartWidget::new(self).height(Length::Fixed(chart_height)))
            .into()
    }
}

impl Chart<AppMessage> for UtilChart {
    type State = ();

    #[inline]
    fn draw<R: Renderer, F: Fn(&mut Frame)>(
        &self,
        renderer: &R,
        bounds: Size,
        draw_fn: F,
    ) -> Geometry {
        renderer.draw_cache(&self.cache, bounds, draw_fn)
    }

    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut chart: ChartBuilder<DB>) {
        use plotters::prelude::*;

        const PLOT_LINE_COLOR: RGBColor = RGBColor(0, 175, 255);

        // Acquire time range
        let newest_time = self
            .data_points
            .front()
            .unwrap_or(&(DateTime::from_timestamp(0, 0).unwrap(), 0.0))
            .0;
        let oldest_time = newest_time - chrono::Duration::seconds(PLOT_SECONDS as i64);
        let mut chart = chart
            .x_label_area_size(0)
            .y_label_area_size(28)
            .margin(20)
            .build_cartesian_2d(oldest_time..newest_time, 0.0..100.0 as f32)
            .expect("failed to build chart");

        chart
            .configure_mesh()
            .bold_line_style(plotters::style::colors::BLUE.mix(0.1))
            .light_line_style(plotters::style::colors::BLUE.mix(0.05))
            .axis_style(ShapeStyle::from(plotters::style::colors::BLUE.mix(0.45)).stroke_width(1))
            .y_labels(10)
            .y_label_style(
                ("sans-serif", 15)
                    .into_font()
                    .color(&plotters::style::colors::BLUE.mix(0.65))
                    .transform(FontTransform::Rotate90),
            )
            .y_label_formatter(&|y: &f32| format!("{}%", y))
            .draw()
            .expect("failed to draw chart mesh");

        chart
            .draw_series(
                AreaSeries::new(
                    self.data_points.iter().map(|x| (x.0, x.1)),
                    0 as f32,
                    PLOT_LINE_COLOR.mix(0.175),
                )
                .border_style(ShapeStyle::from(PLOT_LINE_COLOR).stroke_width(2)),
            )
            .expect("failed to draw chart data");
    }
}
