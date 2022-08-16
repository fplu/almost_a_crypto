use std::io::Read;

use ed25519_dalek::Signature;

use crate::error::Error;

pub fn read_u32(reader: &mut dyn Read, u: &mut u32) -> Result<(), Error> {
    const LEN: usize = (u32::BITS / 8) as usize;

    let mut slice: [u8; LEN] = [0; LEN];

    match reader.read_exact(&mut slice[..]) {
        Ok(_) => (),
        Err(_) => return Err(Error::EndOfBuffer),
    };
    let num = u32::from_be_bytes(slice);

    *u = num;
    Ok(())
}

pub fn read_u64(reader: &mut dyn Read, u: &mut u64) -> Result<(), Error> {
    const LEN: usize = (u64::BITS / 8) as usize;

    let mut slice: [u8; LEN] = [0; LEN];

    match reader.read_exact(&mut slice[..]) {
        Ok(_) => (),
        Err(_) => return Err(Error::EndOfBuffer),
    };
    let num = u64::from_be_bytes(slice);

    *u = num;
    Ok(())
}

pub fn read_usize(reader: &mut dyn Read, u: &mut usize) -> Result<(), Error> {
    const LEN: usize = (usize::BITS / 8) as usize;

    let mut slice: [u8; LEN] = [0; LEN];

    match reader.read_exact(&mut slice[..]) {
        Ok(_) => (),
        Err(_) => return Err(Error::EndOfBuffer),
    };
    let num = usize::from_be_bytes(slice);

    *u = num;
    Ok(())
}

pub fn read_u128(reader: &mut dyn Read, u: &mut u128) -> Result<(), Error> {
    const LEN: usize = (u128::BITS / 8) as usize;

    let mut slice: [u8; LEN] = [0; LEN];

    match reader.read_exact(&mut slice[..]) {
        Ok(_) => (),
        Err(_) => return Err(Error::EndOfBuffer),
    };
    let num = u128::from_be_bytes(slice);

    *u = num;
    Ok(())
}

pub fn read_struct<U>(reader: &mut dyn Read, s: &mut U) -> Result<(), Error>
where
    U: Readable,
{
    match U::from_reader(reader) {
        Ok(v) => {
            *s = v;
            Ok(())
        }
        Err(e) => return Err(e),
    }
}

pub fn read_vec_len(reader: &mut dyn Read, l: &mut usize) -> Result<(), Error> {
    let mut len_u32: u32 = 0;
    match read_u32(reader, &mut len_u32) {
        Ok(_) => {
            *l = len_u32 as usize;
            Ok(())
        }
        Err(e) => return Err(e),
    }
}

pub fn read_vec_u32(reader: &mut dyn Read, v: &mut Vec<u32>) -> Result<(), Error> {
    let mut len: usize = 0;
    match read_vec_len(reader, &mut len) {
        Ok(_) => (),
        Err(_) => return Err(Error::EndOfBuffer),
    };

    let mut vec: Vec<u32> = Vec::with_capacity(len);
    for _ in 0..len {
        let mut u: u32 = 0;
        match read_u32(reader, &mut u) {
            Ok(s) => s,
            Err(_) => return Err(Error::EndOfBuffer),
        };
        vec.push(u);
    }

    *v = vec;
    Ok(())
}

pub fn read_vec_u64(reader: &mut dyn Read, v: &mut Vec<u64>) -> Result<(), Error> {
    let mut len: usize = 0;
    match read_vec_len(reader, &mut len) {
        Ok(_) => (),
        Err(_) => return Err(Error::EndOfBuffer),
    };

    let mut vec: Vec<u64> = Vec::with_capacity(len);
    for _ in 0..len {
        let mut u: u64 = 0;
        match read_u64(reader, &mut u) {
            Ok(s) => s,
            Err(_) => return Err(Error::EndOfBuffer),
        };
        vec.push(u);
    }

    *v = vec;
    Ok(())
}

pub fn read_vec_u128(reader: &mut dyn Read, v: &mut Vec<u128>) -> Result<(), Error> {
    let mut len: usize = 0;
    match read_vec_len(reader, &mut len) {
        Ok(_) => (),
        Err(_) => return Err(Error::EndOfBuffer),
    };

    let mut vec: Vec<u128> = Vec::with_capacity(len);
    for _ in 0..len {
        let mut u: u128 = 0;
        match read_u128(reader, &mut u) {
            Ok(s) => s,
            Err(_) => return Err(Error::EndOfBuffer),
        };
        vec.push(u);
    }

    *v = vec;
    Ok(())
}

pub fn read_vec_struct<U>(reader: &mut dyn Read, v: &mut Vec<U>) -> Result<(), Error>
where
    U: Readable,
{
    let mut len: usize = 0;
    match read_vec_len(reader, &mut len) {
        Ok(_) => (),
        Err(_) => return Err(Error::EndOfBuffer),
    };

    let mut vec: Vec<U> = Vec::with_capacity(len);
    for _ in 0..len {
        let u = match U::from_reader(reader) {
            Ok(s) => s,
            Err(_) => return Err(Error::EndOfBuffer),
        };
        vec.push(u);
    }

    *v = vec;
    Ok(())
}

pub fn read_string(reader: &mut dyn Read, str: &mut String) -> Result<(), Error> {
    let mut len: usize = 0;
    match read_vec_len(reader, &mut len) {
        Ok(_) => (),
        Err(_) => return Err(Error::EndOfBuffer),
    };

    let mut slice = vec![0; len];

    match reader.read_exact(&mut slice[..]) {
        Ok(s) => s,
        Err(_) => return Err(Error::InvalidFormat),
    };
    let value_str = match String::from_utf8(slice) {
        Ok(str) => str,
        Err(_) => return Err(Error::InvalidFormat),
    };

    *str = value_str;
    Ok(())
}

pub fn read_signature(reader: &mut dyn Read, sign: &mut Signature) -> Result<(), Error> {
    const LEN: usize = Signature::BYTE_SIZE;

    let mut slice: [u8; LEN] = [0; LEN];

    match reader.read_exact(&mut slice[..]) {
        Ok(_) => (),
        Err(_) => return Err(Error::EndOfBuffer),
    };

    let s = match Signature::from_bytes(&slice) {
        Ok(s) => s,
        Err(_) => return Err(Error::EndOfBuffer),
    };

    *sign = s;
    Ok(())
}
pub trait Readable {
    fn from_reader(reader: &mut dyn Read) -> Result<Self, Error>
    where
        Self: Sized;
}
