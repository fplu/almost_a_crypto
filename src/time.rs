use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    error::Error,
    reader::{Readable, Reader},
};

#[derive(Copy, Clone)]
pub struct Time {
    pub secs: u64,
    pub subsec_millis: u32,
}

impl Time {
    pub fn zero() -> Self {
        Time {
            secs: 0,
            subsec_millis: 0,
        }
    }

    pub fn from_second(secs: u64) -> Self {
        Time {
            secs: secs,
            subsec_millis: 0,
        }
    }

    pub fn now() -> Self {
        let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        Time {
            secs: duration.as_secs(),
            subsec_millis: duration.subsec_millis(),
        }
    }

    pub fn to_u128(&self) -> u128 {
        self.secs as u128 * 1000 + self.subsec_millis as u128
    }

    pub fn from_u128(u: u128) -> Self {
        Time {
            secs: (u / 1000) as u64,
            subsec_millis: (u % 1000) as u32,
        }
    }

    pub fn to_bytes(&self) -> [u8; 16] {
        self.to_u128().to_be_bytes()
    }

    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Time::from_u128(u128::from_be_bytes(bytes))
    }
}

impl Readable<Time> for Time {
    fn from_reader(reader: &mut Reader) -> Result<Self, Error> {
        Ok(Time::from_u128(match reader.read_u128() {
            Ok(v) => v,
            Err(e) => return Err(e),
        }))
    }
}

impl PartialEq for Time {
    fn eq(&self, other: &Self) -> bool {
        self.secs == other.secs && self.subsec_millis == other.subsec_millis
    }
}
impl Eq for Time {}
