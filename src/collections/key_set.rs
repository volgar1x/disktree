use std::ops::Deref;

use crate::{Key, KeyRef, collections::key_slice::KeySlice};

#[derive(Clone, Debug)]
pub struct KeySet(Vec<Key>);

impl Deref for KeySet {
    type Target = KeySlice;

    fn deref(&self) -> &Self::Target {
        let slice: &[Key] = &self.0;
        unsafe { std::mem::transmute(slice) }
    }
}

impl KeySet {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn insert(&mut self, key: Key) -> bool {
        if let Err(index) = self.0.binary_search(&key) {
            self.0.insert(index, key);
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, key: impl AsRef<KeyRef>) -> bool {
        let key = key.as_ref();
        if let Ok(index) = self.0.binary_search_by(|item| item.as_ref().cmp(key)) {
            self.0.remove(index);
            true
        } else {
            false
        }
    }
}

impl Default for KeySet {
    fn default() -> Self {
        Self::new()
    }
}

impl FromIterator<Key> for KeySet {
    fn from_iter<T: IntoIterator<Item = Key>>(iter: T) -> Self {
        let mut keys = iter.into_iter().collect::<Vec<_>>();
        keys.sort_unstable();
        keys.dedup();
        Self(keys)
    }
}

impl IntoIterator for KeySet {
    type Item = Key;
    type IntoIter = std::vec::IntoIter<Key>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl AsRef<KeySlice> for KeySet {
    fn as_ref(&self) -> &KeySlice {
        KeySlice::new(KeyRef::new_slice(&self.0))
    }
}
