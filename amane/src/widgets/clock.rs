use amane::{Center, Color, Column, Rectangle, Text, Weight, Widget};

use crate::home::Instance;
use crate::modules::clock;
use crate::theme;
use crate::widgets::Look;

// the clock widget: the time, and the date under it
pub fn view(_instance: &Instance, look: &Look) -> Rectangle {
    let labels: Vec<Box<dyn Widget>> = vec![
        Box::new(
            Text::new(clock::time())
                .size(30.0)
                .weight(Weight::Bold)
                .color(Color::from(theme::MODULE_FOREGROUND)),
        ),
        Box::new(
            Text::new(clock::date())
                .size(11.0)
                .color(Color::from(theme::PANEL_MUTED)),
        ),
    ];

    // a click always ends a hold, even though the clock does nothing else
    let tile = Rectangle::new()
        .width(look.width)
        .height(look.height)
        .fill(background(look))
        .radius(theme::BAR_RADIUS)
        .align_child(Center, Center)
        .on_click(|_| {
            crate::home::clicked();
        })
        .child(Column::new(labels).gap(2.0).align(Center));

    if look.editing {
        tile.border(1.0, theme::EDIT_BORDER)
    } else {
        tile
    }
}

fn background(look: &Look) -> Color {
    if look.deleting {
        Color::from(theme::TRASH_BACKGROUND)
    } else {
        Color::from(theme::MODULE_BACKGROUND)
    }
}
