use std::process::Command;
use std::sync::{LazyLock, Mutex, MutexGuard};
use std::thread;
use std::time::Duration;

use amane::request_frame;

/*
 * The library asks pulse for a sink literally named "@DEFAULT_SINK@" and this
 * sound server (the one of wsl, over rdp) has no sink with that name: the read
 * comes back empty, the volume sits at 0% and the writes go nowhere.
 *
 * pactl resolves that name on its own, so the volume is read and written with
 * it. The widget reads a cache and never runs a command, and the commands run
 * on other threads, so a sound server that stops answering cannot freeze the
 * interface the way the library's own service does.
 */
#[derive(Clone, Copy, Default, PartialEq)]
pub struct Levels {
    pub volume: u8,
    pub muted: bool,
}

static CACHE: LazyLock<Mutex<Levels>> = LazyLock::new(|| Mutex::new(Levels::default()));

/// Keeps the cache in sync with the sound server.
pub fn listen() {
    thread::spawn(|| {
        loop {
            let levels = read();

            let changed = {
                let mut cache = lock(&CACHE);
                let changed = *cache != levels;

                *cache = levels;

                changed
            };

            // only a change redraws, so an idle sink costs nothing
            if changed {
                request_frame();
            }

            thread::sleep(Duration::from_secs(1));
        }
    });
}

/// The last known state of the default sink.
pub fn levels() -> Levels {
    *lock(&CACHE)
}

pub fn set_volume(volume: u8) {
    let volume = volume.min(100);

    // the bar answers right away, the server catches up in its own thread
    lock(&CACHE).volume = volume;
    request_frame();

    amane::spawn(&format!(
        "timeout 2 pactl set-sink-volume @DEFAULT_SINK@ {volume}%"
    ));
}

pub fn toggle_mute() {
    let muted = {
        let mut cache = lock(&CACHE);

        cache.muted = !cache.muted;
        cache.muted
    };

    request_frame();

    // pactl only takes 1, 0 or toggle here, not the yes and no it reports
    let state = if muted { "1" } else { "0" };

    amane::spawn(&format!(
        "timeout 2 pactl set-sink-mute @DEFAULT_SINK@ {state}"
    ));
}

fn read() -> Levels {
    Levels {
        volume: percent(&pactl("get-sink-volume")),
        muted: pactl("get-sink-mute").contains("yes"),
    }
}

fn pactl(command: &str) -> String {
    let output = Command::new("timeout")
        .arg("2")
        .arg("pactl")
        .arg(command)
        .arg("@DEFAULT_SINK@")
        .output();

    match output {
        Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
        Err(_) => String::new(),
    }
}

// "Volume: front-left: 65536 / 75% / -9.29 dB, ..." holds the level after the first slash
fn percent(text: &str) -> u8 {
    let Some((_, rest)) = text.split_once('/') else {
        return 0;
    };
    let Some((value, _)) = rest.split_once('%') else {
        return 0;
    };

    value.trim().parse::<u8>().unwrap_or(0).min(100)
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|error| error.into_inner())
}

#[cfg(test)]
mod tests {
    use super::percent;

    #[test]
    fn reads_the_percentage_of_a_volume() {
        assert_eq!(percent("Volume: front-left: 65536 / 75% / -9.29 dB"), 75);
        assert_eq!(percent("Volume: front-left: 65536 /  0% / -inf dB"), 0);
    }

    #[test]
    fn a_reply_without_a_percentage_is_zero() {
        assert_eq!(percent(""), 0);
        assert_eq!(percent("Mute: yes"), 0);
    }
}
