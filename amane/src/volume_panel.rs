use std::process::Command;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::Duration;

use amane::{
    Audio, Center, Color, Column, Horizontal, Key, Keyboard, Layer, LayerWindow, Margin, Monitor,
    Padding, Parent, Rectangle, Row, Service, Size, Text, Vertical, Weight, Widget, Zone, children,
};

use crate::theme;

// the same width as the launcher and the system panel
const WIDTH_RATIO: f32 = 0.52;
const PADDING: f32 = 16.0;
const GAP: f32 = 8.0;
const ROW_HEIGHT: f32 = 38.0;
// the same step the scroll of the bar uses
const STEP: i32 = 5;

pub struct VolumePanel {
    open: bool,
}

impl Service for VolumePanel {
    fn new() -> Self {
        Self { open: false }
    }

    fn listen() {}
}

/// The name of the devices, which the sound server changes on its own.
pub struct AudioDevices {
    sink: String,
    source: String,
    cache: Arc<Mutex<(String, String)>>,
}

impl Service for AudioDevices {
    fn new() -> Self {
        let cache = Arc::new(Mutex::new((String::new(), String::new())));
        let worker = Arc::clone(&cache);

        // pactl runs on its own thread: it can take a while or hang, and the
        // service lock must never be held while it works
        thread::spawn(move || {
            loop {
                let sink = pactl("get-default-sink");
                let source = pactl("get-default-source");

                *lock(&worker) = (sink, source);

                thread::sleep(Duration::from_secs(2));
            }
        });

        Self {
            sink: String::new(),
            source: String::new(),
            cache,
        }
    }

    fn interval() -> Duration {
        Duration::from_secs(2)
    }

    fn update(&mut self) -> bool {
        let (sink, source) = lock(&self.cache).clone();

        let changed = sink != self.sink || source != self.source;

        self.sink = sink;
        self.source = source;

        changed
    }
}

/// One playing stream, with its own volume (the "buffers" of pulseaudio).
#[derive(Clone, Debug, PartialEq)]
pub struct Stream {
    id: u32,
    name: String,
    role: String,
    volume: u8,
    muted: bool,
}

/// The streams that are playing right now.
pub struct Streams {
    streams: Vec<Stream>,
    cache: Arc<Mutex<Vec<Stream>>>,
}

impl Service for Streams {
    fn new() -> Self {
        let cache = Arc::new(Mutex::new(Vec::new()));
        let worker = Arc::clone(&cache);

        // pactl runs on its own thread: it can take a while or hang, and the
        // service lock must never be held while it works
        thread::spawn(move || {
            loop {
                let streams = parse_streams(&pactl_list("list sink-inputs"));

                *lock(&worker) = streams;

                thread::sleep(Duration::from_secs(1));
            }
        });

        Self {
            streams: Vec::new(),
            cache,
        }
    }

    fn interval() -> Duration {
        Duration::from_secs(1)
    }

    fn update(&mut self) -> bool {
        let streams = lock(&self.cache).clone();

        if streams == self.streams {
            return false;
        }

        self.streams = streams;

        true
    }
}

pub fn toggle() {
    // the read ends before the write: both on the same service would wait
    // for each other
    let is_open = VolumePanel::read().open;

    if is_open {
        close();
    } else {
        open();
    }
}

pub fn open() {
    VolumePanel::write().open = true;
}

pub fn close() {
    VolumePanel::write().open = false;
}

/// Lets a keybinding open the panel: `amane ipc call volume toggle`.
pub fn ipc(args: &[String]) -> String {
    match args.first().map(String::as_str) {
        Some("open") => {
            open();
            "abierto".to_string()
        }
        Some("close") => {
            close();
            "cerrado".to_string()
        }
        Some("state") => {
            if VolumePanel::read().open {
                "abierto".to_string()
            } else {
                "cerrado".to_string()
            }
        }
        _ => {
            toggle();
            "alternado".to_string()
        }
    }
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    let open = VolumePanel::read().open;
    let width = monitor.width as f32 * WIDTH_RATIO;

    // the window covers the whole screen, so a click outside the box closes it
    let child = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(Color::TRANSPARENT)
        .align_child(amane::Center, amane::Center)
        .on_click(|_| close())
        .child(content(width));

    LayerWindow::new()
        .width(monitor.width as f32)
        .height(monitor.height as f32)
        .anchor_vertical(Vertical::Top)
        .anchor_horizontal(Horizontal::Left)
        .margin(Margin {
            top: 0,
            right: 0,
            bottom: 0,
            left: 0,
        })
        .layer(Layer::Overlay)
        .keyboard(Keyboard::Exclusive)
        .space(Zone::Ignore)
        .namespace("amane-volume-panel")
        .visible(open)
        .on_key(|key| {
            if key == Key::Escape {
                close();
            }
        })
        .child(child)
}

fn content(width: f32) -> Rectangle {
    let audio = Audio::read();
    let devices = AudioDevices::read();
    let streams = Streams::read().streams.clone();

    let box_height = crate::system_panel::height();
    let bar_width = width - 36.0;

    // the streams are dropped one by one until everything fits in the box
    let mut shown = streams.clone();

    let column = loop {
        let items = build_items(&audio, &devices, &shown, bar_width);
        let column = Column::new(items).gap(GAP);

        let needed = match Widget::height(&column) {
            Size::Fixed(height) => height + PADDING * 2.0,
            _ => 0.0,
        };

        if needed <= box_height || shown.is_empty() {
            break column;
        }

        shown.pop();
    };

    Rectangle::new()
        .width(width)
        // the same height as the system panel, so the two can swap later
        .height(box_height)
        .fill(theme::LAUNCHER_BACKGROUND)
        .radius(14.0)
        .padding(Padding {
            top: PADDING,
            right: 18.0,
            bottom: PADDING,
            left: 18.0,
        })
        // the clicks inside the box do not close it
        .on_click(|_| {})
        // the content is shorter than the system panel, so it sits in the middle
        .align_child(Center, Center)
        .child(column)
}

fn build_items(
    audio: &Audio,
    devices: &AudioDevices,
    streams: &[Stream],
    bar_width: f32,
) -> Vec<Box<dyn Widget>> {
    let volume = audio.volume();
    let muted = audio.muted();

    let mut items: Vec<Box<dyn Widget>> = vec![
        Box::new(header()),
        Box::new(divider()),
        Box::new(section("ESTADO ACTUAL")),
        Box::new(readout(volume, muted)),
        Box::new(stat_row("Salida", device(&devices.sink))),
        Box::new(section("MASTER")),
        // the bar leaves room for the percentage and the two buttons
        Box::new(master_row(volume, muted, bar_width - 160.0)),
    ];

    // the streams are grouped by what they are: music, calls, and the rest
    let mut groups: Vec<(&'static str, Vec<&Stream>)> = Vec::new();

    for stream in streams {
        let group = group_of(&stream.role);

        match groups.iter_mut().find(|(name, _)| *name == group) {
            Some((_, list)) => list.push(stream),
            None => groups.push((group, vec![stream])),
        }
    }

    for (name, list) in groups {
        items.push(Box::new(section(name)));

        for stream in list {
            items.push(Box::new(stream_row(stream)));
        }
    }

    items.push(Box::new(actions()));

    items
}

fn header() -> Rectangle {
    Rectangle::new().width(Parent).height(34.0).child(
        Row::new(children![
            Text::new("VOLUMEN")
                .size(15.0)
                .weight(Weight::Bold)
                .color(Color::from(theme::PANEL_FOREGROUND)),
        ])
        .width(Parent)
        .justify(amane::Start)
        .align(Center),
    )
}

fn divider() -> Rectangle {
    Rectangle::new()
        .width(Parent)
        .height(1.0)
        .fill(theme::PANEL_BORDER)
        .opacity(0.5)
}

fn section(label: &str) -> Text {
    Text::new(label)
        .size(11.0)
        .weight(Weight::Bold)
        .color(Color::from(theme::PANEL_MUTED))
}

fn stat_row(label: &str, value: String) -> Row {
    Row::new(children![
        Text::new(label)
            .size(13.0)
            .color(Color::from(theme::PANEL_FOREGROUND)),
        Text::new(value)
            .size(13.0)
            .weight(Weight::Bold)
            .color(Color::from(theme::PANEL_FOREGROUND)),
    ])
    .width(Parent)
    .height(20.0)
    .justify(amane::SpaceBetween)
    .align(Center)
}

/// The icon follows the same levels as wayle: muted, low, medium and high.
fn icon(volume: u8, muted: bool) -> &'static str {
    if muted {
        "\u{eee8}"
    } else if volume == 0 {
        "\u{f026}"
    } else if volume < 50 {
        "\u{f027}"
    } else {
        "\u{f028}"
    }
}

fn readout(volume: u8, muted: bool) -> Row {
    let state = if muted {
        ("silenciado", theme::PANEL_DANGER)
    } else {
        ("sonando", theme::PANEL_SUCCESS)
    };

    Row::new(children![
        Text::new(icon(volume, muted))
            .size(32.0)
            .color(Color::from(theme::PANEL_FOREGROUND)),
        Text::new(format!("{volume}%"))
            .size(30.0)
            .weight(Weight::Bold)
            .color(Color::from(theme::PANEL_FOREGROUND)),
        Text::new(state.0)
            .size(12.0)
            .weight(Weight::Bold)
            .color(Color::from(state.1)),
    ])
    .width(Parent)
    .gap(14.0)
    .align(Center)
}

/// The bar that shows how loud it is.
fn level_bar(volume: u8, muted: bool, width: f32) -> Rectangle {
    let percent = if muted { 0.0 } else { f32::from(volume) };
    let filled = width * percent / 100.0;

    Rectangle::new()
        .width(width)
        .height(12.0)
        .fill(theme::MODULE_BACKGROUND)
        .radius(6.0)
        .child(
            Rectangle::new()
                .width(filled)
                .height(12.0)
                .fill(Color::from(theme::MODULE_FOREGROUND))
                .radius(6.0),
        )
}

/// The master volume, the one the bar shows.
fn master_row(volume: u8, muted: bool, bar_width: f32) -> Row {
    Row::new(children![
        Text::new(format!("{volume}%"))
            .size(14.0)
            .weight(Weight::Bold)
            .color(Color::from(theme::PANEL_FOREGROUND)),
        level_bar(volume, muted, bar_width),
        master_step("−", -STEP),
        master_step("+", STEP),
    ])
    .width(Parent)
    .height(ROW_HEIGHT)
    .gap(10.0)
    .align(Center)
}

/// One playing stream with its own volume.
fn stream_row(stream: &Stream) -> Row {
    let color = if stream.muted {
        theme::PANEL_DANGER
    } else {
        theme::PANEL_FOREGROUND
    };

    Row::new(children![
        Column::new(children![
            Text::new(&stream.name)
                .size(13.0)
                .weight(Weight::Bold)
                .color(Color::from(theme::PANEL_FOREGROUND)),
            Text::new(role_label(&stream.role))
                .size(10.0)
                .color(Color::from(theme::PANEL_MUTED)),
        ])
        .gap(2.0)
        .width(240.0),
        Text::new(format!("{}%", stream.volume))
            .size(13.0)
            .weight(Weight::Bold)
            .color(Color::from(color)),
        level_bar(stream.volume, stream.muted, 260.0),
        stream_step("−", stream.id, -STEP),
        stream_step("+", stream.id, STEP),
    ])
    .width(Parent)
    .height(ROW_HEIGHT)
    .gap(10.0)
    .align(Center)
}

fn actions() -> Row {
    Row::new(children![mute_button(), mixer_button()])
        .width(Parent)
        .height(38.0)
        .gap(GAP)
        .justify(amane::SpaceEvenly)
        .align(Center)
}

fn master_step(label: &'static str, step: i32) -> Rectangle {
    Rectangle::new()
        .width(40.0)
        .height(30.0)
        .fill(theme::MODULE_BACKGROUND)
        .radius(7.0)
        .align_child(Center, Center)
        .cursor(amane::Pointer)
        .on_click(move |_| {
            let current = i32::from(Audio::read().volume());
            let changed = (current + step).clamp(0, 100);

            Audio::set_volume(changed as u8);
        })
        .child(
            Text::new(label)
                .size(15.0)
                .weight(Weight::Bold)
                .color(Color::from(theme::MODULE_FOREGROUND)),
        )
}

fn stream_step(label: &'static str, id: u32, step: i32) -> Rectangle {
    Rectangle::new()
        .width(34.0)
        .height(30.0)
        .fill(theme::MODULE_BACKGROUND)
        .radius(7.0)
        .align_child(Center, Center)
        .cursor(amane::Pointer)
        .on_click(move |_| {
            let current = i32::from(stream_volume(id));
            let changed = (current + step).clamp(0, 100);

            amane::spawn(&format!("pactl set-sink-input-volume {id} {changed}%"));
        })
        .child(
            Text::new(label)
                .size(14.0)
                .weight(Weight::Bold)
                .color(Color::from(theme::MODULE_FOREGROUND)),
        )
}

fn mute_button() -> Rectangle {
    Rectangle::new()
        .width(150.0)
        .height(38.0)
        .fill(theme::MODULE_BACKGROUND)
        .radius(7.0)
        .align_child(Center, Center)
        .cursor(amane::Pointer)
        .on_click(|_| Audio::toggle_mute())
        .child(
            Text::new("Silenciar")
                .size(13.0)
                .weight(Weight::Bold)
                .color(Color::from(theme::MODULE_FOREGROUND)),
        )
}

fn mixer_button() -> Rectangle {
    Rectangle::new()
        .width(150.0)
        .height(38.0)
        .fill(theme::MODULE_BACKGROUND)
        .radius(7.0)
        .align_child(Center, Center)
        .cursor(amane::Pointer)
        .on_click(|_| {
            amane::spawn("pavucontrol");
        })
        .child(
            Text::new("Abrir mixer")
                .size(13.0)
                .weight(Weight::Bold)
                .color(Color::from(theme::MODULE_FOREGROUND)),
        )
}

fn device(name: &str) -> String {
    if name.is_empty() {
        String::from("Desconocida")
    } else {
        name.to_string()
    }
}

/// The group a stream belongs to, by its pulseaudio role.
fn group_of(role: &str) -> &'static str {
    match role {
        "music" | "video" | "movie" => "MULTIMEDIA",
        "phone" | "communication" => "COMUNICACIONES",
        _ => "OTRAS APLICACIONES",
    }
}

/// The role in words, so the user knows what each stream is.
fn role_label(role: &str) -> String {
    match role {
        "music" => String::from("música"),
        "video" | "movie" => String::from("video"),
        "phone" => String::from("llamada"),
        "communication" => String::from("comunicación"),
        "text" => String::from("texto"),
        "event" => String::from("evento"),
        "alarm" => String::from("alarma"),
        "" => String::from("sin rol"),
        other => other.to_string(),
    }
}

/// Reads the streams of `pactl list sink-inputs`.
fn parse_streams(output: &str) -> Vec<Stream> {
    output
        .split("Sink Input #")
        .skip(1)
        .filter_map(|block| {
            let id = block.lines().next()?.trim().parse().ok()?;
            let volume = percent_in(block)?;
            let role = property(block, "media.role").unwrap_or_default();
            let name = property(block, "application.name")
                .or_else(|| property(block, "media.name"))
                .unwrap_or_else(|| format!("Flujo {id}"));
            let muted = block
                .lines()
                .any(|line| line.trim_start().starts_with("Mute: yes"));

            Some(Stream {
                id,
                name,
                role,
                volume,
                muted,
            })
        })
        .collect()
}

/// The percentage of the first volume line of a block.
fn percent_in(block: &str) -> Option<u8> {
    let line = block
        .lines()
        .find(|line| line.trim_start().starts_with("Volume:"))?;

    let after = line.split_once('/')?.1;
    let number = after.split('%').next()?.trim();

    number.parse().ok()
}

/// Reads a property of a block, like `media.role = "music"`.
fn property(block: &str, name: &str) -> Option<String> {
    block.lines().find_map(|line| {
        let (key, value) = line.split_once('=')?;

        if key.trim() != name {
            return None;
        }

        Some(value.trim().trim_matches('"').to_string())
    })
}

/// The volume of a stream right now.
fn stream_volume(id: u32) -> u8 {
    percent_in(&pactl_list(&format!("get-sink-input-volume {id}"))).unwrap_or(0)
}

fn pactl(command: &str) -> String {
    pactl_list(command).trim().to_string()
}

/// A lock that does not die when a thread panics.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|error| error.into_inner())
}

fn pactl_list(command: &str) -> String {
    // timeout: if pactl gets stuck, the service thread gives up instead of
    // waiting forever
    Command::new("timeout")
        .args(["2", "pactl", command])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{group_of, parse_streams, role_label};

    const LIST: &str = "Sink Input #23\n\
\tDriver: protocol-native.c\n\
\tSink: 0\n\
\tMute: no\n\
\tVolume: front-left: 45875 / 70% / -9.29 dB,   front-right: 45875 / 70% / -9.29 dB\n\
\tProperties:\n\
\t\tmedia.role = \"music\"\n\
\t\tapplication.name = \"Chromium\"\n\
\n\
Sink Input #24\n\
\tMute: yes\n\
\tVolume: front-left: 32768 / 50% / -18.06 dB,   front-right: 32768 / 50% / -18.06 dB\n\
\tProperties:\n\
\t\tmedia.role = \"phone\"\n\
\t\tapplication.name = \"Discord\"\n";

    #[test]
    fn reads_the_streams_of_pulseaudio() {
        let streams = parse_streams(LIST);

        assert_eq!(streams.len(), 2);
        assert_eq!(streams[0].id, 23);
        assert_eq!(streams[0].name, "Chromium");
        assert_eq!(streams[0].role, "music");
        assert_eq!(streams[0].volume, 70);
        assert!(!streams[0].muted);
        assert_eq!(streams[1].volume, 50);
        assert!(streams[1].muted);
    }

    #[test]
    fn groups_the_streams_by_role() {
        assert_eq!(group_of("music"), "MULTIMEDIA");
        assert_eq!(group_of("video"), "MULTIMEDIA");
        assert_eq!(group_of("phone"), "COMUNICACIONES");
        assert_eq!(group_of("event"), "OTRAS APLICACIONES");
        assert_eq!(role_label("music"), "música");
    }
}
