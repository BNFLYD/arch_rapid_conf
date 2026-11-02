mod bar;
mod modules;
mod theme;

use amane::App;

fn main() {
    App::new()
        .font(theme::FONT)
        .window_per_monitor(bar::view)
        .run();
}
