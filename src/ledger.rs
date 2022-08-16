use std::fmt::{self, Debug, Formatter};
use std::io::Read;
use std::io::Write;
use std::vec;

use crate::block::Block;
use crate::error::Error;
use crate::num::Num;
use crate::reader::{read_struct, read_vec_struct, read_vec_u64, Readable};
use crate::transaction::Transaction;
use crate::user::PublicUser;
use crate::writer::{write_struct, write_vec_struct, write_vec_u64, Writable};

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

    pub fn new_unsafe(chain: &Vec<u32>, blocks: &Vec<Block>) -> Self {
        let ledger = Ledger::empty();
        let mut partial_ledger = PartialLedger::empty();

        for i in chain {
            let block = &blocks[*i as usize];

            for transaction in &block.content.transactions {
                partial_ledger.apply_transaction(&ledger, &transaction).ok();
            }
        }
        partial_ledger.to_ledger(&ledger)
    }

    pub fn contains(&self, transaction: &Transaction) -> bool {
        if self.nonces_transaction.contains(&transaction.content.nonce) {
            return true;
        }
        return false;
    }

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

        self.to_writer(&mut bytes).ok();

        bytes
    }

    pub fn from_bytes(bytes: &Vec<u8>) -> Result<Self, Error> {
        let mut slice: &[u8] = bytes;
        Self::from_reader(&mut slice)
    }
}

impl Debug for Ledger {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "________________________\nLedger\n________________________\n"
        )
        .ok();
        for user in &self.users {
            write!(f, "{:?}: {:?} coins\n", user.user, user.money).ok();
        }
        write!(f, "________________________\n")
    }
}

impl Writable for Ledger {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<(), Error> {
        write_vec_u64(writer, &self.nonces_transaction)
            .and_then(|_| write_vec_struct(writer, &self.users))
    }
}

impl Readable for Ledger {
    fn from_reader(reader: &mut dyn Read) -> Result<Self, Error> {
        let mut ledger = Ledger::empty();

        match read_vec_u64(reader, &mut ledger.nonces_transaction)
            .and_then(|_| read_vec_struct(reader, &mut ledger.users))
        {
            Ok(_) => (),
            Err(_) => return Err(Error::InvalidFormat),
        };

        Ok(ledger)
    }
}
impl UserData {
    pub fn zero() -> Self {
        UserData {
            user: PublicUser::zero(),
            money: Num::zero(),
        }
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

impl Writable for UserData {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<(), Error> {
        write_struct(writer, &self.user) //
            .and_then(|_| write_struct(writer, &self.money))
    }
}

impl Readable for UserData {
    fn from_reader(reader: &mut dyn Read) -> Result<Self, Error> {
        let mut user_data = UserData::zero();

        match read_struct(reader, &mut user_data.user) //
            .and_then(|_| read_struct(reader, &mut user_data.money))
        {
            Ok(m) => m,
            Err(_) => return Err(Error::InvalidFormat),
        };

        Ok(user_data)
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

        if ledger.contains(&transaction) {
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
