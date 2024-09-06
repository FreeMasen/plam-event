use std::collections::HashMap;

use atom_syndication::{CategoryBuilder, FixedDateTime, LinkBuilder, Text, WriteConfig};
use chrono::{DateTime, Datelike, Local, Utc};
use serde::Serialize;

const HTML_TEMPLATE: &str = include_str!("template.html");

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
    std::fs::write(
        "events.json",
        serde_json::to_string_pretty(&events).unwrap().as_bytes(),
    )
    .unwrap();
    tokio::fs::create_dir_all("public").await.ok();
    to_rss(&events).unwrap();
}

fn to_rss(events: &HashMap<String, Event>) -> Result<(), Box<dyn std::error::Error>> {
    let mut feed = atom_syndication::FeedBuilder::default();
    feed.title("Power Lifting America Events")
        .link(
            LinkBuilder::default()
                .href("http://gh.freemasen.com/plam-event/atom.xml")
                .rel("self")
                .build(),
        )
        .id("http://gh.freemasen.com/plam-event/atom.xml");
    let mut last_date = DateTime::from_timestamp(0, 0).expect("0 dt");
    for ev in events.values() {
        let ev_date = ev.date();
        let published = ev.created();
        if last_date < published {
            last_date = published.into();
        }
        let item = atom_syndication::EntryBuilder::default()
            .id(&ev.url)
            .title(ev.summary.clone())
            .published(published)
            .updated(published)
            .link(
                LinkBuilder::default()
                    .href(ev.url.to_string())
                    .rel("alternate")
                    .build(),
            )
            .summary(Text::plain(ev.summary()))
            .content(
                atom_syndication::ContentBuilder::default()
                    .lang("en-us".to_string())
                    .content_type("html".to_string())
                    .base(ev.url.to_string())
                    .value(ev.content())
                    .build(),
            )
            .category(CategoryBuilder::default().term(ev.state()).build())
            .category(
                CategoryBuilder::default()
                    .term(format!("year-{}", ev_date.year()))
                    .build(),
            )
            .build();
        feed.entry(item);
    }
    if last_date == DateTime::from_timestamp(0, 0).expect("0 dt") {
        last_date = Utc::now();
    }
    let feed = feed.updated(last_date).build();
    let mut f = std::fs::File::options()
        .create(true)
        .write(true)
        .truncate(true)
        .open("./public/atom.xml")
        .unwrap();
    feed.write_with_config(
        &mut f,
        WriteConfig {
            indent_size: Some(4),
            write_document_declaration: true,
        },
    )
    .unwrap();
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

#[derive(Debug, Default)]
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
                }
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
                }
                2 => {
                    if seg.len() != 2 || !seg.chars().all(|c| c.is_ascii_uppercase()) {
                        eprintln!("invalid state: `{}`\n`{}`", seg, s);
                        return None;
                    }
                    ret.state = seg;
                }
                3 => {
                    ret.city = seg;
                }
                4 => {
                    ret.addr3 = seg;
                }
                5 => {
                    ret.addr2 = seg;
                }
                6 => {
                    ret.addr1 = seg;
                }
                _ => break,
            }
        }
        Some(ret)
    }

    fn addr1(&self) -> Option<&str> {
        (!self.addr1.is_empty()).then_some(self.addr1)
    }
    fn addr2(&self) -> Option<&str> {
        (!self.addr2.is_empty()).then_some(self.addr2)
    }
    fn addr3(&self) -> Option<&str> {
        (!self.addr3.is_empty()).then_some(self.addr3)
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
        Self::dt_from_str(&self.dtstamp)
    }

    fn created(&self) -> FixedDateTime {
        Self::dt_from_str(&self.created)
    }

    fn address(&self) -> Address {
        Address::from(&self.location).unwrap_or_default()
    }

    fn state(&self) -> &str {
        let Some(addr) = Address::from(&self.location) else {
            eprintln!("invalid address: `{}`", self.location);
            return "UN";
        };
        addr.state
    }

    fn date_str_to_rfc_string(s: &str) -> String {
        let year = &s[0..4];
        let month = &s[4..6];
        let day = &s[6..8];
        let hour = &s[9..11];
        let minute = &s[11..13];
        let sec = &s[13..15];
        format!("{year}-{month}-{day}T{hour}:{minute}:{sec}.0Z")
    }

    fn dt_from_str(s: &str) -> FixedDateTime {
        FixedDateTime::parse_from_rfc3339(&Self::date_str_to_rfc_string(s)).unwrap()
    }

    fn summary(&self) -> String {
        let dt = self.date().with_timezone(Local::now().offset());
        format!("{}-{}-{} ({}) {}", dt.year(), dt.month(), dt.day(), self.state(), self.summary)
    }

    fn content(&self) -> String {
        let addr = self.address();
        let date = self.date();

        HTML_TEMPLATE
            .replace("{{event_name}}", &self.summary)
            .replace(
                "{{event_date}}",
                &format!("{:0>4}-{:0>2}-{:0>2}", date.year(), date.month(), date.day()),
            )
            .replace("{{address1}}", &addr.addr1().map(|a| format!("{a}<br />")).unwrap_or_else(String::new))
            .replace("{{address2}}", &addr.addr2().map(|a| format!("{a}<br />")).unwrap_or_else(String::new))
            .replace("{{address3}}", &addr.addr3().map(|a| format!("{a}<br />")).unwrap_or_else(String::new))
            .replace("{{city}}", addr.city)
            .replace("{{state}}", addr.state)
            .replace("{{zip}}", addr.zip)
    }
}
