use can_dbc::DBC;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct Dbc {
    pub name: Arc<str>,
    raw_dbc: Arc<[u8]>,
    pub inner: DBC,
}

impl Dbc {
    pub fn new(name: Arc<str>, raw_dbc: Arc<[u8]>) -> Result<Self, String> {
        match DBC::from_slice(&raw_dbc) {
            Ok(dbc) => Ok(Self {
                name: name,
                raw_dbc: raw_dbc,
                inner: dbc,
            }),
            Err(e) => match e {
                can_dbc::Error::Incomplete(dbc, _str) => {
                    log::warn!("INCOMPLETE DBC");
                    Ok(Self {
                        name: name,
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
