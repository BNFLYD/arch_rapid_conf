use amane::{Center, Color, Column, DesktopApp, Image, Rectangle, Text, Weight, children};

use crate::theme;

/*
 * A tile of the start menu grid: one app with its icon on top and its name
 * below. It is only the content; the cell it lives in (its size, its click and
 * its future drag) belongs to the grid in menu.rs.
 */

// how big the icon is inside the 96 px cell
const ICON: f32 = 48.0;

pub fn view(app: &DesktopApp) -> Column {
    Column::new(children![
        icon(app),
        Text::new(app.name().to_string())
            .size(11.0)
            .color(Color::from(theme::MODULE_FOREGROUND)),
    ])
    .gap(6.0)
    .align(Center)
}

fn icon(app: &DesktopApp) -> Rectangle {
    let base = Rectangle::new().width(ICON).height(ICON);

    match app.icon_path() {
        Some(path) => base.fill(Image::contain(path)).clip(),
        // no icon: the initial takes its place
        None => base.align_child(Center, Center).child(
            Text::new(initial(app.name()))
                .size(28.0)
                .weight(Weight::Bold)
                .color(Color::from(theme::PANEL_MUTED)),
        ),
    }
}

fn initial(name: &str) -> String {
    name.chars()
        .next()
        .map(String::from)
        .unwrap_or_else(|| "?".to_string())
}

#[cfg(test)]
mod tests {
    use super::initial;

    #[test]
    fn takes_the_first_letter_of_the_name() {
        assert_eq!(initial("Firefox"), "F");
        assert_eq!(initial(""), "?");
    }
}
