pub mod clock;
pub mod heatmap;
pub mod shortcut;
pub mod tile;

use amane::{Color, End, Padding, Rectangle, Text};

use crate::home::Instance;
use crate::theme;

/*
 * The widgets the board can show. A widget is an entry of the board's state,
 * not an object that moves around: its view is a plain function that builds
 * the rectangle out of the instance and the size its cells give it.
 *
 * Each kind also says how it is offered in the catalog of the editing mode:
 * its name, a short description, an icon and the sizes it accepts.
 */

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Clock,
    Heatmap,
    Shortcut,
}

impl Kind {
    pub fn parse(word: &str) -> Option<Self> {
        match word {
            "clock" => Some(Kind::Clock),
            "heatmap" => Some(Kind::Heatmap),
            "shortcut" => Some(Kind::Shortcut),
            _ => None,
        }
    }

    // the word the layout file uses
    pub fn word(self) -> &'static str {
        match self {
            Kind::Clock => "clock",
            Kind::Heatmap => "heatmap",
            Kind::Shortcut => "shortcut",
        }
    }
}

/// How big a widget is drawn, and what the editing mode is doing to it.
pub struct Look {
    pub width: f32,
    pub height: f32,
    pub editing: bool,
    // about to be deleted: its outline goes red
    pub deleting: bool,
    // while it is being resized, the size it has right now
    pub resizing: Option<(usize, usize)>,
}

pub struct Spec {
    pub kind: Kind,
    pub title: &'static str,
    pub description: &'static str,
    // a nerd font glyph, shown in the catalog
    pub icon: &'static str,
    pub default_size: (usize, usize),
    pub min_size: (usize, usize),
    // none means as big as the board allows
    pub max_size: Option<(usize, usize)>,
    pub view: fn(&Instance, &Look) -> Rectangle,
}

/// The current size, drawn over the widget while it is being resized. It is a
/// sibling of the content, never a replacement for it.
pub fn size_label(size: (usize, usize), width: f32, height: f32) -> Rectangle {
    Rectangle::new()
        .width(width)
        .height(height)
        .align_child(End, End)
        .padding(Padding {
            top: 0.0,
            right: 8.0,
            bottom: 4.0,
            left: 0.0,
        })
        .child(
            Text::new(format!("{}×{}", size.0, size.1))
                .size(9.0)
                .color(Color::from(theme::SIZE_LABEL)),
        )
}

/// Everything the board can add, in the order the catalog shows it.
pub const SPECS: &[Spec] = &[
    Spec {
        kind: Kind::Clock,
        title: "Reloj",
        description: "La hora y la fecha",
        icon: "\u{f017}",
        // the clock always shows the time and the date, so 2x1 is its floor
        default_size: (2, 1),
        min_size: (2, 1),
        max_size: None,
        view: clock::view,
    },
    Spec {
        kind: Kind::Heatmap,
        title: "Actividad Git",
        description: "Tus commits locales, sin importar el remoto",
        icon: "\u{f1d3}",
        default_size: (7, 2),
        min_size: (7, 2),
        max_size: None,
        view: heatmap::view,
    },
    Spec {
        kind: Kind::Shortcut,
        title: "Acceso directo",
        description: "Abre una app o un comando",
        icon: "\u{f0c1}",
        default_size: (1, 1),
        min_size: (1, 1),
        max_size: Some((2, 2)),
        view: shortcut::view,
    },
];

/// How a kind is drawn at a given size, and what it offers.
pub fn spec(kind: Kind) -> &'static Spec {
    SPECS
        .iter()
        .find(|spec| spec.kind == kind)
        .expect("every kind has a spec")
}

pub fn specs() -> &'static [Spec] {
    SPECS
}
