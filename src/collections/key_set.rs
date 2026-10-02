use std::{borrow::Borrow, iter, ops::Deref, slice, vec};

use crate::{Key, KeyRef, collections::key_slice::KeySlice};

#[derive(Debug)]
pub struct KeySet(Vec<KeyRef>);

impl Deref for KeySet {
    type Target = KeySlice;

    fn deref(&self) -> &Self::Target {
        KeySlice::new_unchecked(&self.0)
    }
}

impl AsRef<KeySlice> for KeySet {
    fn as_ref(&self) -> &KeySlice {
        KeySlice::new_unchecked(&self.0)
    }
}

impl Borrow<KeySlice> for KeySet {
    fn borrow(&self) -> &KeySlice {
        KeySlice::new_unchecked(&self.0)
    }
}

impl KeySet {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn insert(&mut self, key: impl AsRef<KeyRef>) -> bool {
        let key = key.as_ref();
        if let Err(index) = self.0.binary_search(key) {
            self.0.insert(index, key.clone());
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, key: impl AsRef<KeyRef>) -> bool {
        let key = key.as_ref();
        if let Ok(index) = self.0.binary_search(key) {
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

impl<T: AsRef<KeyRef>> FromIterator<T> for KeySet {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut keys = iter
            .into_iter()
            .map(|key| key.as_ref().clone())
            .collect::<Vec<_>>();
        keys.sort_unstable();
        keys.dedup();
        Self(keys)
    }
}

impl<'a> IntoIterator for &'a KeySet {
    type Item = &'a KeyRef;
    type IntoIter = slice::Iter<'a, KeyRef>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl IntoIterator for KeySet {
    type Item = Key;
    type IntoIter = iter::Map<vec::IntoIter<KeyRef>, fn(KeyRef) -> Key>;

    fn into_iter(self) -> Self::IntoIter {
        self.0
            .into_iter()
            .map(|key| Key::from_array(key.into_array()))
    }
}
