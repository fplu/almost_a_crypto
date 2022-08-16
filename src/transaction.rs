use std::fmt::{self, Debug, Formatter};
use std::io::Read;
use std::io::Write;

use super::signature::Signable;
use crate::error::Error;
use crate::num::Num;
use crate::reader::{read_signature, read_struct, read_u64, Readable};
use crate::user::{PublicUser, User};
use crate::writer::{write_signature, write_struct, write_u64, Writable};
use ed25519_dalek::Signature;
use rand::Rng;

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
    pub fn new_from_coinbase(to: &PublicUser, value: &Num) -> Self {
        Transaction {
            content: TransactionContent {
                from: PublicUser::new_coinbase(),
                to: to.clone(),
                value: value.clone(),
                nonce: rand::thread_rng().gen_range(0, u64::max_value()),
            },
            signature: Signature::from_bytes(&[
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0,
            ])
            .unwrap(),
        }
    }
    pub fn zero() -> Self {
        Transaction {
            content: TransactionContent::zero(),
            signature: Signature::from_bytes(&[
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0,
            ])
            .unwrap(),
        }
    }
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

        self.to_writer(&mut bytes).ok();

        bytes
    }

    pub fn from_bytes(bytes: &Vec<u8>) -> Result<Self, Error> {
        let mut slice: &[u8] = bytes;
        Self::from_reader(&mut slice)
    }
}

impl Writable for Transaction {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<(), Error> {
        write_signature(writer, &self.signature).and_then(|_| write_struct(writer, &self.content))
    }
}

impl Readable for Transaction {
    fn from_reader(reader: &mut dyn Read) -> Result<Self, Error> {
        let mut transaction = Transaction::zero();

        match read_signature(reader, &mut transaction.signature) //
            .and_then(|_| read_struct(reader, &mut transaction.content))
        {
            Ok(_) => (),
            Err(e) => return Err(e),
        };

        Ok(transaction)
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
    pub fn zero() -> Self {
        TransactionContent {
            from: PublicUser::zero(),
            to: PublicUser::zero(),
            value: Num::zero(),
            nonce: 0,
        }
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

impl Writable for TransactionContent {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<(), Error> {
        write_struct(writer, &self.from)
            .and_then(|_| write_struct(writer, &self.to))
            .and_then(|_| write_u64(writer, self.nonce))
            .and_then(|_| write_struct(writer, &self.value))
    }
}

impl Readable for TransactionContent {
    fn from_reader(reader: &mut dyn Read) -> Result<Self, Error> {
        let mut transaction_content = TransactionContent::zero();

        match read_struct(reader, &mut transaction_content.from)
            .and_then(|_| read_struct(reader, &mut transaction_content.to))
            .and_then(|_| read_u64(reader, &mut transaction_content.nonce))
            .and_then(|_| read_struct(reader, &mut transaction_content.value))
        {
            Ok(_) => (),
            Err(_) => return Err(Error::InvalidFormat),
        };

        Ok(transaction_content)
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
