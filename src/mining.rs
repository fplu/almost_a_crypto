use crate::block::BlockContent;

use super::sha256::Sha256Hash;
use rand::Rng;
use std::thread::{self, JoinHandle};
type OnMined = fn(BlockContent, u128, Sha256Hash);
pub struct Miner {
    block_content: BlockContent,
    on_mined: OnMined,
    difficulty: u128,
}

impl Miner {
    pub fn new(block_content: BlockContent, on_mined: OnMined, difficulty: u128) -> Self {
        Miner {
            block_content: block_content,
            on_mined: on_mined,
            difficulty: difficulty,
        }
    }

    pub fn mine(block_content: &BlockContent, difficulty: u128) -> (u128, Sha256Hash) {
        let bytes = block_content.to_bytes();
        let mut rng = rand::thread_rng();

        for _ in 0..(u128::max_value()) {
            let proof_of_work = rng.gen_range(0, u128::max_value());
            let hash = Sha256Hash::new(&[&bytes, &proof_of_work.to_be_bytes().to_vec()]);
            if Miner::check_difficulty(&hash, difficulty) {
                return (proof_of_work, hash);
            }
        }

        panic!("unreachable code!");
    }

    pub fn start_mining(&self) -> JoinHandle<()> {
        let on_mined = self.on_mined;
        let difficulty = self.difficulty;
        let block_content = self.block_content.clone();

        thread::spawn(move || {
            let (nonce, hash) = Miner::mine(&block_content, difficulty);
            (on_mined)(block_content, nonce, hash);
        })
    }

    pub fn check_difficulty(hash: &Sha256Hash, difficulty: u128) -> bool {
        (hash.as_u128()[0] & difficulty) == 0
    }
}
