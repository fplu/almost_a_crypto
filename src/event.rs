// use crate::transaction::Transaction;

// use super::block::Block;
// use super::sha256::Sha256Hash;

// fn onRawTransactionReceived(bytes: &Vec<u8>) {
//     match Transaction::from_bytes(bytes) {
//         Ok(t) => onTransactionReceived(t),
//         Err(e) => (),
//     }
// }

// fn onTransactionReceived(transaction: Transaction) {}

// fn onRawBlockReceived(bytes: Vec<u8>) {}

// fn onBlockMined(nonce: u128, hash: Sha256Hash) {}

// fn onBlockReceived(block: Block) {}
