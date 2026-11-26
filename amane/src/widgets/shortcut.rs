use std::path::PathBuf;

use amane::{Apps, Center, Color, Column, Image, Pointer, Rectangle, Service, Stack, Text, Widget};

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

    // the size label goes on top of the content, the content is never replaced
    let body = Column::new(content).gap(6.0).align(Center);

    let tile = Rectangle::new()
        .width(look.width)
        .height(look.height)
        .fill(theme::MODULE_BACKGROUND)
        .radius(theme::BAR_RADIUS)
        .align_child(Center, Center);

    let tile = match look.resizing {
        Some(size) => tile.child(Stack::new(vec![
            Box::new(body) as Box<dyn Widget>,
            Box::new(crate::widgets::size_label(size, look.width, look.height)),
        ])),
        None => tile.child(body),
    };

    let tile = if !look.editing {
        tile
    } else if look.deleting {
        tile.border(1.0, theme::DELETE_BORDER)
    } else {
        tile.border(1.0, theme::EDIT_BORDER)
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
