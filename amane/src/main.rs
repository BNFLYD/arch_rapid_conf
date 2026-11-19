mod app_launcher;
mod audio_level;
mod bar;
mod canvas;
mod menu;
mod modules;
mod network_panel;
mod system_panel;
mod theme;
mod usage;
mod volume_panel;
mod wallpaper;
mod widgets;

use amane::App;

fn main() {
    // keeps the volume of the sink up to date with the sound server
    audio_level::listen();

    App::new()
        .font(theme::FONT)
        .window_per_monitor(system_panel::view)
        .window_per_monitor(app_launcher::view)
        // after the launcher, so the password field takes the keyboard focus
        .window_per_monitor(network_panel::view)
        .window_per_monitor(volume_panel::view)
        // the canvas comes first: the backdrop of the menu. the menu is mapped
        // before the bar, so the bar stays above it
        .window_per_monitor(canvas::view)
        .window_per_monitor(bar::start_menu)
        .window_per_monitor(bar::view)
        .ipc("launcher", app_launcher::ipc)
        .ipc("panel", system_panel::ipc)
        .ipc("network", network_panel::ipc)
        .ipc("volume", volume_panel::ipc)
        .ipc("start", bar::menu_ipc)
        .run();
}
