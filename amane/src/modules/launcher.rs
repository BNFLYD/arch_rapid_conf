use amane::{Button, Color, Pointer, Rectangle, Text, Weight};

use crate::{system_panel, theme};

pub fn view() -> Rectangle {
    Rectangle::new()
        .width(34.0)
        .height(24.0)
        .fill(theme::MODULE_BACKGROUND)
        .radius(7.0)
        .align_child(amane::Center, amane::Center)
        .cursor(Pointer)
        .on_click(|button| match button {
            Button::Left => {
                system_panel::close();
                amane::spawn("wofi --show drun");
            }
            Button::Right => system_panel::toggle(),
            Button::Middle => {}
        })
        .child(
            Text::new("〇")
                .size(18.0)
                .weight(Weight::Bold)
                .color(Color::from(theme::MODULE_FOREGROUND)),
        )
}
