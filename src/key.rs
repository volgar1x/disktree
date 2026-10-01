use std::fmt::{self, Write};

use crate::hex;

pub const KEY_LENGTH: usize = 32;

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Key([u8; KEY_LENGTH]);

impl Key {
    pub const ZERO: Key = Key([0u8; KEY_LENGTH]);

    pub fn new(key: [u8; KEY_LENGTH]) -> Self {
        Self(key)
    }

    pub fn from_ref(bytes: &[u8; KEY_LENGTH]) -> &Self {
        unsafe { std::mem::transmute(bytes) }
    }

    pub fn from_bytes(key: &[u8]) -> Option<&Self> {
        key.as_array().map(Self::from_ref)
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        if hex.len() != KEY_LENGTH * 2 {
            return None;
        }
        let chunks = hex.as_bytes().as_chunks::<2>().0;
        let mut bytes = [0u8; KEY_LENGTH];
        for (index, [msb, lsb]) in chunks.iter().enumerate() {
            bytes[index] = hex::read_byte(*msb, *lsb)?;
        }
        Some(Self(bytes))
    }

    pub fn copy_from_slice(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != KEY_LENGTH {
            return None;
        }
        let mut key = [0u8; KEY_LENGTH];
        key.copy_from_slice(bytes);
        Some(Self(key))
    }

    pub fn as_bytes(&self) -> &[u8; KEY_LENGTH] {
        &self.0
    }

    pub fn into_bytes(self) -> [u8; KEY_LENGTH] {
        self.0
    }

    pub fn into_box(self) -> Box<[u8]> {
        let mut boxed = Box::<[u8]>::new_uninit_slice(KEY_LENGTH);
        unsafe {
            boxed.assume_init_mut().copy_from_slice(&self.0);
            boxed.assume_init()
        }
    }

    pub fn into_vec(self) -> Vec<u8> {
        let mut vec = Vec::with_capacity(KEY_LENGTH);
        unsafe {
            vec.spare_capacity_mut()
                .assume_init_mut()
                .copy_from_slice(&self.0);
            vec.set_len(KEY_LENGTH);
        }
        vec
    }

    pub fn hex_display(&self) -> impl fmt::Display {
        fmt::from_fn(|f| fmt::Debug::fmt(self, f))
    }

    pub fn to_hex_string(&self) -> String {
        self.hex_display().to_string()
    }
}

impl fmt::Debug for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for x in self.0.iter() {
            let [msb, lsb] = hex::write_byte(*x);
            f.write_char(msb)?;
            f.write_char(lsb)?;
        }
        Ok(())
    }
}
