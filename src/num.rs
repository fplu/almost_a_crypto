use std::fmt::{self, Debug, Formatter};
use std::{cmp::Ordering, ops::SubAssign, str::FromStr};

use num::{rational::BigRational, FromPrimitive, Zero};

use crate::error::Error;
use crate::reader::{Readable, Reader};
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

        let str = self.value.to_string();
        let b = str.as_bytes();
        bytes.extend((b.len() as u32).to_be_bytes());
        bytes.extend(b);

        bytes
    }

    pub fn from_bytes(bytes: &Vec<u8>) -> Result<Self, Error> {
        let mut reader = Reader::new(bytes.clone());

        Self::from_reader(&mut reader)
    }
}

impl Readable<Num> for Num {
    fn from_reader(reader: &mut Reader) -> Result<Self, Error> {
        let size = match reader.read_u32() {
            Ok(s) => s,
            Err(_) => return Err(Error::InvalidFormat),
        } as usize;

        let value_slice: Vec<u8> = match reader.read_bytes(size) {
            Ok(s) => s,
            Err(_) => return Err(Error::InvalidFormat),
        };
        let value_str = match String::from_utf8(value_slice) {
            Ok(str) => str,
            Err(_) => return Err(Error::InvalidFormat),
        };
        let value = match BigRational::from_str(value_str.as_str()) {
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
