use std::{
    io::{Read, Write},
    time::{SystemTime, UNIX_EPOCH},
    u128,
};

use crate::{
    error::Error,
    reader::{read_u128, Readable},
    writer::{write_u128, Writable},
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

impl Writable for Time {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<(), Error> {
        write_u128(writer, self.to_u128())
    }
}

impl Readable for Time {
    fn from_reader(reader: &mut dyn Read) -> Result<Self, Error> {
        let mut u: u128 = 0;

        match read_u128(reader, &mut u) {
            Ok(_) => (),
            Err(_) => return Err(Error::InvalidFormat),
        };

        Ok(Time::from_u128(u))
    }
}

impl PartialEq for Time {
    fn eq(&self, other: &Self) -> bool {
        self.secs == other.secs && self.subsec_millis == other.subsec_millis
    }
}
impl Eq for Time {}
