use std::thread;
use std::time::Duration;

use amane::{Color, Rectangle, Service, Text};

use crate::theme;

pub struct Clock {
    time: String,
}

impl Service for Clock {
    fn new() -> Self {
        Self {
            time: amane::output("date +%H:%M"),
        }
    }

    fn listen() {
        loop {
            let seconds = amane::output("date +%S")
                .parse::<u64>()
                .unwrap_or(0)
                .min(59);

            thread::sleep(Duration::from_secs(60 - seconds));

            Self::write().time = amane::output("date +%H:%M");
        }
    }
}

pub fn view() -> Rectangle {
    let clock = Clock::read();

    Rectangle::new()
        .width(58.0)
        .height(24.0)
        .fill(theme::MODULE_BACKGROUND)
        .radius(7.0)
        .align_child(amane::Center, amane::Center)
        .child(
            Text::new(&clock.time)
                .size(14.0)
                .color(Color::from(theme::MODULE_FOREGROUND)),
        )
}
