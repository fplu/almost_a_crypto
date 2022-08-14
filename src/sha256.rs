use std::{fmt::{ self, Debug, Formatter }};
use sha2::Sha256;
use sha2::Digest;

use crate::{reader::{Reader, Readable}, error::Error};

#[derive(Copy, Clone)]
pub struct  Sha256Hash {
	pub arr_u128: [u128; 2],
	// pub arr_u64: [u64; 4],
	// pub arr_u32: [u32; 8],
	// pub arr_u16: [u16; 16],
	// pub arr_u8: [u8; 32]
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
            // hasher.digest

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
	
	// pub fn as_u32(&self) -> &[u32; 8] {
	// 	unsafe{&self.arr_u32}
	// }

	pub fn to_bytes(&self) -> [u8; 32]  {
		[self.arr_u128[0].to_be_bytes().to_vec(), self.arr_u128[1].to_be_bytes().to_vec()].concat().try_into().unwrap()
	}
}

impl Readable<Sha256Hash> for Sha256Hash {
    fn from_reader(reader: &mut Reader) -> Result<Self, Error> {
        let a = match reader.read_u128() {
            Ok(v) => v,
            Err(e) => return Err(e)
        };
        let b = match reader.read_u128() {
            Ok(v) => v,
            Err(e) => return Err(e)
        };
        Ok(Sha256Hash { 
			arr_u128: [a, b]
		})
	}

}

