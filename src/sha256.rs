use std::{fmt::{ self, Debug, Formatter }, io::{Read, Write}};
use sha2::Sha256;
use sha2::Digest;

use crate::{reader::{Readable, read_u128}, error::Error, writer::{Writable, write_u128}};

#[derive(Copy, Clone)]
pub struct  Sha256Hash {
	pub arr_u128: [u128; 2],
}


impl Debug for Sha256Hash {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		let arr_u8: [u8; 32] = self.to_bytes();

		write!(f, "{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
			arr_u8[0x00], 
			arr_u8[0x01],
			arr_u8[0x02],
			arr_u8[0x03],
			arr_u8[0x04],
			arr_u8[0x05],
			arr_u8[0x06],
			arr_u8[0x07],
			arr_u8[0x08], 
			arr_u8[0x09],
			arr_u8[0x0A],
			arr_u8[0x0B],
			arr_u8[0x0C],
			arr_u8[0x0D],
			arr_u8[0x0E],
			arr_u8[0x0F],
			arr_u8[0x10], 
			arr_u8[0x11],
			arr_u8[0x12],
			arr_u8[0x13],
			arr_u8[0x14],
			arr_u8[0x15],
			arr_u8[0x16],
			arr_u8[0x17],
			arr_u8[0x18], 
			arr_u8[0x19],
			arr_u8[0x1A],
			arr_u8[0x1B],
			arr_u8[0x1C],
			arr_u8[0x1D],
			arr_u8[0x1E],
			arr_u8[0x1F],
		)
	}
}

impl PartialEq for Sha256Hash {
    fn eq(&self, other: &Self) -> bool {
        let a = self.as_u128();
        let b = other.as_u128();
        a[0] == b[0] && a[1] == b[1]
    }
}
impl Eq for Sha256Hash {}


impl Sha256Hash {
    pub const BITS:u32 = 256;

    pub fn new(arr_of_bytes: &[&Vec<u8>]) -> Self{
        let mut hasher = Sha256::new();
        for elem in arr_of_bytes {
            hasher.update(elem);
        }

		Sha256Hash::from_bytes(hasher.finalize().to_vec().try_into().unwrap())

    }
	pub fn zero () -> Self{
		Sha256Hash { arr_u128: [0;2]}	
	}
	pub fn from_bytes(value: [u8; 32]) -> Self{
		Sha256Hash { 
			arr_u128: [u128::from_be_bytes(value[0..16].to_vec().try_into().unwrap()), 
            u128::from_be_bytes(value[16..32].to_vec().try_into().unwrap())]
		}
	}
	pub fn as_u128(&self) -> &[u128; 2] {
		&self.arr_u128
	}

	pub fn to_bytes(&self) -> [u8; 32]  {
		[self.arr_u128[0].to_be_bytes().to_vec(), self.arr_u128[1].to_be_bytes().to_vec()].concat().try_into().unwrap()
	}
}

impl Writable for Sha256Hash {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<(), Error> {
        write_u128(writer, self.arr_u128[0]) //
        .and_then(|_| write_u128(writer, self.arr_u128[1]))
    }
}

impl Readable for Sha256Hash {
    fn from_reader(reader: &mut dyn Read) -> Result<Self, Error> {
        let mut sha = Sha256Hash::zero();

        match read_u128(reader, &mut sha.arr_u128[0]) //
        .and_then(|_| read_u128(reader, &mut sha.arr_u128[1])) {
            Ok(_) => (),
            Err(_) => return Err(Error::InvalidFormat),
        }

        Ok(sha)
	}

}

