mod counter_example;

use crate::counter_example::counter::Counter;

fn main() -> iced::Result {
    iced::run("Counter", Counter::update, Counter::view)
}
