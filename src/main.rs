extern crate chrono;
extern crate dirs;
extern crate serde;
extern crate storaget;

use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::{env, io::Write};
use storaget::*;

pub struct Error(String);
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
type EventResult<T> = Result<T, Error>;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Event {
    description: String,
    date: Date,
    kind: EventKind,
}

impl Event {
    pub fn new(description: String, date: Date, kind: EventKind) -> Self {
        Event {
            description,
            date,
            kind,
        }
    }
    pub fn get_description(&self) -> &str {
        &self.description
    }
    pub fn get_date(&self) -> Date {
        self.date.clone()
    }
    pub fn try_date(&self) -> EventResult<NaiveDate> {
        let (month, day) = match self.date {
            Date::Full(date) => (date.month(), date.day()),
            Date::Trunc { month, day } => (month, day),
        };
        match try_date_from_md(month, day, 3) {
            Some(date) => Ok(date),
            None => Err(Error(format!(
                "Wrong date format for {} # {}-{}-{}",
                self.description,
                Utc::today().year(),
                month,
                day
            ))),
        }
    }
    pub fn print(&self) {
        let mut today_sign = String::new();
        let date = match self.try_date() {
            Ok(date) => {
                if date == Utc::today().naive_local() {
                    today_sign = "*** => ".to_string();
                }
                date.to_string()
            }
            Err(_) => "___NONE___".to_string(),
        };
        println!("{}{}\t{} {}", today_sign, date, self.description, self.kind);
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Date {
    Full(NaiveDate),
    Trunc { month: u32, day: u32 },
}

impl Default for Date {
    fn default() -> Self {
        Date::Full(Utc::today().naive_local())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EventKind {
    Birthday,
    Namesday,
    Other,
}

impl std::fmt::Display for EventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EventKind::Birthday => write!(f, "{}", "🎀"),
            EventKind::Namesday => write!(f, "{}", "🎉"),
            EventKind::Other => write!(f, "{}", ""),
        }
    }
}

impl Default for EventKind {
    fn default() -> Self {
        EventKind::Other
    }
}

fn try_date_from_md(month: u32, day: u32, count: u32) -> Option<NaiveDate> {
    if month == 0 || day == 0 {
        return None;
    }
    let date = date_from_md(month, day);
    if let Some(date) = date {
        return Some(date);
    } else if count == 0 {
        return date;
    }
    try_date_from_md(month, day - 1, count - 1)
}

fn date_from_md(month: u32, day: u32) -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt(Utc::today().year(), month, day)
}

fn date_from_ymd(year: i32, month: u32, day: u32) -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt(year, month, day)
}

fn process_date_str(date_str: &str) -> Option<Date> {
    let mut _parts: Vec<&str> = date_str.split("-").collect();
    let parts = _parts
        .iter()
        .map(|p| p.replace("\n", ""))
        .collect::<Vec<String>>();
    match parts.len() {
        2 => Some(Date::Trunc {
            month: parts[0]
                .parse::<u32>()
                .expect("Month format is wrong. Expect 05 or 12"),
            day: parts[1]
                .parse::<u32>()
                .expect("Day format is wrong. Expect 05 or 15"),
        }),
        3 => Some(Date::Full(
            date_from_ymd(
                parts[0].parse::<i32>().expect("Year format is wrong"),
                parts[1].parse::<u32>().expect("Month format is wrong"),
                parts[2].parse::<u32>().expect("Day format is wrong"),
            )
            .expect("Invalid date, the given date format is valid, but the given date not exist"),
        )),
        _ => None,
    }
}

fn print(msg: &str, events: Vec<&Event>) {
    println!("{}\n============================================", msg);
    if events.len() == 0 {
        println!("There is no event to display");
    } else {
        events.iter().for_each(|e: &&Event| e.print());
    }
}

pub fn cmd_read(text: &str, buffer: &mut String) {
    print!("{}: ", text);
    std::io::stdout().flush().unwrap();
    std::io::stdin()
        .read_line(buffer)
        .expect("Error while reading input");
}

pub fn clean_str(text: &mut String) {
    *text = text.replace("\n", "");
}

fn main() -> PackResult<()> {
    let mut events: Pack<Vec<Event>> = Pack::load_or_init(
        dirs::home_dir().expect("Error while getting your home folder"),
        ".event",
    )?;
    let mut events_ordered: Vec<&Event> = events.iter().collect();
    events_ordered.sort_by(|a, b| {
        if let Ok(_a) = a.try_date() {
            if let Ok(_b) = b.try_date() {
                return _a.cmp(&_b);
            }
        }
        std::cmp::Ordering::Equal
    });
    let args = env::args().collect::<Vec<String>>();
    if args.len() == 1 {
        print(
            "Comming events",
            events_ordered
                .iter()
                .filter(|e: &&&Event| {
                    if let Ok(date) = e.try_date() {
                        if date >= Utc::today().naive_local() {
                            return true;
                        }
                    }
                    false
                })
                .map(|e| *e)
                .collect::<Vec<&Event>>(),
        );
    }
    if args.len() == 2 {
        if args[1] == "all" {
            print(
                "All events",
                events_ordered.iter().map(|e| *e).collect::<Vec<&Event>>(),
            );
        } else if args[1] == "today" {
            print(
                "Today events",
                events_ordered
                    .iter()
                    .map(|e| *e)
                    .filter(|e: &&Event| {
                        if let Ok(date) = e.try_date() {
                            if date == Utc::today().naive_local() {
                                return true;
                            }
                        }
                        false
                    })
                    .collect::<Vec<&Event>>(),
            )
        } else if args[1] == "month" {
            print(
                "This month",
                events_ordered
                    .iter()
                    .map(|e| *e)
                    .filter(|e: &&Event| {
                        if let Ok(date) = e.try_date() {
                            if date >= Utc::today().naive_local()
                                && date.month() == Utc::today().naive_local().month()
                            {
                                return true;
                            }
                        }
                        false
                    })
                    .collect::<Vec<&Event>>(),
            )
        } else if args[1] == "week" {
            print(
                "This week",
                events_ordered
                    .iter()
                    .map(|e| *e)
                    .filter(|e: &&Event| {
                        if let Ok(date) = e.try_date() {
                            if date >= Utc::today().naive_local()
                                && (date.ordinal() as f32 / 7.0).ceil() as u32
                                    == (Utc::today().naive_local().ordinal() as f32 / 7.0).ceil()
                                        as u32
                            {
                                return true;
                            }
                        }
                        false
                    })
                    .collect::<Vec<&Event>>(),
            )
        } else if args[1] == "30" {
            print(
                "Next 30 days",
                events_ordered
                    .iter()
                    .map(|e| *e)
                    .filter(|e: &&Event| {
                        if let Ok(date) = e.try_date() {
                            if date >= Utc::today().naive_local()
                                && (date.ordinal() - Utc::today().naive_local().ordinal()) <= 30
                            {
                                return true;
                            }
                        }
                        false
                    })
                    .collect::<Vec<&Event>>(),
            )
        } else if args[1] == "add" {
            let mut date = String::new();
            let mut desc = String::new();
            let mut kind = String::new();
            cmd_read("Date (2020-04-20 / 04-20)", &mut date);
            cmd_read("Description", &mut desc);
            cmd_read("Event kind (0 Birthday, 1 Namesday, 2 Other)", &mut kind);
            clean_str(&mut date);
            clean_str(&mut desc);
            clean_str(&mut kind);
            events.as_mut().push(Event::new(
                desc,
                process_date_str(&date).expect("Wrong date!"),
                match kind.as_str() {
                    "0" => EventKind::Birthday,
                    "1" => EventKind::Namesday,
                    "2" => EventKind::Other,
                    _ => panic!("Out of range event!"),
                },
            ));
            println!("Ok!");
        } else {
            println!("Unknown command: {}", args[1]);
        }
    }
    // print("Events:", events.iter().map(|e| e).collect::<Vec<&Event>>());
    Ok(())
}
