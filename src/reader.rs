use ed25519_dalek::Signature;

use crate::error::Error;

pub struct Reader {
    bytes: Vec<u8>,
}

impl Reader {
    pub fn new(bytes: Vec<u8>) -> Self {
        Reader { bytes: bytes }
    }

    pub fn read_bytes<U>(&mut self, bytes_count: usize) -> Result<U, Error>
    where
        U: TryFrom<Vec<u8>>,
    {
        if bytes_count > self.bytes.len() {
            return Err(Error::EndOfBuffer);
        }

        let vec: Vec<u8> = self.bytes.drain(0..bytes_count).collect();
        Ok(match vec.try_into() {
            Ok(res) => res,
            Err(_) => return Err(Error::EndOfBuffer),
        })
    }
    pub fn read<U>(&mut self) -> Result<U, Error>
    where
        U: Readable<U>,
    {
        U::from_reader(self)
    }

    pub fn read_u32(&mut self) -> Result<u32, Error> {
        const LEN: usize = (u32::BITS / 8) as usize;

        let slice: [u8; LEN] = match self.read_bytes(LEN) {
            Ok(s) => s,
            Err(_) => return Err(Error::EndOfBuffer),
        };
        let num = u32::from_be_bytes(slice);
        Ok(num)
    }

    pub fn read_u64(&mut self) -> Result<u64, Error> {
        const LEN: usize = (u64::BITS / 8) as usize;

        let slice: [u8; LEN] = match self.read_bytes(LEN) {
            Ok(s) => s,
            Err(_) => return Err(Error::EndOfBuffer),
        };
        let num = u64::from_be_bytes(slice);
        Ok(num)
    }

    pub fn read_u128(&mut self) -> Result<u128, Error> {
        const LEN: usize = (u128::BITS / 8) as usize;

        let slice: [u8; LEN] = match self.read_bytes(LEN) {
            Ok(s) => s,
            Err(_) => return Err(Error::EndOfBuffer),
        };
        let num = u128::from_be_bytes(slice);
        Ok(num)
    }

    pub fn read_signature(&mut self) -> Result<Signature, Error> {
        let signature_slice: [u8; Signature::BYTE_SIZE] =
            match self.read_bytes(Signature::BYTE_SIZE) {
                Ok(s) => s,
                Err(_) => return Err(Error::EndOfBuffer),
            };

        match Signature::from_bytes(&signature_slice) {
            Ok(s) => Ok(s),
            Err(_) => return Err(Error::EndOfBuffer),
        }
    }

    pub fn read_vec<U>(&mut self) -> Result<Vec<U>, Error>
    where
        U: Readable<U>,
    {
        let len = match self.read_u32() {
            Ok(s) => s,
            Err(_) => return Err(Error::EndOfBuffer),
        } as usize;

        let mut v: Vec<U> = vec![];
        for _ in 0..len {
            v.push(match U::from_reader(self) {
                Ok(s) => s,
                Err(_) => return Err(Error::EndOfBuffer),
            });
        }

        Ok(v)
    }

    pub fn read_vec_u32(&mut self) -> Result<Vec<u32>, Error> {
        let len = match self.read_u32() {
            Ok(s) => s,
            Err(_) => return Err(Error::EndOfBuffer),
        } as usize;

        let mut v: Vec<u32> = vec![];
        for _ in 0..len {
            v.push(match self.read_u32() {
                Ok(s) => s,
                Err(_) => return Err(Error::EndOfBuffer),
            });
        }

        Ok(v)
    }

    pub fn read_vec_u64(&mut self) -> Result<Vec<u64>, Error> {
        let len = match self.read_u32() {
            Ok(s) => s,
            Err(_) => return Err(Error::EndOfBuffer),
        } as usize;

        let mut v: Vec<u64> = vec![];
        for _ in 0..len {
            v.push(match self.read_u64() {
                Ok(s) => s,
                Err(_) => return Err(Error::EndOfBuffer),
            });
        }

        Ok(v)
    }

    pub fn read_vec_u128(&mut self) -> Result<Vec<u128>, Error> {
        let len = match self.read_u32() {
            Ok(s) => s,
            Err(_) => return Err(Error::EndOfBuffer),
        } as usize;

        let mut v: Vec<u128> = vec![];
        for _ in 0..len {
            v.push(match self.read_u128() {
                Ok(s) => s,
                Err(_) => return Err(Error::EndOfBuffer),
            });
        }

        Ok(v)
    }

    pub fn is_empty(&self) -> bool {
        return self.bytes.is_empty();
    }
}

pub trait Readable<T> {
    fn from_reader(reader: &mut Reader) -> Result<T, Error>;
}
