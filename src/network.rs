use crate::block::Block;
use crate::error::Error;
use crate::merkle::Merkle;
use crate::reader::read_string;
use crate::reader::read_u32;
use crate::reader::Readable;
use crate::transaction::Transaction;
use crate::writer::write_string;
use crate::writer::write_struct;
use crate::writer::write_u32;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::JoinHandle;

#[derive(Copy, Clone)]
pub enum PacketKind {
    None = 0,
    Transaction,
    Block,
    AskMerkle,
    ReceiveMerkle,
}

pub fn listener(
    addr: String,
    block_sender: Sender<Block>,
    transaction_sender: Sender<Transaction>,
    ask_merkle_sender: Sender<String>,
    receive_merkle_sender: Sender<Merkle>,
) -> Result<JoinHandle<()>, Error> {
    let listener = match TcpListener::bind(addr.clone()) {
        Ok(v) => v,
        Err(_) => {
            return Err(Error::TcpListenerBind);
        }
    };

    Ok(thread::spawn(move || {
        for stream in listener.incoming() {
            let mut stream = match stream {
                Ok(v) => v,
                Err(_) => continue,
            };
            let mut kind: u32 = 0;

            match read_u32(&mut stream, &mut kind) {
                Ok(_) => (),
                Err(_) => continue,
            };

            match kind {
                kind if kind == (PacketKind::Transaction as u32) => {
                    let transaction = match Transaction::from_reader(&mut stream) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };
                    match transaction_sender.send(transaction) {
                        Ok(_) => (),
                        Err(_) => continue,
                    };
                }
                kind if kind == (PacketKind::Block as u32) => {
                    let block = match Block::from_reader(&mut stream) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };

                    match block_sender.send(block) {
                        Ok(_) => (),
                        Err(_) => continue,
                    };
                }
                kind if kind == (PacketKind::AskMerkle as u32) => {
                    let mut callback_addr = String::new();

                    match read_string(&mut stream, &mut callback_addr) {
                        Ok(_) => (),
                        Err(_) => continue,
                    };

                    match ask_merkle_sender.send(callback_addr) {
                        Ok(_) => (),
                        Err(_) => continue,
                    };
                }
                kind if kind == (PacketKind::ReceiveMerkle as u32) => {
                    let merkle = match Merkle::from_reader(&mut stream) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };
                    match receive_merkle_sender.send(merkle) {
                        Ok(_) => (),
                        Err(_) => continue,
                    };
                }
                _ => continue,
            }
        }
    }))
}

pub fn send_transaction_at(transaction: &Transaction, addr: &String) -> Result<(), Error> {
    let mut stream = match TcpStream::connect(addr) {
        Ok(v) => v,
        Err(_) => return Err(Error::TcpStreamConnect),
    };

    let mut msg: Vec<u8> = vec![];

    write_u32(&mut msg, PacketKind::Transaction as u32)
        .and_then(|_| write_struct(&mut msg, transaction))
        .ok();

    match stream.write(&msg) {
        Ok(_) => (),
        Err(_) => return Err(Error::TcpFailToSend),
    };

    Ok(())
}

pub fn send_block_at(block: &Block, addr: &String) -> Result<(), Error> {
    let mut stream = match TcpStream::connect(addr) {
        Ok(v) => v,
        Err(_) => {
            return Err(Error::TcpStreamConnect);
        }
    };

    let mut msg: Vec<u8> = vec![];

    write_u32(&mut msg, PacketKind::Block as u32)
        .and_then(|_| write_struct(&mut msg, block))
        .ok();

    match stream.write(&msg) {
        Ok(_) => (),
        Err(_) => return Err(Error::TcpFailToSend),
    };

    Ok(())
}

pub fn send_merkle_at(merkle: &Merkle, addr: &String) -> Result<(), Error> {
    let mut stream = match TcpStream::connect(addr) {
        Ok(v) => v,
        Err(_) => {
            return Err(Error::TcpStreamConnect);
        }
    };

    let mut msg: Vec<u8> = vec![];

    write_u32(&mut msg, PacketKind::ReceiveMerkle as u32)
        .and_then(|_| write_struct(&mut msg, merkle))
        .ok();

    match stream.write(&msg) {
        Ok(_) => (),
        Err(_) => return Err(Error::TcpFailToSend),
    };

    Ok(())
}

pub fn ask_merkle_at(addr: &String, callback_addr: String) -> Result<(), Error> {
    let mut stream = match TcpStream::connect(addr) {
        Ok(v) => v,
        Err(_) => return Err(Error::TcpStreamConnect),
    };

    let mut msg: Vec<u8> = vec![];

    write_u32(&mut msg, PacketKind::AskMerkle as u32) //
        .and_then(|_| write_string(&mut msg, callback_addr))
        .ok();

    match stream.write(&msg) {
        Ok(_) => (),
        Err(_) => return Err(Error::TcpFailToSend),
    };

    Ok(())
}

pub fn broadcast_block(block: &Block, connected_addr: &Vec<String>) {
    for addr in connected_addr {
        send_block_at(&block, addr).ok();
    }
}

pub fn broadcast_transaction(transaction: &Transaction, connected_addr: &Vec<String>) {
    for addr in connected_addr {
        send_transaction_at(&transaction, addr).ok();
    }
}
