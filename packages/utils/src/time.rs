use cosmwasm_std::{BlockInfo, Timestamp};
use cw_storage_plus::U64Key;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Duration is an amount of time, measured in seconds
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, JsonSchema, Debug)]
pub struct Duration(u64);

impl Duration {
    pub fn new(secs: u64) -> Duration {
        Duration(secs)
    }

    pub fn after(&self, block: &BlockInfo) -> Expiration {
        self.after_time(block.time)
    }

    pub fn after_time(&self, timestamp: Timestamp) -> Expiration {
        Expiration::at_timestamp(timestamp.plus_seconds(self.0))
    }

    pub fn seconds(&self) -> u64 {
        self.0
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, JsonSchema, Debug)]
pub struct Expiration(Timestamp);

impl Expiration {
    pub fn now(block: &BlockInfo) -> Self {
        Self(block.time)
    }

    pub fn at_timestamp(timestamp: Timestamp) -> Self {
        Self(timestamp)
    }

    pub fn is_expired(&self, block: &BlockInfo) -> bool {
        self.is_expired_time(block.time)
    }

    pub fn is_expired_time(&self, timestamp: Timestamp) -> bool {
        timestamp >= self.0
    }

    pub fn time(&self) -> Timestamp {
        self.0
    }

    pub fn as_key(&self) -> U64Key {
        U64Key::new(self.0.nanos())
    }
}
