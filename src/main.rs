extern crate chrono;
extern crate dirs;
extern crate serde;
extern crate storaget;

use chrono::prelude::*;
use chrono::ParseResult;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use storaget::*;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Event {
    description: String,
    month: u32,
    day: u32,
    kind: EventKind,
}

impl Event {
    pub fn new(description: String, month: u32, day: u32, kind: EventKind) -> Self {
        Event {
            description,
            month,
            day,
            kind,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EventKind {
    Birthday,
    Namesday,
    Other,
}

impl Default for EventKind {
    fn default() -> Self {
        EventKind::Other
    }
}

fn date_from_md(month: u32, day: u32) -> ParseResult<NaiveDate> {
    NaiveDate::parse_from_str(
        &format!("{}-{}-{}", Utc::today().year(), month, day),
        "%Y-%m-%d",
    )
}

fn date_from_ymd(year: i32, month: u32, day: u32) -> ParseResult<NaiveDate> {
    NaiveDate::parse_from_str(&format!("{}-{}-{}", year, month, day), "%Y-%m-%d")
}

fn date_from_str(date: &str) -> ParseResult<NaiveDate> {
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
}

fn add(token: &[String]) {}

fn main() -> PackResult<()> {
    let mut events: Pack<Vec<Event>> = Pack::load_or_init(
        dirs::home_dir().expect("Error while getting your home folder"),
        ".event",
    )?;
    events.as_mut().push(Event::default());
    Ok(())
}
