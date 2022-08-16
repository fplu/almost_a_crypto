use std::fmt::{self, Debug, Formatter};
use std::io::Read;
use std::io::Write;
use std::vec;

use crate::reader::{read_u32, read_vec_struct, Readable};
use crate::writer::{write_u32, write_vec_struct, Writable};
use crate::{
    block::Block, blockchain::Blockchain, error::Error, ledger::Ledger, sha256::Sha256Hash,
};

#[derive(Clone, PartialEq, Eq)]
pub struct Merkle {
    pub blocks: Vec<Block>,
    pub chains: Vec<Blockchain>, // ordered index of the blockchain
    pub main: u32,
}

impl Merkle {
    pub const DIFFICULTY: u128 = 0xF1FF;

    pub fn zero() -> Self {
        Merkle {
            blocks: vec![],
            chains: vec![],
            main: 0,
        }
    }

    pub fn new_from_nothingness() -> Self {
        Merkle {
            blocks: vec![Block::new_genesis()],
            chains: vec![Blockchain::new(vec![0], Ledger::empty())],
            main: 0,
        }
    }

    pub fn main(&self) -> &Blockchain {
        &self.chains[self.main as usize]
    }

    fn contains(&self, block: &Block) -> bool {
        self.blocks.contains(block)
    }

    fn find_block_by_hash(&self, hash: Sha256Hash) -> Result<usize, Error> {
        for (i, block) in self.blocks.iter().enumerate() {
            if block.hash == hash {
                return Ok(i);
            }
        }
        return Err(Error::NotFound);
    }

    fn find_or_create_blockchain(
        &self,
        block: &Block,
    ) -> Result<(Blockchain, Option<usize>), Error> {
        for (i, chain) in self.chains.iter().enumerate() {
            if chain.last(&self.blocks).hash == block.content.prev_block_hash {
                return Ok((chain.clone(), Some(i)));
            }
        }
        let index = match self.find_block_by_hash(block.content.prev_block_hash) {
            Ok(i) => i,
            Err(_) => return Err(Error::PreviousBlockNotFound),
        };

        for chain in self.chains.iter() {
            for (i, j) in chain.index.iter().enumerate() {
                if (*j as usize) == index {
                    let new_chain = chain.index[0..i + 1].to_vec();

                    let new_ledger = Ledger::new_unsafe(&new_chain, &self.blocks);
                    let new_blockchain = Blockchain::new(new_chain, new_ledger);
                    return Ok((new_blockchain, None));
                }
            }
        }

        return Err(Error::BlockExistButIsNotInAnyBranch);
    }

    pub fn add_block(&mut self, block: Block) -> Result<(), Error> {
        if self.contains(&block) {
            return Err(Error::BlockAlreadyExist); // do nothing
        }

        match block.verify(Merkle::DIFFICULTY) {
            Ok(_) => (),
            Err(e) => return Err(e),
        }

        let (blockchain, index) = match self.find_or_create_blockchain(&block) {
            Ok(lb) => lb,
            Err(e) => return Err(e),
        };

        let last_valid_block = blockchain.last(&self.blocks);
        if last_valid_block.content.index + 1 != block.content.index {
            return Err(Error::BlockIndexAreNotContiguous);
        }
        if last_valid_block.hash != block.content.prev_block_hash {
            return Err(Error::BlockPrevHashDoesNotMatch);
        }

        let new_ledger = match block.verify_payload(&blockchain.ledger) {
            Ok(l) => (l),
            Err(e) => return Err(e),
        };

        self.blocks.push(block.clone());
        let block_index = self.blocks.len() - 1;

        let chain_index = match index {
            Some(i) => i,
            None => {
                self.chains.push(blockchain.clone());
                self.chains.len() - 1
            }
        };

        self.chains[chain_index].index.push(block_index as u32);
        self.chains[chain_index].ledger = new_ledger;

        if self.chains[chain_index].len() > self.main().len() {
            self.main = chain_index as u32
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

impl Writable for Merkle {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<(), Error> {
        write_vec_struct(writer, &self.blocks)
            .and_then(|_| write_vec_struct(writer, &self.chains))
            .and_then(|_| write_u32(writer, self.main as u32))
    }
}

impl Readable for Merkle {
    fn from_reader(reader: &mut dyn Read) -> Result<Self, Error> {
        let mut merkle = Merkle::zero();

        match read_vec_struct(reader, &mut merkle.blocks)
            .and_then(|_| read_vec_struct(reader, &mut merkle.chains))
            .and_then(|_| read_u32(reader, &mut merkle.main))
        {
            Ok(_) => (),
            Err(_) => return Err(Error::InvalidFormat),
        };

        Ok(merkle)
    }
}

impl Debug for Merkle {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "<Merkle>",)
    }
}
