use crate::error::Error;
use crate::ledger::PartialLedger;
use crate::reader::Readable;
use crate::transaction::Transaction;
use crate::{mining::Miner, reader::Reader};

use super::ledger::Ledger;
use super::sha256::Sha256Hash;
use super::time::Time;
use std::fmt::{self, Debug, Formatter};
use std::vec;

#[derive(Clone)]
pub struct BlockContent {
    pub index: usize,
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
        index: usize,
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

        bytes.extend(self.proof_of_work.to_be_bytes());
        bytes.extend(self.hash.to_bytes());
        bytes.extend(self.content.to_bytes());

        bytes
    }

    pub fn from_bytes(bytes: &Vec<u8>) -> Result<Self, Error> {
        let mut reader = Reader::new(bytes.clone());

        Self::from_reader(&mut reader)
    }
}

impl Readable<Block> for Block {
    fn from_reader(reader: &mut Reader) -> Result<Self, Error> {
        let proof_of_work = match reader.read_u128() {
            Ok(s) => s,
            Err(_) => return Err(Error::InvalidFormat),
        };

        let hash = match reader.read() {
            Ok(s) => s,
            Err(_) => return Err(Error::InvalidFormat),
        };

        let content = match reader.read() {
            Ok(c) => c,
            Err(_) => return Err(Error::InvalidFormat),
        };

        Ok(Block {
            proof_of_work: proof_of_work,
            hash: hash,
            content: content,
        })
    }
}

impl BlockContent {
    pub fn new(
        index: usize,
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

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        /*
            in the code index is usize, because usize is faster than u64 on 32-bit or u32 on 64-bit
            but machine still have to communicate, so we cast it to u32 before converting it to [u8]
        */
        bytes.extend((self.index as u32).to_be_bytes());
        bytes.extend(self.timestamp.to_bytes());
        bytes.extend(self.prev_block_hash.to_bytes());
        bytes.extend((self.transactions.len() as u32).to_be_bytes());
        for elem in &self.transactions {
            bytes.extend(elem.to_bytes());
        }

        bytes
    }

    pub fn from_bytes(bytes: &Vec<u8>) -> Result<Self, Error> {
        let mut reader = Reader::new(bytes.clone());

        Self::from_reader(&mut reader)
    }
}

impl Readable<BlockContent> for BlockContent {
    fn from_reader(reader: &mut Reader) -> Result<Self, Error> {
        let index = match reader.read_u32() {
            Ok(s) => s,
            Err(_) => return Err(Error::InvalidFormat),
        } as usize;

        let timestamp = match reader.read() {
            Ok(s) => s,
            Err(_) => return Err(Error::InvalidFormat),
        };

        let prev_block_hash = match reader.read() {
            Ok(s) => s,
            Err(_) => return Err(Error::InvalidFormat),
        };

        let transactions = match reader.read_vec() {
            Ok(s) => s,
            Err(_) => return Err(Error::InvalidFormat),
        };

        Ok(BlockContent {
            index: index,
            timestamp: timestamp,
            prev_block_hash: prev_block_hash,
            transactions: transactions,
        })
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
