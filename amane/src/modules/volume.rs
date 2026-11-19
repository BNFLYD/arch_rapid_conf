use amane::{Button, Center, Color, Padding, Rectangle, Row, Scroll, Text, children};

use crate::{audio_level, modules, theme};

pub fn view() -> Rectangle {
    let levels = audio_level::levels();

    let icon = if levels.muted {
        ""
    } else if levels.volume == 0 {
        ""
    } else if levels.volume < 50 {
        ""
    } else {
        ""
    };

    let label = format!("{}%", levels.volume);

    let icon_text = Text::new(icon)
        .size(14.0)
        .color(Color::from(theme::MODULE_FOREGROUND));
    let icon_width = modules::measured_width(&icon_text) + modules::PADDING;

    let content = Row::new(children![
        Rectangle::new()
            .width(icon_width)
            .height(24.0)
            .padding(Padding {
                top: 0.0,
                right: modules::PADDING,
                bottom: 0.0,
                left: 0.0,
            })
            .align_child(Center, Center)
            .child(icon_text),
        Text::new(label)
            .size(14.0)
            .color(Color::from(theme::MODULE_FOREGROUND)),
    ])
    .gap(modules::GAP)
    .align(Center);

    let width = modules::measured_width(&content) + modules::PADDING * 2.0;

    Rectangle::new()
        .width(width)
        .height(24.0)
        .fill(theme::MODULE_BACKGROUND)
        .radius(7.0)
        .padding(Padding {
            top: 0.0,
            right: modules::PADDING,
            bottom: 0.0,
            left: modules::PADDING,
        })
        .on_click(open_volume_panel)
        .on_scroll(change_volume)
        .align_child(Center, Center)
        .child(content)
}

fn open_volume_panel(_: Button) {
    crate::volume_panel::toggle();
}

fn change_volume(scroll: Scroll) {
    let current = i32::from(audio_level::levels().volume);
    let step = if scroll.y < 0.0 { 5 } else { -5 };
    let changed = (current + step).clamp(0, 100);

    audio_level::set_volume(changed as u8);
}
