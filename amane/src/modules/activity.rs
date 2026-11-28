use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use amane::Service;

// A commit can take a moment to reach the widget, but drawing never waits for Git.
const REFRESH: Duration = Duration::from_secs(60);
const WEEKS: i64 = 53;

#[derive(Clone, Copy)]
pub struct Day {
    pub month: u32,
    pub count: u32,
}

pub struct Activity {
    pub days: Vec<Day>,
    pub total: u32,
}

impl Service for Activity {
    fn new() -> Self {
        Self {
            days: Vec::new(),
            total: 0,
        }
    }

    fn listen() {
        loop {
            // Scanning and Git stay outside the service lock: the menu can draw meanwhile.
            let (days, total) = collect();
            let mut activity = Self::write();

            activity.days = days;
            activity.total = total;

            drop(activity);
            thread::sleep(REFRESH);
        }
    }
}

fn collect() -> (Vec<Day>, u32) {
    let (start, mut days) = calendar(local_today());
    let repositories = repositories();
    let identities = identities(&repositories);
    let mut seen = HashSet::new();
    let mut total = 0;

    for repository in repositories {
        let Some(log) = output(
            Command::new("git")
                .arg("-C")
                .arg(&repository)
                .arg("log")
                .arg("--date=short")
                .arg("--format=%H%x09%ae%x09%an%x09%cd")
                .arg(format!("--since={}", start.iso()))
                // Local branches and a possible detached HEAD; no remote-only refs.
                .arg("HEAD")
                .arg("--branches")
                .arg("--"),
        ) else {
            continue;
        };

        total += apply_log(&log, &identities, start.days(), &mut days, &mut seen);
    }

    (days, total)
}

fn apply_log(
    log: &str,
    identities: &Identities,
    start: i64,
    days: &mut [Day],
    seen: &mut HashSet<String>,
) -> u32 {
    let mut added = 0;

    for line in log.lines() {
        let Some((hash, email, name, date)) = parse_log_line(line) else {
            continue;
        };

        if !identities.matches(name, email) {
            continue;
        }

        let Some(date) = parse_date(date) else {
            continue;
        };

        let index = date.days() - start;

        if index < 0 || index as usize >= days.len() {
            continue;
        }

        // The same commit can be reachable from several branches and clones.
        if !seen.insert(hash.to_string()) {
            continue;
        }

        days[index as usize].count += 1;
        added += 1;
    }

    added
}

fn parse_log_line(line: &str) -> Option<(&str, &str, &str, &str)> {
    let mut fields = line.split('\t');
    let (Some(hash), Some(email), Some(name), Some(date), None) = (
        fields.next(),
        fields.next(),
        fields.next(),
        fields.next(),
        fields.next(),
    ) else {
        return None;
    };

    if hash.is_empty() || (email.is_empty() && name.is_empty()) {
        return None;
    }

    Some((hash, email, name, date))
}

#[derive(Default)]
struct Identities {
    emails: HashSet<String>,
    names: HashSet<String>,
}

impl Identities {
    fn matches(&self, name: &str, email: &str) -> bool {
        self.emails.contains(&email.to_lowercase()) || self.names.contains(&name.to_lowercase())
    }
}

fn identities(repositories: &[PathBuf]) -> Identities {
    let mut identities = Identities::default();

    // Names also rescue old commits whose email was mistyped. No credentials are read.
    for key in ["name", "email"] {
        let key = format!("user.{key}");

        if let Some(config) = output(
            Command::new("git")
                .args(["config", "--global", "--get-all"])
                .arg(&key),
        ) {
            add_config(&mut identities, &key, &config);
        }

        for repository in repositories {
            if let Some(config) = output(
                Command::new("git")
                    .arg("-C")
                    .arg(repository)
                    .args(["config", "--get-all"])
                    .arg(&key),
            ) {
                add_config(&mut identities, &key, &config);
            }
        }
    }

    identities
}

fn add_config(identities: &mut Identities, key: &str, config: &str) {
    let values = config
        .lines()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_lowercase);

    if key == "user.email" {
        identities.emails.extend(values);
    } else {
        identities.names.extend(values);
    }
}

fn repositories() -> Vec<PathBuf> {
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        return Vec::new();
    };

    let mut found = Vec::new();

    // Projects is deep; .config is itself a repo and only needs shallow nesting.
    discover(&home.join("Projects"), 6, &mut found);
    discover(&home.join(".config"), 2, &mut found);

    found.sort();
    found.dedup();
    found
}

fn discover(path: &Path, depth: usize, found: &mut Vec<PathBuf>) {
    // A .git file also makes linked worktrees count as repositories.
    if path.join(".git").exists() {
        found.push(path.to_path_buf());
    }

    if depth == 0 {
        return;
    }

    let Ok(entries) = fs::read_dir(path) else {
        return;
    };

    for entry in entries.flatten() {
        let Ok(kind) = entry.file_type() else {
            continue;
        };

        if !kind.is_dir() {
            continue;
        }

        let name = entry.file_name();
        let name = name.to_string_lossy();

        // Dependencies, build output and hidden caches are not working repos.
        if name.starts_with('.')
            || matches!(name.as_ref(), "node_modules" | "target" | "dist" | "build")
        {
            continue;
        }

        discover(&entry.path(), depth - 1, found);
    }
}

fn output(command: &mut Command) -> Option<String> {
    let output = command.output().ok()?;

    if !output.status.success() {
        return None;
    }

    Some(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Date {
    year: i32,
    month: u32,
    day: u32,
}

impl Date {
    // Days since the Unix epoch, using the proleptic Gregorian calendar.
    fn days(self) -> i64 {
        let year = self.year as i64 - i64::from(self.month <= 2);
        let era = if year >= 0 { year } else { year - 399 } / 400;
        let year_of_era = year - era * 400;
        let month = self.month as i64;
        let shifted_month = month + if month > 2 { -3 } else { 9 };
        let day_of_year = (153 * shifted_month + 2) / 5 + self.day as i64 - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;

        era * 146_097 + day_of_era - 719_468
    }

    fn from_days(days: i64) -> Self {
        let shifted = days + 719_468;
        let era = if shifted >= 0 {
            shifted
        } else {
            shifted - 146_096
        } / 146_097;
        let day_of_era = shifted - era * 146_097;
        let year_of_era =
            (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
        let mut year = year_of_era + era * 400;
        let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let shifted_month = (5 * day_of_year + 2) / 153;
        let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
        let month = shifted_month + if shifted_month < 10 { 3 } else { -9 };

        year += i64::from(month <= 2);

        Self {
            year: year as i32,
            month: month as u32,
            day: day as u32,
        }
    }

    fn iso(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

fn parse_date(text: &str) -> Option<Date> {
    let mut fields = text.trim().split('-');
    let (Some(year), Some(month), Some(day), None) =
        (fields.next(), fields.next(), fields.next(), fields.next())
    else {
        return None;
    };

    let date = Date {
        year: year.parse().ok()?,
        month: month.parse().ok()?,
        day: day.parse().ok()?,
    };

    // Reject impossible dates such as 2025-02-30.
    (Date::from_days(date.days()) == date).then_some(date)
}

fn local_today() -> Date {
    if let Some(date) = output(Command::new("date").arg("+%Y-%m-%d"))
        .as_deref()
        .and_then(parse_date)
    {
        return date;
    }

    // UTC is only a fallback if the platform's date command is unavailable.
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    Date::from_days((seconds / 86_400) as i64)
}

fn calendar(today: Date) -> (Date, Vec<Day>) {
    let end = today.days();
    // 1970-01-01 was Thursday; Sunday is the first column's first row.
    let weekday = (end + 4).rem_euclid(7);
    let start = end - weekday - (WEEKS - 1) * 7;
    let first = Date::from_days(start);

    let days = (start..=end)
        .map(|day| Day {
            month: Date::from_days(day).month,
            count: 0,
        })
        .collect();

    (first, days)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_survive_leap_days_and_the_epoch() {
        for text in ["1970-01-01", "2024-02-29", "2026-04-01"] {
            let date = parse_date(text).unwrap();
            assert_eq!(Date::from_days(date.days()), date);
            assert_eq!(date.iso(), text);
        }

        assert_eq!(parse_date("1970-01-01").unwrap().days(), 0);
        assert_eq!(parse_date("2025-02-29"), None);
    }

    #[test]
    fn the_calendar_starts_on_sunday_and_ends_today() {
        let today = parse_date("2026-04-01").unwrap(); // Wednesday
        let (start, days) = calendar(today);

        assert_eq!(start.iso(), "2025-03-30");
        assert_eq!((start.days() + 4).rem_euclid(7), 0);
        assert_eq!(days.len(), 368);
        assert_eq!(days.last().unwrap().month, 4);
    }

    #[test]
    fn counts_own_commits_once_and_ignores_other_dates_and_authors() {
        let today = parse_date("2026-04-01").unwrap();
        let (start, mut days) = calendar(today);
        let identities = Identities {
            emails: HashSet::from([String::from("me@example.com")]),
            names: HashSet::from([String::from("mori-conatur")]),
        };
        let mut seen = HashSet::new();
        let before = Date::from_days(start.days() - 1).iso();
        let log = format!(
            "one\tme@example.com\tBNFLYD\t2026-04-01\n\
             two\tME@example.com\tBNFLYD\t2026-03-31\n\
             typo\twrong@example.com\tmori-conatur\t2026-04-01\n\
             other\tsomeone@example.com\tSomeone\t2026-04-01\n\
             old\tme@example.com\tBNFLYD\t{before}\n\
             broken\n"
        );

        assert_eq!(
            apply_log(&log, &identities, start.days(), &mut days, &mut seen),
            3
        );
        assert_eq!(days.last().unwrap().count, 2);

        // Another clone reports the same commit: it must not add another one.
        let duplicate = "one\tme@example.com\tBNFLYD\t2026-04-01\n";
        assert_eq!(
            apply_log(duplicate, &identities, start.days(), &mut days, &mut seen),
            0
        );
        assert_eq!(days.iter().map(|day| day.count).sum::<u32>(), 3);
    }
}
