use std::time::Duration;
use iced::{Element, Subscription, Task};
use stressapp::message::AppMessage;
use stressapp::monitor_chart::MonitorChart;

mod stressapp;
mod gui_connection;

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

    fn title(&self) -> String {
        String::from("CPU Monitor Example")
    }

    fn update(&mut self, message: AppMessage){
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
        let content = iced::widget::Column::new()
            .spacing(20)
            .align_x(iced::Alignment::Start)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .push(iced::widget::Text::new("Server"))
            .push(self.server_chart.view());

        iced::widget::Container::new(content)
            .padding(5)
            .center_x(iced::Length::Fill)
            .center_y(iced::Length::Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<AppMessage> {
        Subscription::batch(vec![self.update_all()])
    }

    fn update_all(&self) -> Subscription<AppMessage> {
        const FPS: u64 = 50;
        iced::time::every(Duration::from_millis(1000 / FPS)).map(|_| AppMessage::Tick)
    }
}

fn main()  {
    iced::application("CPU Monitor Example", State::update, State::view)
        .antialiasing(true)
        .run_with(|| State::new())
        .unwrap()
}