mod bar;
mod modules;
mod system_panel;
mod theme;

use amane::App;

fn main() {
    App::new()
        .font(theme::FONT)
        .window(system_panel::view)
        .window_per_monitor(bar::view)
        .run();
}
