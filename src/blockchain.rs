use crate::error::Error;
use crate::ledger::Ledger;
use crate::merkle::Merkle;
use crate::reader::{Readable, Reader};
use crate::sha256::Sha256Hash;

use super::block::Block;
use super::mining::Miner;

#[derive(Clone, PartialEq, Eq)]
pub struct Blockchain {
    pub index: Vec<usize>,
    pub ledger: Ledger,
}

impl Blockchain {
    pub fn new(index: Vec<usize>, ledger: Ledger) -> Self {
        Blockchain {
            index: index,
            ledger: ledger,
        }
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }
    pub fn verify(&self, blocks: &Vec<Block>) -> bool {
        let mut last_index: usize = 0;
        let mut last_hash: Sha256Hash = Sha256Hash::zero();

        for i in self.index.iter() {
            let block = &blocks[*i];

            if block.is_genesis() && last_index == 0 {
                continue;
            }

            if block.content.index != last_index + 1 {
                println!("Index mismatch {} != {}", &block.content.index, &i);
                return false;
            } else if !Miner::check_difficulty(&block.hash, Merkle::DIFFICULTY) {
                println!("Error hash not mined");
                return false;
            } else if block.content.prev_block_hash != last_hash {
                println!("previous block hash do not match");
                return false;
            }

            last_index += 1;
            last_hash = block.hash;
        }

        true
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        bytes.extend((self.index.len() as u32).to_be_bytes());
        for elem in &self.index {
            bytes.extend((*elem as u32).to_be_bytes())
        }
        bytes.extend(self.ledger.to_bytes());

        bytes
    }
    pub fn from_bytes(bytes: &Vec<u8>) -> Result<Self, Error> {
        let mut reader = Reader::new(bytes.clone());

        Self::from_reader(&mut reader)
    }
}

impl Readable<Blockchain> for Blockchain {
    fn from_reader(reader: &mut Reader) -> Result<Self, Error> {
        let index = match reader.read_vec_u32() {
            Ok(s) => s as Vec<u32>,
            Err(_) => return Err(Error::InvalidFormat),
        }
        .iter()
        .map(|x| *x as usize)
        .collect();

        let ledger = match reader.read() {
            Ok(v) => v,
            Err(_) => return Err(Error::InvalidFormat),
        };

        Ok(Blockchain {
            index: index,
            ledger: ledger,
        })
    }
}
