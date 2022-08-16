use std::io::Write;

use ed25519_dalek::Signature;

use crate::error::Error;

pub fn write_u32(writer: &mut dyn Write, u: u32) -> Result<(), Error> {
    match writer.write_all(&u.to_be_bytes()) {
        Ok(_) => Ok(()),
        Err(_) => return Err(Error::EndOfBuffer),
    }
}

pub fn write_u64(writer: &mut dyn Write, u: u64) -> Result<(), Error> {
    match writer.write_all(&u.to_be_bytes()) {
        Ok(_) => Ok(()),
        Err(_) => return Err(Error::EndOfBuffer),
    }
}

pub fn write_u128(writer: &mut dyn Write, u: u128) -> Result<(), Error> {
    match writer.write_all(&u.to_be_bytes()) {
        Ok(_) => Ok(()),
        Err(_) => return Err(Error::EndOfBuffer),
    }
}

pub fn write_struct<U>(writer: &mut dyn Write, s: &U) -> Result<(), Error>
where
    U: Writable,
{
    s.to_writer(writer)
}

pub fn write_vec_u32(writer: &mut dyn Write, v: &Vec<u32>) -> Result<(), Error> {
    match write_u32(writer, v.len() as u32) {
        Ok(_) => (),
        Err(_) => return Err(Error::EndOfBuffer),
    };

    for elem in v {
        match write_u32(writer, *elem) {
            Ok(_) => (),
            Err(_) => return Err(Error::EndOfBuffer),
        };
    }

    Ok(())
}

pub fn write_vec_u64(writer: &mut dyn Write, v: &Vec<u64>) -> Result<(), Error> {
    match write_u32(writer, v.len() as u32) {
        Ok(_) => (),
        Err(_) => return Err(Error::EndOfBuffer),
    };

    for elem in v {
        match write_u64(writer, *elem) {
            Ok(_) => (),
            Err(_) => return Err(Error::EndOfBuffer),
        };
    }

    Ok(())
}

pub fn write_vec_u128(writer: &mut dyn Write, v: &Vec<u128>) -> Result<(), Error> {
    match write_u32(writer, v.len() as u32) {
        Ok(_) => (),
        Err(_) => return Err(Error::EndOfBuffer),
    };

    for elem in v {
        match write_u128(writer, *elem) {
            Ok(_) => (),
            Err(_) => return Err(Error::EndOfBuffer),
        };
    }

    Ok(())
}

pub fn write_vec_struct<U>(writer: &mut dyn Write, v: &Vec<U>) -> Result<(), Error>
where
    U: Writable,
{
    match write_u32(writer, v.len() as u32) {
        Ok(_) => (),
        Err(_) => return Err(Error::EndOfBuffer),
    };

    for elem in v {
        match write_struct(writer, elem) {
            Ok(_) => (),
            Err(_) => return Err(Error::EndOfBuffer),
        };
    }

    Ok(())
}

pub fn write_string(writer: &mut dyn Write, s: String) -> Result<(), Error> {
    let v = s.as_bytes();
    match write_u32(writer, v.len() as u32) {
        Ok(_) => (),
        Err(_) => return Err(Error::EndOfBuffer),
    };

    match writer.write_all(v) {
        Ok(_) => (),
        Err(_) => return Err(Error::InvalidFormat),
    };

    Ok(())
}

pub fn write_signature(writer: &mut dyn Write, s: &Signature) -> Result<(), Error> {
    match writer.write_all(&s.to_bytes()) {
        Ok(_) => (),
        Err(_) => return Err(Error::EndOfBuffer),
    };

    Ok(())
}

pub trait Writable {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<(), Error>
    where
        Self: Sized;
}
