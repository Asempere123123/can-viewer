use chrono::{DateTime, Utc};
use regex_macro::regex;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Messages(Vec<Message>);

impl Messages {
    pub fn from_string(string: Cow<'_, str>) -> Messages {
        Messages(
            string
                .lines()
                .filter_map(|message_str| Message::from_str(message_str))
                .collect(),
        )
    }

    pub fn empty() -> Messages {
        Messages(Vec::new())
    }

    pub fn extend(&mut self, other: &Messages) {
        self.0.extend_from_slice(&other.0);
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Message {
    id: u32,
    contents: u64,
    timestamp: DateTime<Utc>,
}

impl Message {
    fn from_str(str: &str) -> Option<Message> {
        if let Some(captures) =
            regex!(r"\(([\d.]+)\)\s+\w+\s+([0-9A-Fa-f]+)#([0-9A-Fa-f]+)").captures(str)
        {
            let Ok(id) = u32::from_str_radix(&captures[2], 10) else {
                return None;
            };

            let Ok(contents) = u64::from_str_radix(&captures[3], 16) else {
                return None;
            };

            let mut timestamp_str = captures[1].split(".");
            let Some(seconds_str) = timestamp_str.next() else {
                return None;
            };
            let Some(nanos_str) = timestamp_str.next() else {
                return None;
            };
            // It is invalid already, so that i dont use it accidentaly
            drop(timestamp_str);

            let Ok(seconds) = i64::from_str_radix(seconds_str, 10) else {
                return None;
            };
            let Some(nanos) = parse_nanos(nanos_str) else {
                return None;
            };

            let Some(timestamp) = DateTime::from_timestamp(seconds, nanos) else {
                return None;
            };

            Some(Message {
                id,
                contents,
                timestamp,
            })
        } else {
            None
        }
    }
}

fn parse_nanos(s: &str) -> Option<u32> {
    // Scale by 10^(9 - len) to get nanoseconds
    let nanos = s.parse::<u32>().ok()? * (10u32.pow(9 - (s.len() as u32)));
    Some(nanos)
}
