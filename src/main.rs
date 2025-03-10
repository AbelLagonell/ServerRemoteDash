mod counter_example;
mod plotters_iced;
mod stress_senders;

use crate::counter_example::counter::Counter;

fn main() -> iced::Result {
    iced::run("Counter", Counter::update, Counter::view)
}
