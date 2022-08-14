use std::vec;

use crate::block::Block;
use crate::error::Error;
use crate::num::Num;
use crate::reader::{Readable, Reader};
use crate::transaction::Transaction;
use crate::user::PublicUser;

#[derive(Clone, PartialEq, Eq)]
pub struct UserData {
    pub user: PublicUser,
    pub money: Num,
}
#[derive(Clone, PartialEq, Eq)]
pub struct Ledger {
    pub users: Vec<UserData>,
    pub nonces_transaction: Vec<u64>,
}

pub struct PartialLedger {
    pub users: Vec<UserData>,
    pub nonces_transaction: Vec<u64>,
}

impl UserData {
    pub fn new(user: PublicUser) -> Self {
        UserData {
            user: user,
            money: Num::zero(),
        }
    }
}

impl Ledger {
    pub fn empty() -> Self {
        Ledger {
            users: vec![],
            nonces_transaction: vec![],
        }
    }

    pub fn new_unsafe(chain: &Vec<usize>, blocks: &Vec<Block>) -> Self {
        let ledger = Ledger::empty();
        let mut partial_ledger = PartialLedger::empty();

        for i in chain {
            let block = &blocks[*i];

            for transaction in &block.content.transactions {
                partial_ledger.apply_transaction(&ledger, &transaction).ok();
            }
        }
        partial_ledger.to_ledger(&ledger)
    }

    // pub fn verify_and_actualize(
    //     &mut self,
    //     blockchain: Blockchain,
    //     difficulty: u128,
    // ) -> Result<bool, bool> {
    //     let mut is_new_block = false;
    //     let mut nonces: Vec<u128> = vec![];
    //     for block in blockchain.blocks {
    //         if block.content.prev_block_hash == self.last_block_hash {
    //             is_new_block = true;
    //         }

    //         if !is_new_block {
    //             continue;
    //         }

    //         match block.verify_and_actualize(difficulty, &nonces, self) {
    //             Ok(b) => (),
    //             Err(b) => return Err(b),
    //         }
    //         nonces.push(block.nonce);
    //     }

    //     return Ok(true);
    // }

    pub fn find_user_data(&self, public_user: PublicUser) -> Option<usize> {
        for (i, elem) in self.users.iter().enumerate() {
            if elem.user == public_user {
                return Some(i);
            }
        }
        return None;
    }

    pub fn find_or_create_user_data(&mut self, public_user: PublicUser) -> usize {
        match self.find_user_data(public_user) {
            Some(i) => return i,
            None => {
                self.users.push(UserData::new(public_user));
                return self.users.len() - 1;
            }
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        bytes.extend((self.nonces_transaction.len() as u32).to_be_bytes());
        for nonce in &self.nonces_transaction {
            bytes.extend(nonce.to_be_bytes())
        }
        bytes.extend((self.users.len() as u32).to_be_bytes());
        for user in &self.users {
            bytes.extend(user.to_bytes())
        }

        bytes
    }

    pub fn from_bytes(bytes: &Vec<u8>) -> Result<Self, Error> {
        let mut reader = Reader::new(bytes.clone());

        Self::from_reader(&mut reader)
    }
}

impl Readable<Ledger> for Ledger {
    fn from_reader(reader: &mut Reader) -> Result<Self, Error> {
        let nonces_transaction = match reader.read_vec_u64() {
            Ok(s) => s,
            Err(_) => return Err(Error::InvalidFormat),
        };

        let users = match reader.read_vec() {
            Ok(s) => s,
            Err(_) => return Err(Error::InvalidFormat),
        };

        Ok(Ledger {
            users: users,
            nonces_transaction: nonces_transaction,
        })
    }
}
impl UserData {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        bytes.extend(self.user.to_bytes());
        bytes.extend(self.money.to_bytes());

        bytes
    }

    pub fn from_bytes(bytes: &Vec<u8>) -> Result<Self, Error> {
        let mut reader = Reader::new(bytes.clone());

        Self::from_reader(&mut reader)
    }
}

impl Readable<UserData> for UserData {
    fn from_reader(reader: &mut Reader) -> Result<Self, Error> {
        let user = match reader.read() {
            Ok(s) => s,
            Err(_) => return Err(Error::InvalidFormat),
        };

        let money = match reader.read() {
            Ok(m) => m,
            Err(_) => return Err(Error::InvalidFormat),
        };

        return Ok(UserData {
            user: user,
            money: money,
        });
    }
}

impl PartialLedger {
    pub fn empty() -> Self {
        PartialLedger {
            users: vec![],
            nonces_transaction: vec![],
        }
    }

    pub fn contains(&self, ledger: &Ledger, transaction: &Transaction) -> bool {
        if self.nonces_transaction.contains(&transaction.content.nonce) {
            return true;
        }

        if ledger
            .nonces_transaction
            .contains(&transaction.content.nonce)
        {
            return true;
        }

        return false;
    }

    pub fn find_user_data(&mut self, ledger: &Ledger, public_user: PublicUser) -> Option<usize> {
        for (i, elem) in self.users.iter().enumerate() {
            if elem.user == public_user {
                return Some(i);
            }
        }

        match ledger.find_user_data(public_user) {
            Some(i) => {
                self.users.push(ledger.users[i].clone());
                return Some(self.users.len() - 1);
            }
            None => (),
        }
        return None;
    }

    pub fn find_or_create_user_data(&mut self, ledger: &Ledger, public_user: PublicUser) -> usize {
        match self.find_user_data(ledger, public_user) {
            Some(i) => return i,
            None => match ledger.find_user_data(public_user) {
                Some(i) => {
                    self.users.push(ledger.users[i].clone());
                    return self.users.len() - 1;
                }
                None => {
                    self.users.push(UserData::new(public_user));
                    return self.users.len() - 1;
                }
            },
        }
    }

    pub fn apply_transaction(
        &mut self,
        ledger: &Ledger,
        transaction: &Transaction,
    ) -> Result<(), Error> {
        if self.contains(ledger, transaction) {
            return Err(Error::TransactionWasAlreadyDone);
        }
        let value = &transaction.content.value;

        if !transaction.content.from.is_coinbase() {
            let user_data_index = match self.find_user_data(ledger, transaction.content.from) {
                Some(data) => data,
                None => return Err(Error::TryingToSendMoneyFromUnknowUser),
            };

            let money = self.users[user_data_index].money.clone();
            match transaction.verify(money) {
                Ok(_) => (),
                Err(e) => return Err(e),
            }

            self.users[user_data_index].money -= value.clone();
        }

        self.nonces_transaction.push(transaction.content.nonce);
        let to_user_data_index = self.find_or_create_user_data(ledger, transaction.content.to);
        self.users[to_user_data_index].money += value.clone();

        return Ok(());
    }

    pub fn to_ledger(&self, ledger: &Ledger) -> Ledger {
        let mut res = Ledger {
            nonces_transaction: self.nonces_transaction.clone(),
            users: self.users.clone(),
        };

        for elem in &ledger.nonces_transaction {
            if !res.nonces_transaction.contains(&elem) {
                res.nonces_transaction.push(elem.clone());
            }
        }

        for a in &ledger.users {
            let mut find = false;
            for b in &res.users {
                if b.user == a.user {
                    find = true;
                    break;
                }
            }

            if !find {
                res.users.push(a.clone());
            }
        }

        res
    }
}
