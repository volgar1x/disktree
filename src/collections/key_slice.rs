use std::{ops::Deref, slice};

use crate::Key;

#[derive(Debug)]
pub struct KeySlice([Key]);

impl Deref for KeySlice {
    type Target = [Key];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl KeySlice {
    pub fn new_unchecked(keys: &[Key]) -> &Self {
        unsafe { std::mem::transmute(keys) }
    }

    pub fn contains(&mut self, key: impl AsRef<Key>) -> bool {
        let key = key.as_ref();
        self.0.binary_search_by(|item| item.cmp(key)).is_ok()
    }
}

impl<'a> IntoIterator for &'a KeySlice {
    type Item = &'a Key;
    type IntoIter = slice::Iter<'a, Key>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
