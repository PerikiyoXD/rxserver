use tokio::io::{self, ErrorKind};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteOrder {
    Little = b'l',
    Big = b'B',
}

impl ByteOrder {
    pub fn from_marker(marker: u8) -> Option<Self> {
        match marker {
            b'l' => Some(Self::Little),
            b'B' => Some(Self::Big),
            _ => None,
        }
    }

    pub fn marker(self) -> u8 {
        self as u8
    }

    pub fn native() -> Self {
        if cfg!(target_endian = "little") {
            Self::Little
        } else {
            Self::Big
        }
    }

    pub fn is_native(self) -> bool {
        self == Self::native()
    }
}

pub trait ByteOrderConversion: Sized {
    fn from_bytes(bytes: &[u8], order: ByteOrder) -> Self;
    fn to_bytes(self, order: ByteOrder) -> Vec<u8>;
}

macro_rules! impl_byte_order {
    ($($t:ty),*) => {
        $(
            impl ByteOrderConversion for $t {
                fn from_bytes(bytes: &[u8], order: ByteOrder) -> Self {
                    match order {
                        ByteOrder::Little => Self::from_le_bytes(bytes.try_into().unwrap()),
                        ByteOrder::Big => Self::from_be_bytes(bytes.try_into().unwrap()),
                    }
                }

                fn to_bytes(self, order: ByteOrder) -> Vec<u8> {
                    match order {
                        ByteOrder::Little => self.to_le_bytes().to_vec(),
                        ByteOrder::Big => self.to_be_bytes().to_vec(),
                    }
                }
            }
        )*
    };
}

impl_byte_order!(u16, u32, i16, i32);

impl ByteOrderConversion for u8 {
    fn from_bytes(bytes: &[u8], _: ByteOrder) -> Self {
        bytes[0]
    }

    fn to_bytes(self, _: ByteOrder) -> Vec<u8> {
        vec![self]
    }
}

pub trait X11WriteExt {
    fn write<T: ByteOrderConversion>(&mut self, val: T, order: ByteOrder) -> io::Result<()>;
}

impl X11WriteExt for Vec<u8> {
    fn write<T: ByteOrderConversion>(&mut self, val: T, order: ByteOrder) -> io::Result<()> {
        self.extend(val.to_bytes(order));
        Ok(())
    }
}

pub trait X11ReadExt {
    fn read<T: ByteOrderConversion>(&self, offset: &mut usize, order: ByteOrder) -> io::Result<T>;
}

impl X11ReadExt for [u8] {
    fn read<T: ByteOrderConversion>(&self, offset: &mut usize, order: ByteOrder) -> io::Result<T> {
        let size = std::mem::size_of::<T>();
        if *offset + size > self.len() {
            return Err(io::Error::new(ErrorKind::UnexpectedEof, "Not enough bytes"));
        }
        let val = T::from_bytes(&self[*offset..*offset + size], order);
        *offset += size;
        Ok(val)
    }
}
