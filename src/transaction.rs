use std::fmt::{self, Debug, Formatter};

/*
Note: I do not now if message should be sign by both part.
*/
use super::signature::Signable;
use crate::error::Error;
use crate::num::Num;
use crate::reader::{Readable, Reader};
use crate::user::{PublicUser, User};
// use ed25519_dalek::ed25519::signature::Signature;
// use ed25519_dalek::ed25519::signature::Signature;
use ed25519_dalek::Signature;

#[derive(Clone)]
pub struct TransactionContent {
    pub from: PublicUser,
    pub to: PublicUser,
    pub value: Num,
    pub nonce: u64,
}

#[derive(Clone)]
pub struct Transaction {
    pub content: TransactionContent,
    pub signature: Signature,
}

impl Transaction {
    pub fn new(from: User, to: PublicUser, value: Num, once: u64) -> Self {
        let content = TransactionContent {
            from: from.as_public(),
            to: to,
            value: value,
            nonce: once,
        };

        Transaction {
            signature: content.sign(&from.key_pair),
            content: content,
        }
    }

    pub fn verify(&self, from_account: Num) -> Result<(), Error> {
        if self.content.value <= Num::zero() {
            return Err(Error::TransactionOf0);
        }
        if self.content.from.is_coinbase() {
            if !self.content.verify(self.content.from.key, self.signature) {
                return Err(Error::WrongTransactionSignature);
            }
            if self.content.value > from_account {
                return Err(Error::SenderDoNotHaveEnoughMoney);
            }
        }
        return Ok(());
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        bytes.extend(self.signature.to_bytes());
        bytes.extend(self.content.to_bytes());

        bytes
    }

    pub fn from_bytes(bytes: &Vec<u8>) -> Result<Self, Error> {
        let mut reader = Reader::new(bytes.clone());

        Self::from_reader(&mut reader)
    }
}

impl Readable<Transaction> for Transaction {
    fn from_reader(reader: &mut Reader) -> Result<Self, Error> {
        let signature = match reader.read_signature() {
            Ok(s) => s,
            Err(_) => return Err(Error::InvalidFormat),
        };

        let content = match reader.read() {
            Ok(c) => c,
            Err(e) => return Err(e),
        };

        Ok(Transaction {
            signature: signature,
            content: content,
        })
    }
}

impl Debug for Transaction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} gives {} to {:?}",
            &self.content.from,
            &self.content.value.to_string(),
            &self.content.to,
        )
    }
}

impl TransactionContent {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        bytes.extend(self.from.to_bytes());
        bytes.extend(self.to.to_bytes());
        bytes.extend(self.nonce.to_be_bytes());
        bytes.extend(self.value.to_bytes());

        bytes
    }

    pub fn from_bytes(bytes: &Vec<u8>) -> Result<Self, Error> {
        let mut reader = Reader::new(bytes.clone());

        Self::from_reader(&mut reader)
    }
}

impl Readable<TransactionContent> for TransactionContent {
    fn from_reader(reader: &mut Reader) -> Result<Self, Error> {
        let from = match reader.read() {
            Ok(s) => s,
            Err(_) => return Err(Error::InvalidFormat),
        };

        let to = match reader.read() {
            Ok(s) => s,
            Err(_) => return Err(Error::InvalidFormat),
        };

        let nonce = match reader.read_u64() {
            Ok(s) => s,
            Err(_) => return Err(Error::InvalidFormat),
        };

        let value = match reader.read() {
            Ok(number) => number,
            Err(_) => return Err(Error::InvalidFormat),
        };

        Ok(TransactionContent {
            from: from,
            to: to,
            nonce: nonce,
            value: value,
        })
    }
}

impl Signable for TransactionContent {
    fn bytes(&self) -> Vec<u8> {
        self.to_bytes()
    }
}

impl PartialEq for Transaction {
    fn eq(&self, other: &Self) -> bool {
        self.content == other.content && self.signature == other.signature
    }
}
impl Eq for Transaction {}

impl PartialEq for TransactionContent {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from
            && self.to == other.to
            && self.value == other.value
            && self.nonce == other.nonce
    }
}
impl Eq for TransactionContent {}
