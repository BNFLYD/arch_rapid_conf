use amane::{Audio, Button, Color, Rectangle, Scroll, Service, Text};

use crate::theme;

pub fn view() -> Rectangle {
    let audio = Audio::read();

    let icon = if audio.muted() {
        ""
    } else if audio.volume() == 0 {
        ""
    } else if audio.volume() < 50 {
        ""
    } else {
        ""
    };

    let label = format!("{icon} {}%", audio.volume());

    Rectangle::new()
        .width(78.0)
        .height(24.0)
        .fill(theme::MODULE_BACKGROUND)
        .radius(7.0)
        .align_child(amane::Center, amane::Center)
        .on_click(open_volume_controls)
        .on_scroll(change_volume)
        .child(
            Text::new(label)
                .size(14.0)
                .color(Color::from(theme::MODULE_FOREGROUND)),
        )
}

fn open_volume_controls(_: Button) {
    amane::spawn("pavucontrol");
}

fn change_volume(scroll: Scroll) {
    let current = i32::from(Audio::read().volume());
    let step = if scroll.y < 0.0 { 5 } else { -5 };
    let changed = (current + step).clamp(0, 100);

    Audio::set_volume(changed as u8);
}
