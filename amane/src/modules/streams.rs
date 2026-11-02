use amane::{Color, Parent, Rectangle, Service, Text};

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

pub struct Runcat {
    text: String,
}

impl Service for Runcat {
    fn new() -> Self {
        Self {
            text: String::new(),
        }
    }

    fn listen() {
        for line in amane::lines(
            "$HOME/.config/waybar/modules/runcat-text/.venv/bin/python $HOME/.config/amane/runcat/main.py",
        ) {
            Self::write().text = line;
        }
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
        .child(Text::new(text).size(12.0).color(Color::from("#c5bfae")))
}

pub fn runcat() -> Rectangle {
    let runcat = Runcat::read();

    Rectangle::new()
        .width(42.0)
        .height(Parent)
        .align_child(amane::Center, amane::Center)
        .child(
            Text::new(&runcat.text)
                .size(37.0)
                .font(theme::RUNCAT_FONT)
                .color(Color::from("#c5bfae")),
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
