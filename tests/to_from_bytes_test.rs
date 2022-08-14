use blockchain::block::{Block, BlockContent};
use blockchain::merkle::Merkle;
use blockchain::mining::Miner;
use blockchain::sha256::Sha256Hash;
use blockchain::time::Time;
use blockchain::user::User;
use blockchain::{num::Num, transaction::Transaction};

#[test]
fn num_to_bytes_from_bytes() {
    let original = Num::from_u64(10);
    let original_bytes = original.to_bytes();
    let reconstructed = match Num::from_bytes(&original_bytes) {
        Ok(v) => v,
        Err(_) => panic!("Num::from_bytes failed to complete"),
    };

    assert_eq!(
        reconstructed, original,
        "Num::from_bytes completed incorrectly"
    )
}

#[test]
fn transaction_to_bytes_from_bytes() {
    let from = User::from_nothingness();
    let to: User = User::from_nothingness();

    let original = Transaction::new(from, to.as_public(), Num::from_u64(10), 1);
    let original_as_bytes = original.to_bytes();

    let reconstructed = match Transaction::from_bytes(&original_as_bytes) {
        Ok(v) => v,
        Err(_) => panic!("Transaction::from_bytes failed to complete"),
    };

    assert_eq!(
        reconstructed, original,
        "Transaction::from_bytes completed incorrectly"
    )
}

#[test]
fn block_to_bytes_from_bytes() {
    let from = User::from_nothingness();
    let to: User = User::from_nothingness();

    let transaction = Transaction::new(from, to.as_public(), Num::from_u64(10), 1);
    let payload: Vec<Transaction> = vec![transaction];
    let original = Block::new(1, Time::from_second(0), Sha256Hash::zero(), payload);
    let original_as_bytes = original.to_bytes();

    let reconstructed = match Block::from_bytes(&original_as_bytes) {
        Ok(v) => v,
        Err(_) => panic!("Block::from_bytes failed to complete"),
    };

    assert_eq!(
        reconstructed, original,
        "Block::from_bytes completed incorrectly"
    )
}

#[test]
fn merkle_to_byte_from_byte() {
    let mut original = Merkle::new_from_nothingness();

    let from = User::new_coinbase();
    let to: User = User::from_nothingness();

    let transaction = Transaction::new(from, to.as_public(), Num::from_u64(10), 1);
    let payload: Vec<Transaction> = vec![transaction];
    let block_content = BlockContent::new(1, Time::from_second(0), Sha256Hash::zero(), payload);

    let difficulty: u128 = 0xF8;
    let (nonce, hash) = Miner::mine(&block_content, difficulty);
    let block = Block::new_mined(block_content, nonce, hash);

    original.add_block(block).ok();

    let bytes = original.to_bytes();
    let reconstructed = match Merkle::from_bytes(&bytes) {
        Ok(v) => v,
        Err(_) => panic!("Block::from_bytes failed to complete"),
    };

    assert_eq!(
        reconstructed, original,
        "Block::from_bytes completed incorrectly"
    )
}
