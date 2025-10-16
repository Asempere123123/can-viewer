use can_dbc::{DBC, Message};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

fn generate_map_from_dbc(dbc: &DBC) -> HashMap<u32, Message> {
    dbc.messages()
        .iter()
        .map(|message| (message.message_id().raw(), message.clone()))
        .collect()
}

pub struct Dbc {
    pub name: Arc<str>,
    messages_map: HashMap<u32, Message>,
    raw_dbc: Arc<[u8]>,
    pub inner: DBC,
}

impl Dbc {
    pub fn new(name: Arc<str>, raw_dbc: Arc<[u8]>) -> Result<Self, String> {
        match DBC::from_slice(&raw_dbc) {
            Ok(dbc) => Ok(Self {
                name: name,
                messages_map: generate_map_from_dbc(&dbc),
                raw_dbc: raw_dbc,
                inner: dbc,
            }),
            Err(e) => match e {
                can_dbc::Error::Incomplete(dbc, _str) => {
                    log::warn!("INCOMPLETE DBC");
                    Ok(Self {
                        name: name,
                        messages_map: generate_map_from_dbc(&dbc),
                        raw_dbc: raw_dbc,
                        inner: dbc,
                    })
                }
                _e => Err(format!("INVALID DBC")),
            },
        }
    }

    pub fn into_serializable(&self) -> SerializableDbc {
        SerializableDbc {
            name: self.name.clone(),
            raw_dbc: self.raw_dbc.clone(),
        }
    }

    pub fn from_serializable(serializable: SerializableDbc) -> Result<Self, String> {
        Self::new(serializable.name, serializable.raw_dbc)
    }
}

#[derive(Serialize, Deserialize)]
pub struct SerializableDbc {
    name: Arc<str>,
    raw_dbc: Arc<[u8]>,
}
