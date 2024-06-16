use std::collections::HashMap;

use atom_syndication::{FixedDateTime, LinkBuilder, WriteConfig};
use chrono::{DateTime, Utc};
use serde::Serialize;

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
    std::fs::write("events.json", serde_json::to_string_pretty(&events).unwrap().as_bytes()).unwrap();
    tokio::fs::create_dir_all("public").await.ok();
    to_rss(&events).unwrap();
}

fn to_rss(events: &HashMap<String, Event>) -> Result<(), Box<dyn std::error::Error>> {
    let mut feed = atom_syndication::FeedBuilder::default();
    let mut ch = rss::ChannelBuilder::default();
    feed.title("Power Lifting America Events")
        .link(
            LinkBuilder::default()
                .href("http://gh.freemasen.com/plam-event/atom.xml")
                .rel("self")
                .build(),
        )
        .id("http://gh.freemasen.com/plam-event/atom.xml");
    ch.title("Power Lifting America Events")
        .last_build_date(Utc::now().format("%a, %d %b %y %H:%M UT").to_string())
        .pub_date("31 Jan 01 00:00 UT".to_string())
        .link("http://gh.freemasen.com/plam-event/atom.xml");
    let mut last_date = DateTime::from_timestamp(0, 0).expect("0 dt");
    for ev in events.values() {
        let ev_date = ev.date();
        if last_date < ev_date {
            last_date = ev_date.into();
        }
        let item = atom_syndication::EntryBuilder::default()
            .id(&ev.url)
            .title(ev.summary.clone())
            .published(ev_date)
            .updated(ev_date)
            .link(LinkBuilder::default().href(ev.url.to_string())
            .rel("alternate").build())
            .content(
                atom_syndication::ContentBuilder::default()
                    .lang("en-us".to_string())
                    .value(ev.location())
                    .build(),
            )
            .build();
        feed.entry(item);
        let entry = rss::ItemBuilder::default()
            .content(format!("<p>{}</p>", ev.location()))
            .title(format!("<h1>{}</h1>", ev.summary))
            .description(format!("<h2>{}</h2>", ev.description))
            .build();
        ch.item(entry);
    }
    if last_date == DateTime::from_timestamp(0, 0).expect("0 dt") {
        last_date = Utc::now();
    }
    let feed = feed
        .updated(last_date)
        .build();
    let mut f = std::fs::File::options()
        .create(true)
        .write(true)
        .truncate(true)
        .open("./public/atom.xml")
        .unwrap();
    let mut f2 = std::fs::File::options()
        .create(true)
        .write(true)
        .truncate(true)
        .open("./public/rss.xml")
        .unwrap();
    feed.write_with_config(&mut f, WriteConfig {
        indent_size: Some(4),
        write_document_declaration: true,
    }).unwrap();
    ch.build().pretty_write_to(&mut f2, b' ', 4).unwrap();
    Ok(())
}

fn update_event(key: &str, value: &str, ev: &mut Event) {
    const DATE_PREFIX: &str = "VALUE=DATE:";
    let value = value
        .trim_start_matches(DATE_PREFIX)
        .replace(" ", " ")
        .to_string();
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

#[derive(Debug)]
struct Address<'a> {
    addr1: &'a str,
    addr2: &'a str,
    addr3: &'a str,
    city: &'a str,
    state: &'a str,
    zip: &'a str,
    country: &'a str,
}

impl<'a> Address<'a> {
    fn from(s: &'a str) -> Option<Self> {
        let mut ret = Self {
            addr1: "",
            addr2: "",
            addr3: "",
            city: "",
            state: "",
            zip: "",
            country: "",
        };
        let segs = s.split("\\,").collect::<Vec<_>>();
        for (i, seg) in segs.into_iter().rev().enumerate() {
            let seg = seg.trim_matches('\\').trim();
            match i {
                0 => {
                    ret.country = seg;
                },
                1 => {
                    if !seg.trim().chars().all(|c| {
                        if c.is_ascii_digit() {
                            return true;
                        }
                        eprintln!("non digit: {c}");
                        false
                    }) {
                        eprintln!("invalid zip: {}\n`{}`", seg, s);
                        return None;
                    }
                    ret.zip = seg;
                },
                2 => {
                    if seg.len() != 2 || !seg.chars().all(|c| c.is_ascii_uppercase()) {
                        eprintln!("invalid state: `{}`\n`{}`", seg, s);
                        return None;
                    }
                    ret.state = seg;
                },
                3 => {
                    ret.city = seg;
                },
                4 => {
                    ret.addr3 = seg;
                },
                5 => {
                    ret.addr2 = seg;
                },
                6 => {
                    ret.addr1 = seg;
                }
                _ => break,
            }
        }
        Some(ret)
    }
}

impl<'a> std::fmt::Display for Address<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.addr1.is_empty() {
            writeln!(f, "{}", self.addr1)?;
        }
        if !self.addr2.is_empty() {
            writeln!(f, "{}", self.addr2)?;
        }
        if !self.addr3.is_empty() {
            writeln!(f, "{}", self.addr3)?;
        }
        writeln!(f, "{}, {} {}", self.city, self.state, self.zip)
    }
}

impl Event {
    fn date(&self) -> FixedDateTime {
        let year = &self.dtstamp[0..4];
        let month = &self.dtstamp[4..6];
        let day = &self.dtstamp[6..8];
        let hour: &str = &self.dtstamp[9..11];
        FixedDateTime::parse_from_rfc3339(&format!("{year}-{month}-{day}T{hour}:00:00.0Z")).unwrap()
    }

    fn location(&self) -> String {
        let Some(addr) = Address::from(&self.location) else {
            eprintln!("invalid address: `{}`", self.location);
            return self.location.to_string();
        };
        addr.to_string()
    }
}
