use std::{fmt, ops::Deref, slice};

use crate::{
    Key, Value,
    collections::GetByKey,
    item::{ITEM_LENGTH, ItemRef},
};

pub struct ItemSlice([ItemRef]);

impl ItemSlice {
    pub fn from_bytes(bytes: &[u8]) -> (&Self, &[u8]) {
        let len = bytes.len() / ITEM_LENGTH;
        let overflow = bytes.len() % ITEM_LENGTH;
        let ptr: *const ItemRef = bytes.as_ptr().cast();
        let items = Self::new(unsafe { std::slice::from_raw_parts(ptr, len) });
        let rest = &bytes[bytes.len() - overflow..];
        (items, rest)
    }

    pub fn new(items: &[ItemRef]) -> &Self {
        unsafe { std::mem::transmute(items) }
    }

    pub fn as_bytes(&self) -> &[u8] {
        let len = self.len() * ITEM_LENGTH;
        let ptr: *const u8 = unsafe { &*self.0.as_ptr().cast() };
        unsafe { std::slice::from_raw_parts(ptr, len) }
    }
}

impl GetByKey for ItemSlice {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn keys(&self) -> impl Iterator<Item = &Key> {
        self.0.iter().map(|item| item.key())
    }

    fn contains(&self, key: &Key) -> bool {
        self.0.binary_search_by_key(&key, |item| item.key()).is_ok()
    }

    fn get(&self, key: &Key) -> Option<Value> {
        self.0
            .binary_search_by_key(&key, |item| item.key())
            .ok()
            .map(|index| self.0[index].value())
    }
}

impl fmt::Debug for ItemSlice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.0.iter().map(|item| (item.key(), item.value())))
            .finish()
    }
}

impl<'a> IntoIterator for &'a ItemSlice {
    type Item = &'a ItemRef;
    type IntoIter = slice::Iter<'a, ItemRef>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Deref for ItemSlice {
    type Target = [ItemRef];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
