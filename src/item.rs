use std::{fmt, mem::MaybeUninit};

use crate::{
    functional::Setter,
    key::{KEY_LENGTH, Key, KeyRef},
    value::{VALUE_LENGTH, Value},
};

pub(crate) const ITEM_LENGTH: usize = KEY_LENGTH + VALUE_LENGTH;

#[derive(Clone, Debug)]
pub struct Item {
    pub key: Key,
    pub value: Value,
}

impl Item {
    pub fn into_bytes(self) -> [u8; ITEM_LENGTH] {
        let mut bytes = MaybeUninit::<[u8; ITEM_LENGTH]>::uninit();
        unsafe {
            let ptr = bytes.as_mut_ptr().cast::<u8>();
            ptr.cast::<[u8; KEY_LENGTH]>().write(*self.key.as_array());
            ptr.add(KEY_LENGTH)
                .cast::<[u8; VALUE_LENGTH]>()
                .write(self.value.into_bytes());
            bytes.assume_init()
        }
    }

    pub fn into_ref(self) -> ItemRef {
        ItemRef(self.into_bytes())
    }
}

#[derive(Clone)]
pub struct ItemRef([u8; ITEM_LENGTH]);

impl ItemRef {
    pub fn from_bytes(bytes: &[u8; ITEM_LENGTH]) -> &Self {
        unsafe { std::mem::transmute(bytes) }
    }

    pub fn cast_vec(vec: Vec<Self>) -> Vec<u8> {
        let (ptr, len, cap) = vec.into_raw_parts();
        unsafe { Vec::from_raw_parts(ptr.cast(), len * ITEM_LENGTH, cap * ITEM_LENGTH) }
    }

    pub fn new(key: Key, value: Value) -> Self {
        Self(Item { key, value }.into_bytes())
    }

    pub fn key(&self) -> &KeyRef {
        let key = unsafe { &*self.0.as_ptr().cast() };
        KeyRef::new(key)
    }

    pub fn set_key(&mut self, key: &KeyRef) {
        unsafe {
            self.0
                .as_mut_ptr()
                .cast::<[u8; KEY_LENGTH]>()
                .write(*key.as_array());
        }
    }

    pub fn value(&self) -> Value {
        let value = unsafe { &*self.0.as_ptr().add(KEY_LENGTH).cast() };
        Value::from_bytes(value)
    }

    pub fn set_value(&mut self, value: Value) {
        unsafe {
            self.0
                .as_mut_ptr()
                .add(KEY_LENGTH)
                .cast::<[u8; VALUE_LENGTH]>()
                .write(value.into_bytes());
        }
    }

    pub fn into_bytes(self) -> [u8; ITEM_LENGTH] {
        self.0
    }
}

impl Setter<Value> for &mut ItemRef {
    fn set(&mut self, value: Value) {
        self.set_value(value);
    }
}

impl fmt::Debug for ItemRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ItemRef")
            .field("key", self.key())
            .field("value", &self.value())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::*;

    #[test]
    fn test_item_cast_vec() {
        const COUNT: usize = 5;

        let items: Vec<_> = (0u32..COUNT as u32)
            .map(|index| {
                let key =
                    Key::new(Bytes::from_owner([index.try_into().unwrap(); KEY_LENGTH])).unwrap();
                let value = Value {
                    offset: index,
                    length: index,
                    decompressed: index,
                    extra: [0; _],
                };
                ItemRef::new(key, value)
            })
            .collect();
        let bytes = ItemRef::cast_vec(items);
        assert_eq!(bytes.len(), COUNT * ITEM_LENGTH);
    }
}
