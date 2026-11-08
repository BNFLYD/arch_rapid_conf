use std::thread;
use std::time::Duration;

use amane::{Color, Cpu, Parent, Rectangle, Service, Text};

use crate::theme;

pub struct CavaLeft {
    text: String,
}

impl Service for CavaLeft {
    fn new() -> Self {
        Self {
            text: String::new(),
        }
    }

    fn listen() {
        for line in amane::lines("$HOME/.config/waybar/scripts/cava-left.sh") {
            Self::write().text = line;
        }
    }
}

pub struct CavaRight {
    text: String,
}

impl Service for CavaRight {
    fn new() -> Self {
        Self {
            text: String::new(),
        }
    }

    fn listen() {
        for line in amane::lines("$HOME/.config/waybar/scripts/cava-right.sh") {
            Self::write().text = line;
        }
    }
}

// runcat: se mueve más rápido cuanto más carga de CPU hay
pub struct Runcat {
    text: String,
}

const ICONS: [char; 5] = ['\u{E900}', '\u{E901}', '\u{E902}', '\u{E903}', '\u{E904}'];
// the cat is in the system panel, and that panel is a fullscreen window: every
// frame of the cat redraws all of it, so it cannot go too fast
const FPS_L: f32 = 3.0;
const FPS_H: f32 = 10.0;

// misma fórmula del script original: intervalo lineal en el uso de CPU
fn interval(percent: u8) -> f32 {
    let delta = (1.0 / FPS_L - 1.0 / FPS_H) / 100.0;

    (1.0 / FPS_L - delta * percent as f32).max(1.0 / FPS_H)
}

impl Service for Runcat {
    fn new() -> Self {
        Self {
            text: ICONS[0].to_string(),
        }
    }

    fn listen() {
        let mut frame = 0usize;

        loop {
            // the cat only moves while the panel that shows it is open: a
            // hidden panel still gets redrawn, and that one covers the screen
            if !crate::system_panel::is_open() {
                thread::sleep(Duration::from_millis(250));
                continue;
            }

            let percent = Cpu::read().percent();

            Self::write().text = ICONS[frame % ICONS.len()].to_string();
            frame += 1;

            thread::sleep(Duration::from_secs_f32(interval(percent)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interval_goes_from_slow_to_fast() {
        assert_eq!(interval(0), 1.0 / FPS_L);
        assert_eq!(interval(100), 1.0 / FPS_H);
    }
}

pub fn cava_left() -> Rectangle {
    let cava = CavaLeft::read();

    cava_view(&spaced_bars(&cava.text))
}

pub fn cava_right() -> Rectangle {
    let cava = CavaRight::read();

    cava_view(&spaced_bars(&cava.text))
}

fn cava_view(text: &str) -> Rectangle {
    Rectangle::new()
        .width(216.0)
        .height(Parent)
        .align_child(amane::Center, amane::Center)
        .child(Text::new(text).size(12.0).color(Color::from("#dad4c3")))
}

pub fn runcat_view(width: f32, size: f32) -> Rectangle {
    let runcat = Runcat::read();

    Rectangle::new()
        .width(width)
        .height(Parent)
        .align_child(amane::Center, amane::Center)
        .child(
            Text::new(&runcat.text)
                .size(size)
                .font(theme::RUNCAT_FONT)
                .color(Color::from("#dad4c3")),
        )
}

fn spaced_bars(text: &str) -> String {
    let mut spaced = String::new();

    for character in text.chars() {
        if !spaced.is_empty() {
            spaced.push(' ');
        }

        spaced.push(character);
    }

    spaced
}
