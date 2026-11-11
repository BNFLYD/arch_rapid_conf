pub const FONT: &str = "JetBrainsMono Nerd Font";
pub const RUNCAT_FONT: &str = "runcat";

// one piece: bar and menu share height, margin, radius and color
pub const BAR_HEIGHT: f32 = 44.0;
pub const SIDE_MARGIN: f32 = 7.0;
pub const BAR_RADIUS: f32 = 7.0;
// the air the block leaves above and below itself in menu mode
pub const MENU_MARGIN: f32 = 7.0;
// the bar window carries the air below it, so it can lift without resizing
pub const BAR_WINDOW_HEIGHT: f32 = BAR_HEIGHT + MENU_MARGIN;

pub const BAR_BACKGROUND: &str = "#0c0c0cb3";
/// solo si sway no declara ningún fondo legible
pub const BACKDROP_FALLBACK: &str = "#cfcfcf";
pub const MODULE_BACKGROUND: &str = "#0c0c0cb3";
pub const MODULE_FOREGROUND: &str = "#dad4c3";

pub const PANEL_FOREGROUND: &str = "#e8e2d3";
pub const PANEL_MUTED: &str = "#a39d90";
pub const PANEL_BORDER: &str = "#555046";
pub const PANEL_DANGER: &str = "#b95f61";
// los colores de los anillos, con los mismos umbrales que wayle
pub const PANEL_SUCCESS: &str = "#8ab879";
pub const PANEL_WARNING: &str = "#d9a441";

// los colores del launcher, taken del estilo de wofi
pub const LAUNCHER_BACKGROUND: &str = "#0c0c0cb3";
pub const LAUNCHER_SEARCH: &str = "#0c0c0c80";
pub const LAUNCHER_SELECTED: &str = "#2b2b2bb3";
