use std::{
    borrow::Borrow,
    fmt::{self, Write},
};

use bytes::Bytes;

use crate::hex;

pub const KEY_LENGTH: usize = 32;

#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub struct KeyRef([u8; KEY_LENGTH]);

impl fmt::Debug for KeyRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for x in self.0 {
            let [msb, lsb] = hex::write_byte(x);
            f.write_char(msb)?;
            f.write_char(lsb)?;
        }
        Ok(())
    }
}

impl KeyRef {
    pub(crate) fn clone(&self) -> Self {
        Self(self.copy_bytes())
    }

    pub(crate) fn into_array(self) -> [u8; KEY_LENGTH] {
        self.0
    }
}

impl KeyRef {
    pub const ZERO: Self = Self([0u8; KEY_LENGTH]);

    pub fn new(bytes: &[u8; KEY_LENGTH]) -> &Self {
        unsafe { std::mem::transmute(bytes) }
    }

    pub fn from_bytes(key: &[u8]) -> Option<&Self> {
        key.as_array().map(Self::new)
    }

    pub fn copy_bytes(&self) -> [u8; KEY_LENGTH] {
        let mut bytes = [0u8; KEY_LENGTH];
        bytes.copy_from_slice(&self.0);
        bytes
    }

    pub fn as_array(&self) -> &[u8; KEY_LENGTH] {
        &self.0
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn as_key(&self, bytes: &Bytes) -> Key {
        Key(bytes.slice_ref(self.as_bytes()))
    }

    pub fn hex_display(&self) -> impl fmt::Display {
        fmt::from_fn(|f| fmt::Debug::fmt(self, f))
    }

    pub fn to_hex_string(&self) -> String {
        self.hex_display().to_string()
    }
}

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Key(Bytes);

impl fmt::Debug for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for x in &self.0 {
            let [msb, lsb] = hex::write_byte(*x);
            f.write_char(msb)?;
            f.write_char(lsb)?;
        }
        Ok(())
    }
}

impl Key {
    pub fn new(bytes: Bytes) -> Option<Self> {
        (bytes.len() == KEY_LENGTH).then(|| Self(bytes))
    }

    pub fn from_array(key: [u8; KEY_LENGTH]) -> Self {
        Self(Bytes::from_owner(key))
    }

    pub fn copy_from_array(bytes: &[u8; KEY_LENGTH]) -> Self {
        Self(Bytes::copy_from_slice(bytes))
    }

    pub fn copy_from_slice(bytes: &[u8]) -> Option<Self> {
        (bytes.len() == KEY_LENGTH).then(|| Self(Bytes::copy_from_slice(bytes)))
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let (chunks, overflow) = hex.as_bytes().as_chunks::<2>();
        if !overflow.is_empty() || chunks.len() != KEY_LENGTH {
            return None;
        }
        let mut bytes = [0u8; KEY_LENGTH];
        for (index, [msb, lsb]) in chunks.iter().enumerate() {
            bytes[index] = hex::read_byte(*msb, *lsb)?;
        }
        Some(Self(Bytes::from_owner(bytes)))
    }

    pub fn as_array(&self) -> &[u8; KEY_LENGTH] {
        self.0.as_array().expect("Wrong key length")
    }

    pub fn into_bytes(self) -> Bytes {
        self.0
    }

    pub fn hex_display(&self) -> impl fmt::Display {
        fmt::from_fn(|f| fmt::Debug::fmt(self, f))
    }

    pub fn to_hex_string(&self) -> String {
        self.hex_display().to_string()
    }
}

impl AsRef<KeyRef> for KeyRef {
    fn as_ref(&self) -> &KeyRef {
        self
    }
}

impl AsRef<KeyRef> for Key {
    fn as_ref(&self) -> &KeyRef {
        KeyRef::new(self.as_array())
    }
}

impl Borrow<KeyRef> for Key {
    fn borrow(&self) -> &KeyRef {
        KeyRef::new(self.as_array())
    }
}

impl PartialEq<KeyRef> for Key {
    fn eq(&self, other: &KeyRef) -> bool {
        self.as_ref() == other
    }
}

impl PartialEq<Key> for KeyRef {
    fn eq(&self, other: &Key) -> bool {
        self == other.as_ref()
    }
}
