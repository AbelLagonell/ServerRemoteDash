mod stressapp;

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
