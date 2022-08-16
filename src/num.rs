use std::fmt::{self, Debug, Formatter};
use std::io::Read;
use std::io::Write;
use std::{cmp::Ordering, ops::SubAssign, str::FromStr};

use num::{rational::BigRational, FromPrimitive, Zero};

use crate::error::Error;
use crate::reader::{read_string, Readable};
use crate::writer::{write_string, Writable};
use std::ops::AddAssign;

#[derive(Clone)]
pub struct Num {
    value: BigRational,
}

impl Num {
    pub fn from_u64(u: u64) -> Self {
        Num {
            value: BigRational::from_u64(u).unwrap(),
        }
    }

    pub fn zero() -> Self {
        Num {
            value: BigRational::zero(),
        }
    }

    pub fn to_string(&self) -> String {
        self.value.to_string()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        self.to_writer(&mut bytes).ok();

        bytes
    }

    pub fn from_bytes(bytes: &Vec<u8>) -> Result<Self, Error> {
        let mut slice: &[u8] = bytes;
        Self::from_reader(&mut slice)
    }
}

impl Writable for Num {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<(), Error> {
        write_string(writer, self.to_string())
    }
}

impl Readable for Num {
    fn from_reader(reader: &mut dyn Read) -> Result<Self, Error> {
        let mut str = String::new();
        match read_string(reader, &mut str) {
            Ok(s) => s,
            Err(_) => return Err(Error::InvalidFormat),
        };

        let value = match BigRational::from_str(str.as_str()) {
            Ok(number) => number,
            Err(_) => return Err(Error::InvalidFormat),
        };

        Ok(Num { value: value })
    }
}

impl AddAssign for Num {
    fn add_assign(&mut self, other: Self) {
        self.value += other.value;
    }
}

impl SubAssign for Num {
    fn sub_assign(&mut self, other: Self) {
        self.value -= other.value;
    }
}

impl Ord for Num {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl PartialOrd for Num {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.value.cmp(&other.value))
    }
}

impl PartialEq for Num {
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }
}
impl Eq for Num {}

impl Debug for Num {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.value.to_string())
    }
}
