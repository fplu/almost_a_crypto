use std::io::Read;
use std::io::Write;

use crate::error::Error;
use crate::ledger::Ledger;
use crate::merkle::Merkle;
use crate::reader::{read_struct, read_vec_u32, Readable};
use crate::sha256::Sha256Hash;
use crate::writer::{write_struct, write_vec_u32, Writable};

use super::block::Block;
use super::mining::Miner;

#[derive(Clone, PartialEq, Eq)]
pub struct Blockchain {
    pub index: Vec<u32>,
    pub ledger: Ledger,
}

impl Blockchain {
    pub fn zero() -> Self {
        Blockchain {
            index: vec![],
            ledger: Ledger::empty(),
        }
    }

    pub fn new(index: Vec<u32>, ledger: Ledger) -> Self {
        Blockchain {
            index: index,
            ledger: ledger,
        }
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }
    pub fn verify(&self, blocks: &Vec<Block>) -> Result<(), Error> {
        let mut last_index: u32 = 0;
        let mut last_hash: Sha256Hash = Sha256Hash::zero();

        for i in self.index.iter() {
            let block = &blocks[*i as usize];

            if block.is_genesis() && last_index == 0 {
                continue;
            }

            if block.content.index != last_index + 1 {
                return Err(Error::BlockIndexAreNotContiguous);
            } else if !Miner::check_difficulty(&block.hash, Merkle::DIFFICULTY) {
                return Err(Error::BlockProofOfWorkIsNotDone);
            } else if block.content.prev_block_hash != last_hash {
                return Err(Error::BlockPrevHashDoesNotMatch);
            }

            last_index += 1;
            last_hash = block.hash;
        }

        Ok(())
    }

    pub fn last<'a>(&self, blocks: &'a Vec<Block>) -> &'a Block {
        &blocks[*self.index.last().unwrap() as usize]
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

impl Writable for Blockchain {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<(), Error> {
        write_vec_u32(writer, &self.index) //
            .and_then(|_| write_struct(writer, &self.ledger))
    }
}

impl Readable for Blockchain {
    fn from_reader(reader: &mut dyn Read) -> Result<Self, Error> {
        let mut blockchain = Blockchain::zero();

        match read_vec_u32(reader, &mut blockchain.index)
            .and_then(|_| read_struct(reader, &mut blockchain.ledger))
        {
            Ok(_) => (),
            Err(_) => return Err(Error::InvalidFormat),
        };

        Ok(blockchain)
    }
}
