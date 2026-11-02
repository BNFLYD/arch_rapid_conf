use amane::{Color, Pointer, Rectangle, Text, Weight};

use crate::theme;

pub fn view() -> Rectangle {
    Rectangle::new()
        .width(34.0)
        .height(24.0)
        .fill(theme::MODULE_BACKGROUND)
        .radius(7.0)
        .align_child(amane::Center, amane::Center)
        .cursor(Pointer)
        .on_click(|_| amane::spawn("wofi --show drun"))
        .child(
            Text::new("〇")
                .size(18.0)
                .weight(Weight::Bold)
                .color(Color::from(theme::MODULE_FOREGROUND)),
        )
}
