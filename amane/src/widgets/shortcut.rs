use std::path::PathBuf;

use amane::{Apps, Center, Color, Column, Image, Pointer, Rectangle, Service, Text, Widget};

use crate::home::Instance;
use crate::theme;
use crate::widgets::Look;

// how big the icon is, whichever cells the shortcut covers
const ICON: f32 = 40.0;

// a shortcut to an app or a command: its icon and name, and a click that
// starts it and closes the menu
pub fn view(instance: &Instance, look: &Look) -> Rectangle {
    let name = instance.options.get("name").cloned().unwrap_or_default();
    let exec = instance.options.get("exec").cloned().unwrap_or_default();

    // the icon of the app with that name, when the app list knows it
    let icon = Apps::read()
        .list()
        .iter()
        .find(|app| app.name() == name)
        .and_then(|app| app.icon_path().map(PathBuf::from));

    let content: Vec<Box<dyn Widget>> = vec![
        Box::new(icon_or_initial(icon, &name)),
        Box::new(
            Text::new(name)
                .size(11.0)
                .color(Color::from(theme::MODULE_FOREGROUND)),
        ),
    ];

    let tile = Rectangle::new()
        .width(look.width)
        .height(look.height)
        .fill(background(look))
        .radius(theme::BAR_RADIUS)
        .align_child(Center, Center)
        .child(Column::new(content).gap(6.0).align(Center));

    let tile = if look.editing {
        tile.border(1.0, theme::EDIT_BORDER)
    } else {
        tile
    };

    // the click stays in both modes, so the pointer targets do not change when
    // the board enters the editing mode; what it does depends on the mode
    tile.cursor(Pointer).on_click(move |_| {
        if crate::home::clicked() {
            return;
        }

        // while the board is edited the widget only moves
        if crate::home::editing() {
            return;
        }

        crate::bar::menu_close();
        amane::spawn(&exec);
    })
}

fn background(look: &Look) -> Color {
    if look.deleting {
        Color::from(theme::TRASH_BACKGROUND)
    } else {
        Color::from(theme::MODULE_BACKGROUND)
    }
}

fn icon_or_initial(icon: Option<PathBuf>, name: &str) -> Rectangle {
    let base = Rectangle::new().width(ICON).height(ICON);

    match icon {
        Some(path) => base.fill(Image::contain(path)).clip(),
        // no icon for that name: the initial takes its place
        None => base.align_child(Center, Center).child(
            Text::new(initial(name))
                .size(22.0)
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
