use can_dbc::MessageId;
use chrono::{DateTime, Utc};
use regex_macro::regex;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Messages(pub HashMap<RawCanMessageId, Vec<Message>>);

impl Messages {
    pub fn from_string(string: Cow<'_, str>) -> Messages {
        let mut messages: HashMap<RawCanMessageId, Vec<Message>> = HashMap::new();
        for (msg_id, message) in string
            .lines()
            .filter_map(|message_str| Message::from_str(message_str))
        {
            messages.entry(msg_id).or_default().push(message);
        }

        Messages(messages)
    }

    pub fn empty() -> Messages {
        Messages(HashMap::new())
    }

    pub fn extend(&mut self, other: &Messages) {
        self.0
            .extend(other.0.iter().map(|(id, msgs)| (*id, msgs.clone())));
    }

    pub fn push(&mut self, id: RawCanMessageId, msg: Message) {
        let messages = self.0.entry(id).or_default();
        let idx = match messages.binary_search_by(|msg_to_comp_against| {
            msg_to_comp_against.timestamp.cmp(&msg.timestamp)
        }) {
            Ok(idx) => idx,
            Err(idx) => idx,
        };

        messages.insert(idx, msg);
    }

    pub fn len(&self) -> usize {
        self.0.iter().map(|(_k, messages)| messages.len()).sum()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub contents: [u8; 8],
    pub timestamp: DateTime<Utc>,
}

impl Message {
    pub fn from_str(str: &str) -> Option<(RawCanMessageId, Message)> {
        if let Some(captures) =
            regex!(r"\(([\d.]+)\)\s+\w+\s+([0-9A-Fa-f]+)#([0-9A-Fa-f]+)").captures(str)
        {
            let Ok(id) = u32::from_str_radix(&captures[2], 16) else {
                return None;
            };

            use hex::FromHex;
            let Ok(contents) = <[u8; 8]>::from_hex(&captures[3]) else {
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

            Some((
                RawCanMessageId(id),
                Message {
                    contents,
                    timestamp,
                },
            ))
        } else {
            None
        }
    }
}

// The MessageId::raw() method adds the extended tag bit, which is inconvenient for this use case
#[derive(Eq, Hash, PartialEq, Clone, Copy, Serialize, Deserialize, Debug)]
#[repr(transparent)]
pub struct RawCanMessageId(pub u32);

impl From<MessageId> for RawCanMessageId {
    fn from(value: MessageId) -> Self {
        match value {
            MessageId::Standard(id) => Self(id as u32),
            MessageId::Extended(id) => Self(id),
        }
    }
}

fn parse_nanos(s: &str) -> Option<u32> {
    // Scale by 10^(9 - len) to get nanoseconds
    let nanos = s.parse::<u32>().ok()? * (10u32.pow(9 - (s.len() as u32)));
    Some(nanos)
}
