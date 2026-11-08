use std::process::Command;
use std::thread;
use std::time::Duration;

use amane::{
    Center, Color, Column, Horizontal, Key, Keyboard, Layer, LayerWindow, Margin, Monitor, Padding,
    Parent, Rectangle, Row, Service, Size, Text, TextInput, Vertical, Weight, Widget, Zone,
    children,
};

use crate::modules::network::NetworkStatus;
use crate::theme;

// the same width as the launcher and the system panel
const WIDTH_RATIO: f32 = 0.52;
const PADDING: f32 = 16.0;
const GAP: f32 = 8.0;
const ROW_HEIGHT: f32 = 38.0;
const PASSWORD_ID: &str = "amane-network-password";
// the panel has the same height as the system panel, so the list is capped
const MAX_NETWORKS: usize = 6;

/// A wifi network seen in the scan.
#[derive(Clone, Debug, PartialEq)]
struct Network {
    ssid: String,
    signal: u8,
    security: String,
    active: bool,
}

impl Network {
    fn secured(&self) -> bool {
        !self.security.is_empty() && self.security != "--"
    }
}

/// A connection the user asked for; the service thread runs it.
#[derive(Clone, Debug, PartialEq)]
struct Request {
    ssid: String,
    password: String,
}

pub struct NetworkPanel {
    open: bool,
    networks: Vec<Network>,
    selected: Option<usize>,
    password: String,
    message: String,
    error: bool,
    request: Option<Request>,
    rescan: bool,
}

impl Service for NetworkPanel {
    fn new() -> Self {
        Self {
            open: false,
            networks: Vec::new(),
            selected: None,
            password: String::new(),
            message: String::new(),
            error: false,
            request: None,
            rescan: false,
        }
    }

    fn listen() {
        let mut was_open = false;

        loop {
            let (open, rescan, request) = {
                let state = NetworkPanel::read();
                (state.open, state.rescan, state.request.clone())
            };

            // the user asked to refresh, or the panel just opened
            if rescan || (open && !was_open) {
                NetworkPanel::write().rescan = false;
                scan();
            }

            was_open = open;

            if let Some(request) = request {
                NetworkPanel::write().request = None;
                connect(&request);
            }

            thread::sleep(Duration::from_secs(1));
        }
    }
}

pub fn toggle() {
    // the read ends before the write: both on the same service would wait
    // for each other
    let is_open = NetworkPanel::read().open;

    if is_open {
        close();
    } else {
        open();
    }
}

pub fn open() {
    {
        let mut state = NetworkPanel::write();
        state.open = true;
        state.selected = None;
        state.password.clear();
        state.message.clear();
        state.error = false;
    }

    // the password field starts empty every time
    TextInput::set_text(PASSWORD_ID, "");
}

pub fn close() {
    let mut state = NetworkPanel::write();
    state.open = false;
    state.selected = None;
    state.password.clear();
}

/// Lets a keybinding open the panel: `amane ipc call network toggle`.
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
            if NetworkPanel::read().open {
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
    let state = NetworkPanel::read();
    let open = state.open;
    let width = monitor.width as f32 * WIDTH_RATIO;

    // the window covers the whole screen, so a click outside the box closes it
    let child = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(Color::TRANSPARENT)
        .align_child(amane::Center, amane::Center)
        .on_click(|_| close())
        .child(content(&state, width));

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
        .namespace("amane-network-panel")
        .visible(open)
        .on_key(|key| {
            if key == Key::Escape {
                close();
            }
        })
        .child(child)
}

fn content(state: &NetworkPanel, width: f32) -> Rectangle {
    let network = NetworkStatus::read();
    // the same height as the system panel, so the two can swap later
    let height = crate::system_panel::height();

    // the space that is left after everything else goes to the list
    let used = column_height(base_items(state, &network)) + PADDING * 2.0;
    let available = (height - used - password_space(state)).max(ROW_HEIGHT);
    let rows = ((available + GAP) / (ROW_HEIGHT + GAP)).floor().max(1.0) as usize;

    let mut items = base_items(state, &network);

    if state.networks.is_empty() {
        items.push(Box::new(empty_message()));
    } else {
        items.push(Box::new(network_grid(state, width, rows)));
    }

    if let Some(index) = state.selected {
        if let Some(network) = state.networks.get(index) {
            if network.secured() {
                items.push(Box::new(section(&format!("CONTRASEÑA · {}", network.ssid))));
                items.push(Box::new(password_row()));
            }
        }
    }

    let column = Column::new(items).gap(GAP);

    Rectangle::new()
        .width(width)
        .height(height)
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
        .child(column)
}

/// Everything in the panel except the list of networks.
fn base_items(state: &NetworkPanel, network: &NetworkStatus) -> Vec<Box<dyn Widget>> {
    let mut items: Vec<Box<dyn Widget>> = vec![
        Box::new(header()),
        Box::new(divider()),
        Box::new(section("ESTADO ACTUAL")),
        Box::new(stat_row("Conexión", connection(network))),
        Box::new(stat_row("Descarga", network.download_rate())),
    ];

    if !state.message.is_empty() {
        items.push(Box::new(message(&state.message, state.error)));
    }

    items.push(Box::new(section("REDES WI-FI")));

    items
}

/// The room that the password field takes when a secured network is picked.
fn password_space(state: &NetworkPanel) -> f32 {
    let secured = state
        .selected
        .and_then(|index| state.networks.get(index))
        .map(Network::secured)
        .unwrap_or(false);

    if secured {
        // the section label and the field
        15.0 + ROW_HEIGHT + GAP
    } else {
        0.0
    }
}

fn column_height(items: Vec<Box<dyn Widget>>) -> f32 {
    match Widget::height(&Column::new(items).gap(GAP)) {
        Size::Fixed(height) => height,
        _ => 0.0,
    }
}

fn header() -> Rectangle {
    Rectangle::new().width(Parent).height(34.0).child(
        Row::new(children![
            Text::new("CONEXIONES DE RED")
                .size(15.0)
                .weight(Weight::Bold)
                .color(Color::from(theme::PANEL_FOREGROUND)),
            refresh_button(),
        ])
        .width(Parent)
        .justify(amane::SpaceBetween)
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

fn message(text: &str, error: bool) -> Rectangle {
    let color = if error {
        theme::PANEL_DANGER
    } else {
        theme::PANEL_SUCCESS
    };

    Rectangle::new()
        .width(Parent)
        .height(30.0)
        .fill(Color::from(theme::MODULE_BACKGROUND))
        .radius(7.0)
        .padding(Padding {
            top: 0.0,
            right: 10.0,
            bottom: 0.0,
            left: 10.0,
        })
        .align_child(amane::Start, Center)
        .child(
            Text::new(text)
                .size(12.0)
                .weight(Weight::Bold)
                .color(Color::from(color)),
        )
}

fn refresh_button() -> Rectangle {
    Rectangle::new()
        .width(88.0)
        .height(26.0)
        .fill(theme::MODULE_BACKGROUND)
        .radius(7.0)
        .align_child(Center, Center)
        .cursor(amane::Pointer)
        .on_click(|_| {
            NetworkPanel::write().rescan = true;
        })
        .child(
            Text::new("⟳  Refrescar")
                .size(11.0)
                .weight(Weight::Bold)
                .color(Color::from(theme::MODULE_FOREGROUND)),
        )
}

fn empty_message() -> Rectangle {
    Rectangle::new()
        .width(Parent)
        .height(40.0)
        .align_child(amane::Start, Center)
        .child(
            Text::new("No hay redes Wi-Fi a la vista")
                .size(12.0)
                .color(Color::from(theme::PANEL_MUTED)),
        )
}

/// The networks in two columns, so they fit in the height of the system panel.
fn network_grid(state: &NetworkPanel, width: f32, rows: usize) -> Row {
    let column_width = (width - 36.0 - GAP) / 2.0;
    let mut left: Vec<Box<dyn Widget>> = Vec::new();
    let mut right: Vec<Box<dyn Widget>> = Vec::new();

    let count = (rows * 2).min(MAX_NETWORKS);

    for (index, network) in state.networks.iter().enumerate().take(count) {
        let row = network_row(network, index, state.selected == Some(index), column_width);

        if index % 2 == 0 {
            left.push(Box::new(row));
        } else {
            right.push(Box::new(row));
        }
    }

    Row::new(children![
        Column::new(left).width(column_width).gap(GAP),
        Column::new(right).width(column_width).gap(GAP),
    ])
    .width(Parent)
    .gap(GAP)
}

fn network_row(network: &Network, index: usize, selected: bool, width: f32) -> Rectangle {
    let background = if selected {
        Color::from(theme::LAUNCHER_SELECTED)
    } else {
        Color::TRANSPARENT
    };

    let ssid = network.ssid.clone();
    let secured = network.secured();

    let mut labels: Vec<Box<dyn Widget>> = vec![Box::new(
        Text::new(&network.ssid)
            .size(14.0)
            .weight(Weight::Bold)
            .color(Color::from(theme::MODULE_FOREGROUND)),
    )];

    if network.active {
        labels.push(Box::new(
            Text::new("en uso")
                .size(11.0)
                .color(Color::from(theme::PANEL_SUCCESS)),
        ));
    }

    let security = if network.secured() {
        network.security.clone()
    } else {
        String::from("abierta")
    };

    Rectangle::new()
        .width(width)
        .height(ROW_HEIGHT)
        .fill(background)
        .radius(7.0)
        .padding(Padding {
            top: 0.0,
            right: 10.0,
            bottom: 0.0,
            left: 10.0,
        })
        .align_child(amane::Start, Center)
        .cursor(amane::Pointer)
        .on_click(move |_| {
            if secured {
                select(index);
            } else {
                ask_connection(ssid.clone(), String::new());
            }
        })
        .child(
            Row::new(children![
                Column::new(labels).gap(2.0).width(Parent),
                Text::new(format!("{}%", network.signal))
                    .size(13.0)
                    .weight(Weight::Bold)
                    .color(Color::from(theme::PANEL_FOREGROUND)),
                Text::new(security)
                    .size(11.0)
                    .color(Color::from(theme::PANEL_MUTED)),
            ])
            .width(Parent)
            .gap(12.0)
            .align(Center),
        )
}

fn password_row() -> Row {
    Row::new(children![
        Rectangle::new()
            .width(Parent)
            .height(ROW_HEIGHT)
            .fill(theme::LAUNCHER_SEARCH)
            .radius(7.0)
            .padding(Padding {
                top: 0.0,
                right: 12.0,
                bottom: 0.0,
                left: 12.0,
            })
            .align_child(amane::Start, Center)
            .child(
                TextInput::new(PASSWORD_ID)
                    .width(Parent)
                    .size(14.0)
                    .color(Color::from(theme::MODULE_FOREGROUND))
                    .placeholder("Contraseña…")
                    .password()
                    .focused()
                    .on_change(|text| {
                        NetworkPanel::write().password = text;
                    })
                    .on_submit(|_| connect_selected()),
            ),
        connect_button(),
    ])
    .width(Parent)
    .gap(GAP)
    .align(Center)
}

fn connect_button() -> Rectangle {
    Rectangle::new()
        .width(110.0)
        .height(ROW_HEIGHT)
        .fill(theme::MODULE_BACKGROUND)
        .radius(7.0)
        .align_child(Center, Center)
        .cursor(amane::Pointer)
        .on_click(|_| connect_selected())
        .child(
            Text::new("Conectarse")
                .size(12.0)
                .weight(Weight::Bold)
                .color(Color::from(theme::MODULE_FOREGROUND)),
        )
}

fn select(index: usize) {
    let mut state = NetworkPanel::write();
    state.selected = Some(index);
    state.password.clear();
    state.message.clear();
    state.error = false;

    TextInput::set_text(PASSWORD_ID, "");
}

fn connect_selected() {
    let (ssid, password) = {
        let state = NetworkPanel::read();
        let ssid = state
            .selected
            .and_then(|index| state.networks.get(index))
            .map(|network| network.ssid.clone());

        (ssid, state.password.clone())
    };

    let Some(ssid) = ssid else {
        return;
    };

    ask_connection(ssid, password);
}

fn ask_connection(ssid: String, password: String) {
    let mut state = NetworkPanel::write();
    state.request = Some(Request { ssid, password });
}

/// Looks for the networks around and saves the list.
fn scan() {
    let output = Command::new("timeout")
        .args([
            "5",
            "nmcli",
            "-t",
            "-f",
            "IN-USE,SSID,SIGNAL,SECURITY",
            "device",
            "wifi",
            "list",
        ])
        .output();

    let mut state = NetworkPanel::write();

    match output {
        Ok(output) if output.status.success() => {
            state.networks = parse_networks(&String::from_utf8_lossy(&output.stdout));
            state.selected = None;
            state.error = false;
            state.message = if state.networks.is_empty() {
                String::from("No se encontraron redes Wi-Fi")
            } else {
                String::new()
            };
        }
        Ok(output) => {
            state.error = true;
            state.message = clean_error(&String::from_utf8_lossy(&output.stderr));
        }
        Err(_) => {
            state.error = true;
            state.message = String::from("NetworkManager (nmcli) no está instalado");
        }
    }
}

/// Connects to a network and tells the user how it went.
fn connect(request: &Request) {
    let mut command = Command::new("timeout");
    command.args(["10", "nmcli", "device", "wifi", "connect", &request.ssid]);

    if !request.password.is_empty() {
        command.args(["password", &request.password]);
    }

    // the command runs before the lock is taken, so the panel does not freeze
    let result = command.output();
    let mut state = NetworkPanel::write();

    match result {
        Ok(output) if output.status.success() => {
            state.message = format!("Conectado a {}", request.ssid);
            state.error = false;
            state.open = false;
        }
        Ok(output) => {
            state.error = true;
            state.message = clean_error(&String::from_utf8_lossy(&output.stderr));
        }
        Err(_) => {
            state.error = true;
            state.message = String::from("NetworkManager (nmcli) no está instalado");
        }
    }
}

fn connection(network: &NetworkStatus) -> String {
    if !network.connected() {
        return String::from("Sin conexión");
    }

    let kind = if network.wireless() {
        "Wi-Fi"
    } else {
        "Ethernet"
    };

    match network.interface() {
        Some(interface) => format!("{kind} · {interface}"),
        None => kind.to_string(),
    }
}

/// Reads the terse output of nmcli: `*:SSID:SIGNAL:SECURITY`.
fn parse_networks(output: &str) -> Vec<Network> {
    output
        .lines()
        .filter_map(|line| {
            let fields = split_escaped(line, ':');

            if fields.len() < 4 {
                return None;
            }

            let ssid = fields[1].trim().to_string();

            if ssid.is_empty() {
                return None;
            }

            Some(Network {
                active: fields[0].trim() == "*",
                ssid,
                signal: fields[2].trim().parse().unwrap_or(0),
                security: fields[3].trim().to_string(),
            })
        })
        .collect()
}

/// Splits on the separator, keeping the escaped ones inside the field.
fn split_escaped(line: &str, separator: char) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut escaped = false;

    for character in line.chars() {
        if escaped {
            current.push(character);
            escaped = false;
            continue;
        }

        match character {
            '\\' => escaped = true,
            _ if character == separator => {
                fields.push(std::mem::take(&mut current));
            }
            _ => current.push(character),
        }
    }

    fields.push(current);
    fields
}

/// Takes the useful part out of the error of nmcli.
fn clean_error(output: &str) -> String {
    let line = output
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("No se pudo conectar")
        .trim();

    line.strip_prefix("Error: ").unwrap_or(line).to_string()
}

#[cfg(test)]
mod tests {
    use super::{clean_error, parse_networks, split_escaped};

    #[test]
    fn reads_the_networks_of_nmcli() {
        let networks = parse_networks("*:MiWifi:78:WPA2\n:Cafe:55:WPA2\n:Abierta:40:--\n");

        assert_eq!(networks.len(), 3);
        assert!(networks[0].active);
        assert_eq!(networks[0].ssid, "MiWifi");
        assert_eq!(networks[0].signal, 78);
        assert_eq!(networks[0].security, "WPA2");
        assert_eq!(networks[2].security, "--");
    }

    #[test]
    fn keeps_the_colons_escaped_inside_the_ssid() {
        // nmcli escapes the colons of the ssid like this
        let fields = split_escaped("*:Mi\\:Red:78:WPA2", ':');

        assert_eq!(fields[1], "Mi:Red");
        assert_eq!(fields[2], "78");
    }

    #[test]
    fn takes_the_last_line_of_an_error() {
        assert_eq!(
            clean_error("Error: no se pudo\nError: Contraseña incorrecta\n"),
            "Contraseña incorrecta"
        );
    }
}
