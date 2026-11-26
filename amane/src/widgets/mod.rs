pub mod clock;
pub mod shortcut;
pub mod tile;

use amane::Rectangle;

use crate::home::Instance;

/*
 * The widgets the board can show. A widget is an entry of the board's state,
 * not an object that moves around: its view is a plain function that builds
 * the rectangle out of the instance and the size its cells give it.
 *
 * What each kind can do (title, description, offered sizes) comes with the
 * catalog of the editing mode, so it is not here yet.
 */

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Clock,
    Shortcut,
}

impl Kind {
    pub fn parse(word: &str) -> Option<Self> {
        match word {
            "clock" => Some(Kind::Clock),
            "shortcut" => Some(Kind::Shortcut),
            _ => None,
        }
    }

    // the word the layout file uses
    pub fn word(self) -> &'static str {
        match self {
            Kind::Clock => "clock",
            Kind::Shortcut => "shortcut",
        }
    }
}

/// How big a widget is drawn, and whether the board is being edited.
pub struct Look {
    pub width: f32,
    pub height: f32,
    pub editing: bool,
    // dragged onto the strip that deletes it
    pub deleting: bool,
}

pub struct Spec {
    pub view: fn(&Instance, &Look) -> Rectangle,
}

/// How a kind is drawn at a given size.
pub fn spec(kind: Kind) -> Spec {
    Spec {
        view: match kind {
            Kind::Clock => clock::view,
            Kind::Shortcut => shortcut::view,
        },
    }
}
