use std::{
    borrow::Borrow,
    fmt::{self, Write},
};

use crate::hex;

pub(crate) const KEY_LENGTH: usize = 32;

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Key(pub [u8; KEY_LENGTH]);

impl Key {
    pub const ZERO: Key = Key([0u8; KEY_LENGTH]);

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

    pub fn into_bytes(self) -> [u8; KEY_LENGTH] {
        self.0
    }
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for Key {
    fn to_string(&self) -> String {
        self.0.iter().flat_map(|x| hex::write_byte(*x)).collect()
    }
}

#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub struct KeyRef([u8; KEY_LENGTH]);

impl KeyRef {
    pub fn new(key: &Key) -> &Self {
        unsafe { std::mem::transmute(&key.0) }
    }

    pub fn new_slice(keys: &[Key]) -> &[Self] {
        unsafe { std::mem::transmute(keys) }
    }

    pub fn from_bytes(bytes: &[u8; KEY_LENGTH]) -> &Self {
        unsafe { std::mem::transmute(bytes) }
    }
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for KeyRef {
    fn to_string(&self) -> String {
        self.0.iter().flat_map(|x| hex::write_byte(*x)).collect()
    }
}

impl fmt::Debug for KeyRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for x in self.0.iter() {
            let [msb, lsb] = hex::write_byte(*x);
            f.write_char(msb)?;
            f.write_char(lsb)?;
        }
        Ok(())
    }
}

impl AsRef<KeyRef> for KeyRef {
    fn as_ref(&self) -> &KeyRef {
        self
    }
}

impl fmt::Debug for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        KeyRef::new(self).fmt(f)
    }
}

impl AsRef<KeyRef> for Key {
    fn as_ref(&self) -> &KeyRef {
        KeyRef::new(self)
    }
}

impl Borrow<KeyRef> for Key {
    fn borrow(&self) -> &KeyRef {
        KeyRef::new(self)
    }
}

impl ToOwned for KeyRef {
    type Owned = Key;

    fn to_owned(&self) -> Self::Owned {
        Key(self.0)
    }
}
