use std::collections::{HashMap, HashSet};

use atom_syndication::{Content, FixedDateTime, Link, LinkBuilder};
use serde::Serialize;
use tokio::{io::AsyncWriteExt, process::Command};

#[tokio::main]
async fn main() {
    let url = "https://powerlifting-america.com/?post_type=tribe_events&ical=1&eventDisplay=list";
    let response = reqwest::get(url)
        .await
        .expect("get")
        .error_for_status()
        .expect("200")
        .text()
        .await
        .expect("text");
    let mut events = HashMap::new();
    let mut current = Event::default();
    for line in response.lines() {
        if line.starts_with("END:VEVENT") {
            let ev = std::mem::take(&mut current);
            events.insert(ev.uid.to_string(), ev);
        } else {
            if let Some((key, value)) = line.split_once(';') {
                update_event(key, value, &mut current);
            } else if let Some((key, value)) = line.split_once(":") {
                update_event(key, value, &mut current);
            } else {
                eprintln!("dropping line: `{line}`");
            }
        }
    }
    tokio::fs::create_dir_all("public").await.ok();
    to_rss(&events).unwrap();
}

fn to_rss(events: &HashMap<String, Event>) -> Result<(), Box<dyn std::error::Error>> {
    let mut ch = atom_syndication::FeedBuilder::default();
    ch.title("Power Lifting America Events")
        .link(LinkBuilder::default().href("https://github.com/freemasen/plam-event").build()
    );
        
    for ev in events.values() {
        let item = atom_syndication::EntryBuilder::default()
            .title(ev.summary.clone())
            .published(ev.date())
            .updated(ev.date())
            .content(
                atom_syndication::ContentBuilder::default()
                    .lang("en-us".to_string())
                    .base(ev.url.clone())
                    .value(ev.location.clone())
                    .build())
            .build();
        ch.entry(item);
    }
    let ch = ch.build();
    let mut f = std::fs::File::options()
        .create(true)
        .write(true)
        .truncate(true)
        .open("./public/atom.xml")
        .unwrap();
    ch.write_to(&mut f).unwrap();
    Ok(())
}

fn update_event(key: &str, value: &str, ev: &mut Event) 
{
    const DATE_PREFIX: &str = "VALUE=DATE:";
    let value = value.trim_start_matches(DATE_PREFIX).replace(" ", " ").to_string();
    match key {
        "UID" => ev.uid = value,
        "END" => ev.end = Some(value),
        "URL" => ev.url = value,
        "PRODID" => ev.prodid = Some(value),
        "TZID" => ev.tzid = Some(value),
        "LOCATION" => ev.location = value,
        "BEGIN" => ev.begin = Some(value),
        "DTEND" => ev.dtend = value,
        "DTSTAMP" => ev.dtstamp = value,
        "SUMMARY" => ev.summary = value,
        "X-WR-CALDESC" => ev.x_wr_caldesc = Some(value),
        "TZOFFSETTO" => ev.tzoffsetto = Some(value),
        "X-ROBOTS-TAG" => ev.x_robots_tag = Some(value),
        "X-PUBLISHED-TTL" => ev.x_published_ttl = Some(value),
        "CATEGORIES" => ev.categories = value,
        "TZOFFSETFROM" => ev.tzoffsetfrom = Some(value),
        "DTSTART" => ev.dtstart = value,
        "LAST-MODIFIED" => ev.last_modified = value,
        "ORGANIZER" => ev.organizer = Some(value),
        "TZNAME" => ev.tzname = Some(value),
        "REFRESH-INTERVAL" => ev.refresh_interval = Some(value),
        "CALSCALE" => ev.calscale = Some(value),
        "ATTACH" => ev.attach = value,
        "METHOD" => ev.method = Some(value),
        "X-WR-CALNAME" => ev.x_wr_calname = Some(value),
        "CREATED" => ev.created = value,
        "X-ORIGINAL-URL" => ev.x_original_url = Some(value),
        "VERSION" => ev.version = Some(value),
        "DESCRIPTION" => ev.description = value,
        "X-Robots-Tag" => ev.x_robots_tag = Some(value),
        _ => eprintln!("dropping key: {key}: {value}"),
    }
}

#[derive(Debug, Default, Serialize, Clone)]
struct Event {
    pub uid: String,
    pub end: Option<String>,
    pub url: String,
    pub prodid: Option<String>,
    pub tzid: Option<String>,
    pub location: String,
    pub begin: Option<String>,
    pub dtend: String,
    pub dtstamp: String,
    pub summary: String,
    pub x_wr_caldesc: Option<String>,
    pub tzoffsetto: Option<String>,
    pub x_robots_tag: Option<String>,
    pub x_published_ttl: Option<String>,
    pub categories: String,
    pub tzoffsetfrom: Option<String>,
    pub dtstart: String,
    pub last_modified: String,
    pub organizer: Option<String>,
    pub tzname: Option<String>,
    pub refresh_interval: Option<String>,
    pub calscale: Option<String>,
    pub attach: String,
    pub method: Option<String>,
    pub x_wr_calname: Option<String>,
    pub created: String,
    pub x_original_url: Option<String>,
    pub version: Option<String>,
    pub description: String,
}

impl Event {
    fn date(&self) -> FixedDateTime {
        let year = &self.dtstamp[0..4];
        let month = &self.dtstamp[4..6];
        let day = &self.dtstamp[6..8];
        let hour = &self.dtstamp[9..11];
        FixedDateTime::parse_from_rfc3339(&format!("{year}-{month}-{day}T{hour}:00:00.0Z")).unwrap()
    }
}
