use crate::{
    block::{Block, BlockContent},
    error::Error,
};

use super::sha256::Sha256Hash;
use core::time;
use rand::Rng;
use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, sleep, JoinHandle},
};
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

    pub fn interuptable_mining(
        block_content: BlockContent,
        difficulty: u128,
        stop: Receiver<()>,
    ) -> Result<(u128, Sha256Hash), Error> {
        let bytes = block_content.to_bytes();
        let mut rng = rand::thread_rng();

        loop {
            for _ in 0..2048 {
                let proof_of_work = rng.gen_range(0, u128::max_value());
                let hash = Sha256Hash::new(&[&bytes, &proof_of_work.to_be_bytes().to_vec()]);
                if Miner::check_difficulty(&hash, difficulty) {
                    return Ok((proof_of_work, hash));
                }
            }
            match stop.try_recv() {
                Ok(_) => return Err(Error::MiningInterupted),
                Err(_) => (),
            }
        }
    }

    pub fn async_mine(
        difficulty: u128,
        result: Sender<Block>,
        start: Receiver<BlockContent>,
        stop: Receiver<()>,
    ) {
        thread::spawn(move || {
            let (mut stopis, mut stopri) = mpsc::channel();
            let (mut resultis, mut resultir) = mpsc::channel();

            _ = resultis;
            _ = stopri;

            loop {
                match resultir.try_recv() {
                    Ok(b) => {
                        result.send(b).ok();
                    }
                    Err(_) => (),
                }
                match stop.try_recv() {
                    Ok(_) => {
                        stopis.send(()).ok();
                    }
                    Err(_) => (),
                }
                match start.try_recv() {
                    Ok(block_content) => {
                        stopis.send(()).ok();
                        (stopis, stopri) = mpsc::channel();
                        (resultis, resultir) = mpsc::channel();

                        thread::spawn(move || {
                            let (proof_of_work, hash) = match Miner::interuptable_mining(
                                block_content.clone(),
                                difficulty,
                                stopri,
                            ) {
                                Ok(v) => v,
                                Err(_) => return,
                            };
                            resultis
                                .send(Block::new_mined(block_content, proof_of_work, hash))
                                .ok();
                        });
                    }
                    Err(_) => (),
                }
                sleep(time::Duration::from_millis(10))
            }
        });
    }

    pub fn start_mining(&self) -> JoinHandle<()> {
        let on_mined = self.on_mined;
        let difficulty = self.difficulty;
        let block_content = self.block_content.clone();

        thread::spawn(move || {
            let (proof_of_work, hash) = Miner::mine(&block_content, difficulty);
            (on_mined)(block_content, proof_of_work, hash);
        })
    }

    pub fn check_difficulty(hash: &Sha256Hash, difficulty: u128) -> bool {
        (hash.as_u128()[0] & difficulty) == 0
    }
}
