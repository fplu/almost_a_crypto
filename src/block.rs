use super::ledger::Ledger;
use super::sha256::Sha256Hash;
use super::time::Time;
use crate::error::Error;
use crate::ledger::PartialLedger;
use crate::merkle::Merkle;
use crate::mining::Miner;
use crate::num::Num;
use crate::reader::{read_struct, read_u128, read_u32, read_vec_struct, Readable};
use crate::transaction::Transaction;
use crate::user::PublicUser;
use crate::writer::{write_struct, write_u128, write_u32, write_vec_struct, Writable};
use std::fmt::{self, Debug, Formatter};
use std::io::{Read, Write};
use std::vec;

#[derive(Clone)]
pub struct BlockContent {
    pub index: u32,
    pub timestamp: Time,
    pub prev_block_hash: Sha256Hash,
    pub transactions: Vec<Transaction>,
}
#[derive(Clone)]
pub struct Block {
    pub content: BlockContent,
    pub proof_of_work: u128,
    pub hash: Sha256Hash,
}

impl Debug for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Block[{}]: {:?} at: {} with: {:?}",
            &self.content.index,
            &self.hash,
            &self.content.timestamp.to_u128(),
            &self.content.transactions,
        )
    }
}

impl Block {
    pub fn zero() -> Self {
        Block {
            content: BlockContent::zero(),
            proof_of_work: 0,
            hash: Sha256Hash::zero(),
        }
    }
    pub fn new_genesis() -> Self {
        Block {
            content: BlockContent {
                index: 0,
                timestamp: Time::zero(),
                prev_block_hash: Sha256Hash::zero(),
                transactions: vec![],
            },
            proof_of_work: 0,
            hash: Sha256Hash::zero(),
        }
    }

    // these three properties are unique to the genesis block
    pub fn is_genesis(&self) -> bool {
        self.content.index == 0
            && self.content.transactions.is_empty()
            && self.content.prev_block_hash == Sha256Hash::zero()
            && self.content.timestamp == Time::zero()
    }

    pub fn new(
        index: u32,
        timestamp: Time,
        prev_block_hash: Sha256Hash,
        transactions: Vec<Transaction>,
    ) -> Self {
        let content = BlockContent {
            index: index,
            timestamp: timestamp,
            prev_block_hash: prev_block_hash,
            transactions: transactions,
        };

        Block {
            content: content,
            hash: Sha256Hash::zero(),
            proof_of_work: 0,
        }
    }

    pub fn new_mined(block_content: BlockContent, proof_of_work: u128, hash: Sha256Hash) -> Self {
        Block {
            content: block_content,
            hash: hash,
            proof_of_work: proof_of_work,
        }
    }

    pub fn verify(&self, difficulty: u128) -> Result<(), Error> {
        // The genesis should never be verified
        if self.is_genesis() {
            return Err(Error::VerifyingGenesisBlock);
        }

        if self.content.transactions.is_empty() {
            return Err(Error::BlockContainsNoTransaction);
        }

        if !Miner::check_difficulty(&self.hash, difficulty) {
            return Err(Error::BlockProofOfWorkIsNotDone);
        }

        if self.hash != self.hash() {
            return Err(Error::BlockHashIsInvalid);
        }

        return Ok(());
    }

    pub fn verify_payload(&self, ledger: &Ledger) -> Result<Ledger, Error> {
        let mut partial_ledger: PartialLedger = PartialLedger::empty();

        for transaction in &self.content.transactions {
            match partial_ledger.apply_transaction(ledger, transaction) {
                Ok(_) => (),
                Err(e) => return Err(e),
            }
        }
        return Ok(partial_ledger.to_ledger(ledger));
    }

    pub fn hash(&self) -> Sha256Hash {
        Sha256Hash::new(&[
            &self.content.to_bytes(),
            &self.proof_of_work.to_be_bytes().to_vec(),
        ])
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

impl Writable for Block {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<(), Error> {
        write_u128(writer, self.proof_of_work)
            .and_then(|_| write_struct(writer, &self.hash))
            .and_then(|_| write_struct(writer, &self.content))
    }
}

impl Readable for Block {
    fn from_reader(reader: &mut dyn Read) -> Result<Self, Error> {
        let mut block = Block::zero();

        match read_u128(reader, &mut block.proof_of_work)
            .and_then(|_| read_struct(reader, &mut block.hash))
            .and_then(|_| read_struct(reader, &mut block.content))
        {
            Ok(_) => (),
            Err(_) => return Err(Error::InvalidFormat),
        };

        Ok(block)
    }
}

impl BlockContent {
    pub fn zero() -> Self {
        BlockContent {
            index: 0,
            timestamp: Time::zero(),
            prev_block_hash: Sha256Hash::zero(),
            transactions: vec![],
        }
    }

    pub fn new(
        index: u32,
        timestamp: Time,
        prev_block_hash: Sha256Hash,
        transactions: Vec<Transaction>,
    ) -> Self {
        BlockContent {
            index: index,
            timestamp: timestamp,
            prev_block_hash: prev_block_hash,
            transactions: transactions,
        }
    }

    pub fn new_from_pending_transaction(
        pending_transactions: &Vec<Transaction>,
        merkle: &Merkle,
        public_user: &PublicUser,
    ) -> Self {
        let ledger = &merkle.main().ledger;
        let mut block_transactions: Vec<Transaction> = vec![Transaction::new_from_coinbase(
            &public_user,
            &Num::from_u64(1),
        )];
        for transaction in pending_transactions {
            if !ledger.contains(transaction) {
                block_transactions.push(transaction.clone());
            }
        }

        BlockContent {
            index: merkle.main().last(&merkle.blocks).content.index + 1,
            timestamp: Time::now(),
            prev_block_hash: merkle.main().last(&merkle.blocks).hash,
            transactions: block_transactions,
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

impl Writable for BlockContent {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<(), Error> {
        write_u32(writer, self.index as u32)
            .and_then(|_| write_struct(writer, &self.timestamp))
            .and_then(|_| write_struct(writer, &self.prev_block_hash))
            .and_then(|_| write_vec_struct(writer, &self.transactions))
    }
}

impl Readable for BlockContent {
    fn from_reader(reader: &mut dyn Read) -> Result<Self, Error> {
        let mut block_content = BlockContent::zero();

        match read_u32(reader, &mut block_content.index)
            .and_then(|_| read_struct(reader, &mut block_content.timestamp))
            .and_then(|_| read_struct(reader, &mut block_content.prev_block_hash))
            .and_then(|_| read_vec_struct(reader, &mut block_content.transactions))
        {
            Ok(_) => (),
            Err(_) => return Err(Error::InvalidFormat),
        };

        Ok(block_content)
    }
}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        self.content == other.content
            && self.proof_of_work == other.proof_of_work
            && self.hash == other.hash
    }
}
impl Eq for Block {}

impl PartialEq for BlockContent {
    fn eq(&self, other: &Self) -> bool {
        if !(self.index == other.index
            && self.timestamp == other.timestamp
            && self.prev_block_hash == other.prev_block_hash)
        {
            return false;
        }

        for (a, b) in self.transactions.iter().zip(other.transactions.iter()) {
            if a != b {
                return false;
            }
        }

        return true;
    }
}
impl Eq for BlockContent {}
