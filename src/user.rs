use ed25519_dalek::{Keypair, PublicKey, PUBLIC_KEY_LENGTH, SecretKey};
use std::fmt::{self, Debug, Formatter};
use rand::rngs::OsRng;

use crate::{reader::{Reader, Readable}, error::Error};

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct PublicUser {
	pub key: PublicKey,
}

pub struct User {
	pub key_pair: Keypair,
}

impl User {
	pub fn from_nothingness() -> Self {
		let mut csprng = OsRng {};
		let key_pair: Keypair = Keypair::generate(&mut csprng);

		User { key_pair: key_pair }
	}
	pub fn new(key_pair: Keypair) -> Self {
		User { key_pair: key_pair }
	}

	pub fn as_public(&self) -> PublicUser {
		PublicUser::new(self.key_pair.public)
	}

	pub fn new_coinbase() -> Self {
		let zero: [u8; 32]= [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
		User { key_pair: Keypair {
			secret: SecretKey::from_bytes(&zero).unwrap(),
			public: PublicKey::from_bytes(&zero).unwrap(),
		} }
	}
}

impl PublicUser {
	pub const BITS:u32 = (PUBLIC_KEY_LENGTH as u32)*8;
	pub const BYTES:usize = PUBLIC_KEY_LENGTH;

	pub fn new_coinbase() -> Self {
		let zero: [u8; 32]= [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
		PublicUser { 
			key: PublicKey::from_bytes(&zero).unwrap()
		} 
	}

	pub fn new(public_key: PublicKey) -> Self {
		PublicUser { key: public_key }
	}

	pub fn to_bytes(&self) -> &[u8; PublicUser::BYTES] {
		self.key.as_bytes()
	}

	pub fn from_bytes(bytes: &[u8; PublicUser::BYTES]) -> Self {

		PublicUser {
			key: match PublicKey::from_bytes(bytes) {
				Ok(number) => number,
				Err(_) => panic!("unreachable code!"),
			},
		}
	}
	pub fn is_coinbase(&self) -> bool {
		self.key.as_bytes().into_iter().all(|&b| b == 0)
	}
}

impl Readable<PublicUser> for PublicUser {

	fn from_reader(reader: &mut Reader) -> Result<Self, Error> {
		let slice: [u8; PUBLIC_KEY_LENGTH] = match reader.read_bytes(PUBLIC_KEY_LENGTH) {
			Ok(s) => s,
			Err(_) => return Err(Error::InvalidFormat),
		};

		Ok(PublicUser::from_bytes(&slice))
	}
}

// impl PartialEq for PublicUser {
// 	fn eq(&self, public_user: &PublicUser) -> bool {
// 		self.key == public_user.key
// 	}
// }

impl Debug for PublicUser {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		let buf = self.key.as_bytes();
		write!(
			f,
			"{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
			buf[0x00], 
			buf[0x01],
			buf[0x02],
			buf[0x03],
			buf[0x04],
			buf[0x05],
			buf[0x06],
			buf[0x07],
			buf[0x08], 
			buf[0x09],
			buf[0x0A],
			buf[0x0B],
			buf[0x0C],
			buf[0x0D],
			buf[0x0E],
			buf[0x0F],
			buf[0x10], 
			buf[0x11],
			buf[0x12],
			buf[0x13],
			buf[0x14],
			buf[0x15],
			buf[0x16],
			buf[0x17],
			buf[0x18], 
			buf[0x19],
			buf[0x1A],
			buf[0x1B],
			buf[0x1C],
			buf[0x1D],
			buf[0x1E],
			buf[0x1F],
		)
	}
}
