use crate::{
    block::{Block, BlockContent},
    error::Error,
    ledger::PartialLedger,
    merkle::Merkle,
    mining::Miner,
    network::{ask_merkle_at, broadcast_block, broadcast_transaction, listener, send_merkle_at},
    transaction::Transaction,
    user::PublicUser,
};
use core::time;
use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread::sleep,
};

pub struct NodeAsync {
    network_block_receiver: Receiver<Block>,
    ask_merkle_receiver: Receiver<String>,
    receive_merkle_receiver: Receiver<Merkle>,
    transaction_receiver: Receiver<Transaction>,

    mine_success_receiver: Receiver<Block>,
    mine_start_sender: Sender<BlockContent>,
    mine_stop_sender: Sender<()>,
}

impl NodeAsync {
    pub fn new(addr: Option<String>, difficulty: Option<u128>) -> Self {
        let (network_block_sender, network_block_receiver): (Sender<Block>, Receiver<Block>) =
            mpsc::channel();
        let (mine_success_sender, mine_success_receiver): (Sender<Block>, Receiver<Block>) =
            mpsc::channel();
        let (mine_start_sender, mine_start_receiver): (
            Sender<BlockContent>,
            Receiver<BlockContent>,
        ) = mpsc::channel();
        let (mine_stop_sender, mine_stop_receiver): (Sender<()>, Receiver<()>) = mpsc::channel();
        let (ask_merkle_sender, ask_merkle_receiver): (Sender<String>, Receiver<String>) =
            mpsc::channel();
        let (receive_merkle_sender, receive_merkle_receiver): (Sender<Merkle>, Receiver<Merkle>) =
            mpsc::channel();
        let (transaction_sender, transaction_receiver): (
            Sender<Transaction>,
            Receiver<Transaction>,
        ) = mpsc::channel();

        match addr {
            Some(v) => {
                listener(
                    v,
                    network_block_sender,
                    transaction_sender,
                    ask_merkle_sender,
                    receive_merkle_sender,
                )
                .ok();
            }
            None => (),
        };

        match difficulty {
            Some(v) => {
                Miner::async_mine(
                    v, //
                    mine_success_sender,
                    mine_start_receiver,
                    mine_stop_receiver,
                );
            }
            None => (),
        };

        NodeAsync {
            network_block_receiver: network_block_receiver,
            mine_success_receiver: mine_success_receiver,
            ask_merkle_receiver: ask_merkle_receiver,
            receive_merkle_receiver: receive_merkle_receiver,
            transaction_receiver: transaction_receiver,
            mine_start_sender: mine_start_sender,
            mine_stop_sender: mine_stop_sender,
        }
    }

    pub fn mining_start(&self, block_content: BlockContent) {
        self.mine_start_sender.send(block_content).ok();
    }
    pub fn mining_stop(&self) {
        self.mine_stop_sender.send(()).ok();
    }

    pub fn get_merkle(&self, addr: &String, callback_addr: String) -> Result<Merkle, Error> {
        match ask_merkle_at(addr, callback_addr) {
            Ok(_) => (),
            Err(_) => return Err(Error::FailToGetMerkle),
        };
        match self.receive_merkle_receiver.recv() {
            Ok(v) => Ok(v),
            Err(_) => Err(Error::FailToGetMerkle),
        }
    }
}

pub struct NodeCache {
    pub pending_transactions: Vec<Transaction>,
    pub partial_ledger: PartialLedger,
}

impl NodeCache {
    pub fn new() -> Self {
        NodeCache {
            pending_transactions: vec![],
            partial_ledger: PartialLedger::empty(),
        }
    }

    pub fn clear(&mut self) {
        self.pending_transactions.clear();
        self.partial_ledger = PartialLedger::empty();
    }

    pub fn push(&mut self, merkle: &Merkle, transaction: &Transaction) -> Result<(), Error> {
        let ledger = &merkle.main().ledger;
        match self.partial_ledger.apply_transaction(ledger, &transaction) {
            Ok(_) => (),
            Err(e) => return Err(e),
        };

        self.pending_transactions.push(transaction.clone());

        Ok(())
    }
}

pub fn create_genesis_node(addr: String, connected_addr: Vec<String>, public_user: PublicUser) {
    let mut merkle = Merkle::new_from_nothingness();

    let na = NodeAsync::new(Some(addr), Some(Merkle::DIFFICULTY));
    let mut nc = NodeCache::new();

    let block_content =
        BlockContent::new_from_pending_transaction(&nc.pending_transactions, &merkle, &public_user);
    na.mining_start(block_content);

    loop {
        match na.network_block_receiver.try_recv() {
            Ok(block) => {
                match merkle.add_block(block.clone()) {
                    Ok(_) => (),
                    Err(_) => continue,
                }
                na.mining_stop();
                broadcast_block(&block, &connected_addr);
                let block_content = BlockContent::new_from_pending_transaction(
                    &nc.pending_transactions,
                    &merkle,
                    &public_user,
                );

                nc.clear();
                na.mining_start(block_content);
            }
            Err(_) => (),
        };
        match na.mine_success_receiver.try_recv() {
            Ok(block) => {
                broadcast_block(&block, &connected_addr);
                match merkle.add_block(block.clone()) {
                    Ok(_) => (),
                    Err(_) => continue,
                }
                let block_content = BlockContent::new_from_pending_transaction(
                    &nc.pending_transactions,
                    &merkle,
                    &public_user,
                );

                nc.clear();
                na.mining_start(block_content);
            }
            Err(_) => (),
        };
        match na.transaction_receiver.try_recv() {
            Ok(transaction) => {
                match nc.push(&merkle, &transaction) {
                    Ok(_) => (),
                    Err(_) => continue,
                }
                broadcast_transaction(&transaction, &connected_addr);
            }
            Err(_) => (),
        };
        match na.ask_merkle_receiver.try_recv() {
            Ok(address) => {
                send_merkle_at(&merkle, &address).ok();
            }
            Err(_) => (),
        };
        match na.receive_merkle_receiver.try_recv() {
            Ok(_) => {
                // do nothing
            }
            Err(_) => (),
        };
        sleep(time::Duration::from_millis(10))
    }
}

pub fn create_full_node(addr: String, connected_addr: Vec<String>, public_user: PublicUser) {
    let na = NodeAsync::new(Some(addr.clone()), Some(Merkle::DIFFICULTY));
    let mut nc = NodeCache::new();

    let mut merkle = match na.get_merkle(&connected_addr[0], addr) {
        Ok(v) => v,
        Err(_) => return,
    };

    loop {
        match na.network_block_receiver.try_recv() {
            Ok(block) => {
                match merkle.add_block(block.clone()) {
                    Ok(_) => (),
                    Err(_) => continue,
                }
                na.mining_stop();
                broadcast_block(&block, &connected_addr);
                let block_content = BlockContent::new_from_pending_transaction(
                    &nc.pending_transactions,
                    &merkle,
                    &public_user,
                );

                nc.clear();
                na.mining_start(block_content);
            }
            Err(_) => (),
        };
        match na.mine_success_receiver.try_recv() {
            Ok(block) => {
                broadcast_block(&block, &connected_addr);
                match merkle.add_block(block.clone()) {
                    Ok(_) => (),
                    Err(_) => continue,
                }
                let block_content = BlockContent::new_from_pending_transaction(
                    &nc.pending_transactions,
                    &merkle,
                    &public_user,
                );

                nc.clear();
                na.mining_start(block_content);
            }
            Err(_) => (),
        };
        match na.transaction_receiver.try_recv() {
            Ok(transaction) => {
                match nc.push(&merkle, &transaction) {
                    Ok(_) => (),
                    Err(_) => continue,
                }
                broadcast_transaction(&transaction, &connected_addr);
            }
            Err(_) => (),
        };
        match na.ask_merkle_receiver.try_recv() {
            Ok(address) => {
                send_merkle_at(&merkle, &address).ok();
            }
            Err(_) => (),
        };
        match na.receive_merkle_receiver.try_recv() {
            Ok(_) => {
                // do nothing
            }
            Err(_) => (),
        };
        sleep(time::Duration::from_millis(10))
    }
}

pub fn create_debug_node(addr: String, connected_addr: Vec<String>) {
    let na = NodeAsync::new(Some(addr.clone()), None);
    let mut nc = NodeCache::new();

    let mut merkle = match na.get_merkle(&connected_addr[0], addr) {
        Ok(v) => v,
        Err(_) => return,
    };

    loop {
        match na.network_block_receiver.try_recv() {
            Ok(block) => {
                match merkle.add_block(block.clone()) {
                    Ok(_) => (),
                    Err(_) => {
                        continue;
                    }
                }
                broadcast_block(&block, &connected_addr);
                println!("{:?}", merkle.main().ledger);
                println!("the main blockchain size is: {}", merkle.main().len());
                println!("There are {} branch: ", merkle.chains.len());
            }
            Err(_) => {}
        };
        match na.mine_success_receiver.try_recv() {
            Ok(_) => {}
            Err(_) => (),
        };
        match na.transaction_receiver.try_recv() {
            Ok(transaction) => {
                match nc.push(&merkle, &transaction) {
                    Ok(_) => (),
                    Err(_) => continue,
                }
                broadcast_transaction(&transaction, &connected_addr);
            }
            Err(_) => (),
        };
        match na.ask_merkle_receiver.try_recv() {
            Ok(address) => {
                send_merkle_at(&merkle, &address).ok();
            }
            Err(_) => (),
        };
        match na.receive_merkle_receiver.try_recv() {
            Ok(_) => {
                // do nothing
            }
            Err(_) => (),
        };
        sleep(time::Duration::from_millis(10))
    }
}
