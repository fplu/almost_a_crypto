use blockchain::block::{Block, BlockContent};
use blockchain::error::Error;
use blockchain::merkle::Merkle;
use blockchain::mining::Miner;
use blockchain::sha256::Sha256Hash;
use blockchain::time::Time;
use blockchain::user::User;
use blockchain::{num::Num, transaction::Transaction};

fn block_mining_on_mined_event(block_content: BlockContent, nonce: u128, sha: Sha256Hash) {
    let hash = Sha256Hash::new(&[&block_content.to_bytes(), &nonce.to_be_bytes().to_vec()]);
    assert_eq!(sha, hash, "mining failed, incorrect hash");

    let difficulty: u128 = Merkle::DIFFICULTY;

    assert!(
        Miner::check_difficulty(&hash, difficulty),
        "mining failed, difficulty not meet {:?}",
        hash
    )
}

#[test]
fn block_mining() {
    let from = User::from_nothingness();
    let to: User = User::from_nothingness();

    let transaction = Transaction::new(from, to.as_public(), Num::from_u64(10), 1);
    let payload: Vec<Transaction> = vec![transaction];
    let block_content = BlockContent::new(1, Time::from_second(0), Sha256Hash::zero(), payload);

    let difficulty: u128 = Merkle::DIFFICULTY;
    let miner = Miner::new(block_content, block_mining_on_mined_event, difficulty);

    let handle = miner.start_mining();
    handle.join().unwrap();
}

#[test]
fn merkle_add_block() {
    let mut merkle = Merkle::new_from_nothingness();

    let from = User::new_coinbase();
    let to: User = User::from_nothingness();

    let transaction = Transaction::new(from, to.as_public(), Num::from_u64(10), 1);
    let payload: Vec<Transaction> = vec![transaction];
    let block_content = BlockContent::new(1, Time::from_second(0), Sha256Hash::zero(), payload);

    let difficulty: u128 = 0xF8;
    let (nonce, hash) = Miner::mine(&block_content, difficulty);
    let block = Block::new_mined(block_content, nonce, hash);

    match merkle.add_block(block) {
        Ok(_) => (),
        Err(e) => panic!("failed to add block to merkle tree: error: {:?}", e),
    }

    assert!(
        merkle.main().len() == 2,
        "failed to add block to merkle tree, merkle main blockchain lenght is {}",
        merkle.main().len()
    )
}

#[test]
fn merkle_add_block_from_invalid_user() {
    let mut merkle = Merkle::new_from_nothingness();

    let from = User::from_nothingness();
    let to: User = User::from_nothingness();

    let transaction = Transaction::new(from, to.as_public(), Num::from_u64(10), 1);
    let payload: Vec<Transaction> = vec![transaction];
    let block_content = BlockContent::new(1, Time::from_second(0), Sha256Hash::zero(), payload);

    let difficulty: u128 = 0xF8;
    let (nonce, hash) = Miner::mine(&block_content, difficulty);
    let block = Block::new_mined(block_content, nonce, hash);

    match merkle.add_block(block) {
        Ok(_) => panic!("transaction was valided despite sender do not exist"),
        Err(e) => {
            if e != Error::TryingToSendMoneyFromUnknowUser {
                panic!("failed to add block to merkle tree: error: {:?}, error should be TryingToSendMoneyFromUnknowUser", e);
            }
        }
    }
}

#[test]
fn merkle_add_block_no_transaction() {
    let mut merkle = Merkle::new_from_nothingness();

    let payload: Vec<Transaction> = vec![];
    let block_content = BlockContent::new(1, Time::from_second(0), Sha256Hash::zero(), payload);

    let difficulty: u128 = 0xF8;
    let (nonce, hash) = Miner::mine(&block_content, difficulty);
    let block = Block::new_mined(block_content, nonce, hash);

    match merkle.add_block(block) {
        Ok(_) => panic!("transaction was valided despite sender do not exist"),
        Err(e) => {
            if e != Error::BlockContainsNoTransaction {
                panic!("failed to add block to merkle tree: error: {:?}, error should be BlockContainsNoTransaction", e);
            }
        }
    }
}

fn block_with_transaction(
    transaction_nonce: u64,
    index: usize,
    prev_block_hash: Sha256Hash,
) -> Block {
    let from = User::new_coinbase();
    let to: User = User::from_nothingness();

    let transaction = Transaction::new(from, to.as_public(), Num::from_u64(10), transaction_nonce);
    let payload: Vec<Transaction> = vec![transaction];
    let block_content = BlockContent::new(index, Time::from_second(0), prev_block_hash, payload);

    let difficulty: u128 = Merkle::DIFFICULTY;
    let (nonce, hash) = Miner::mine(&block_content, difficulty);
    let block = Block::new_mined(block_content, nonce, hash);

    block
}

#[test]
fn merkle_branch() {
    let mut merkle = Merkle::new_from_nothingness();

    let c1_b1 = block_with_transaction(1, 1, Sha256Hash::zero());
    let c1_b2 = block_with_transaction(2, 2, c1_b1.hash);
    let c1_b3 = block_with_transaction(3, 3, c1_b2.hash);

    let c2_b1 = block_with_transaction(4, 1, Sha256Hash::zero());
    let c2_b2 = block_with_transaction(5, 2, c2_b1.hash);
    let c2_b3 = block_with_transaction(6, 3, c2_b2.hash);
    let c2_b4 = block_with_transaction(7, 4, c2_b3.hash);

    match merkle.add_block(c1_b1) {
        Ok(_) => (),
        Err(e) => panic!("failed to add block c1_b1 to merkle tree: error: {:?}", e),
    }
    assert!(
        merkle.main().len() == 2,
        "Error main branch size, expected 2 but get: {}",
        merkle.main().len(),
    );
    match merkle.add_block(c1_b2) {
        Ok(_) => (),
        Err(e) => panic!("failed to add block c1 b2 to merkle tree: error: {:?}", e),
    }
    assert!(
        merkle.main().len() == 3,
        "Error main branch size, expected 3 but get: {}",
        merkle.main().len(),
    );
    match merkle.add_block(c1_b3) {
        Ok(_) => (),
        Err(e) => panic!("failed to add block c1 b3 to merkle tree: error: {:?}", e),
    }
    assert!(
        merkle.main().len() == 4,
        "Error main branch size, expected 4 but get: {}",
        merkle.main().len(),
    );
    assert!(
        merkle.main == 0,
        "Error invalid main branch, should be 0 but is {}",
        merkle.main
    );
    assert!(
        merkle.chains.len() == 1,
        "Error invalid too much branch, expected 1 but get: {}",
        merkle.chains.len()
    );

    match merkle.add_block(c2_b1) {
        Ok(_) => (),
        Err(e) => panic!("failed to add block c2 b1 to merkle tree: error: {:?}", e),
    }
    assert!(
        merkle.chains.len() == 2,
        "Error invalid merkle should have 2 branch but has {}",
        merkle.chains.len()
    );
    match merkle.add_block(c2_b2) {
        Ok(_) => (),
        Err(e) => panic!("failed to add block c2 b2 to merkle tree: error: {:?}", e),
    }
    match merkle.add_block(c2_b3) {
        Ok(_) => (),
        Err(e) => panic!("failed to add block c2 b3 to merkle tree: error: {:?}", e),
    }
    assert!(
        merkle.main == 0,
        "Error invalid main branch, should be still be 0 but is {}",
        merkle.main,
    );
    match merkle.add_block(c2_b4) {
        Ok(_) => (),
        Err(e) => panic!("failed to add block c2 b4 to merkle tree: error: {:?}", e),
    }
    assert!(
        merkle.main == 1,
        "Error invalid main branch, should be 1 but is {}",
        merkle.main
    );
    assert!(
        merkle.chains.len() == 2,
        "Error invalid merkle should have 2 branch but has {}",
        merkle.chains.len()
    );
}

#[test]
fn invalid_blockchain() {
    let mut merkle = Merkle::new_from_nothingness();

    let c1_b1 = block_with_transaction(1, 1, Sha256Hash::zero());
    let c1_b2 = block_with_transaction(1, 1, c1_b1.hash);

    match merkle.add_block(c1_b1) {
        Ok(_) => (),
        Err(e) => panic!("failed to add block c1_b1 to merkle tree: error: {:?}", e),
    }
    assert!(
        merkle.main().len() == 2,
        "Error main branch size, expected 2 but get: {}",
        merkle.main().len(),
    );
    match merkle.add_block(c1_b2) {
        Ok(_) => panic!("should fail with Error::BlockIndexAreNotContiguous but succeed"),
        Err(e) => assert!(
            e == Error::BlockIndexAreNotContiguous,
            "should fail with Error::BlockIndexAreNotContiguous but get: {:?}",
            e
        ),
    }
}
